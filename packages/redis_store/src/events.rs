use crate::{RedisConnectionPool, r_types::RedisError};
use fred::prelude::{Client, StreamsInterface};
use fred::types::streams::XReadResponse;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ── Stream keys ────────────────────────────────────────────────────────────────

/// Ride lifecycle events: start, cancel, driver arrived, end.
pub const RIDE_EVENTS_STREAM: &str = "swg:stream:ride_events";
/// Driver offer events sent to the rider during dispatch matching.
pub const NEXT_DRIVER_OFFERS_STREAM: &str = "swg:stream:next_driver_offers";

// ── Pub/sub channels ───────────────────────────────────────────────────────────

/// Live driver GPS fixes during an active ride. The notification service
/// subscribes here and fans each `LocationEvent` out to the customer app's
/// `WatchDriverLocationChanges` gRPC stream, keyed by `ride_id`.
pub const DRIVER_LOCATION_CHANGE_CHANNEL: &str =
    "driver_location_change_channel";

// ── Consumer group names ───────────────────────────────────────────────────────

pub const RIDE_EVENTS_GROUP: &str = "ride-event-workers";
pub const NEXT_DRIVER_OFFERS_GROUP: &str = "driver-offer-workers";

// ── Producer ───────────────────────────────────────────────────────────────────

/// Holds the target stream key for a set of related events.
pub struct EventsManger<'a> {
    pub stream_key: &'a str,
}

impl<'a> EventsManger<'a> {
    pub fn new(stream_key: &'a str) -> Self {
        EventsManger { stream_key }
    }

    /// Append `event` to the stream as a JSON payload field.
    /// ACK is the responsibility of the consumer; fire-and-forget for the producer.
    pub async fn publish_event<T: serde::Serialize + Send + Sync>(
        &self,
        event: Option<&T>,
        redis_client: &RedisConnectionPool,
    ) -> Result<(), RedisError> {
        redis_client.xadd_event(self.stream_key, &event).await.map(|_| ())
    }
}

// ── Consumer group setup ───────────────────────────────────────────────────────

/// Idempotent: create all application consumer groups.
/// Call once at startup before spawning any consumer tasks.
pub async fn ensure_all_groups(
    redis: &RedisConnectionPool,
) -> Result<(), RedisError> {
    let groups = [
        (RIDE_EVENTS_STREAM, RIDE_EVENTS_GROUP),
        (NEXT_DRIVER_OFFERS_STREAM, NEXT_DRIVER_OFFERS_GROUP),
    ];
    for (stream, group) in groups {
        redis.ensure_consumer_group(stream, group).await?;
    }
    Ok(())
}

// ── Consumer loop ──────────────────────────────────────────────────────────────

/// Boxed async handler: receives `(message_id, json_payload)`.
/// Return `true` → message is ACKed.  Return `false` → stays in PEL for recovery.
pub type EventHandlerFn = Box<
    dyn Fn(
            String,
            String,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send>>
        + Send
        + Sync,
>;

/// Run a blocking `XREADGROUP` consumer loop on `stream`/`group`.
///
/// Spawn this inside `tokio::spawn` at application startup.
///
/// ```rust,ignore
/// tokio::spawn(run_event_consumer(
///     dedicated_client,
///     RIDE_EVENTS_STREAM,
///     RIDE_EVENTS_GROUP,
///     "worker-0".into(),
///     Box::new(|msg_id, payload| Box::pin(async move {
///         // process payload …
///         true // ACK on success
///     })),
/// ));
/// ```
pub async fn run_event_consumer(
    client: Client,
    stream: &'static str,
    group: &'static str,
    consumer_name: String,
    handler: EventHandlerFn,
) {
    loop {
        let result: Result<XReadResponse<String, String, String, String>, _> =
            client
                .xreadgroup_map(
                    group,
                    &consumer_name,
                    Some(10u64),    // COUNT
                    Some(5_000u64), // BLOCK ms
                    false, // NOACK: false → entries enter the PEL until xack'd
                    stream,
                    ">", // only messages not yet delivered to this group
                )
                .await;

        let response = match result {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(stream, error = %e, "xreadgroup_map failed, retrying in 1 s");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        let records = match response.get(stream) {
            Some(r) => r.clone(),
            None => continue, // blocked read timed out with no messages
        };

        for (message_id, fields) in records {
            let payload = fields.get("payload").cloned().unwrap_or_default();
            if handler(message_id.clone(), payload).await {
                let ack: Result<i64, _> =
                    client.xack(stream, group, message_id.clone()).await;
                if let Err(e) = ack {
                    tracing::error!(
                        stream,
                        message_id = %message_id,
                        error = %e,
                        "xack failed"
                    );
                }
            } else {
                tracing::warn!(
                    stream,
                    message_id = %message_id,
                    "handler returned false — message remains in PEL"
                );
            }
        }
    }
}

// ── Recovery ───────────────────────────────────────────────────────────────────

/// Reclaim and re-process messages idle for more than `idle_ms` milliseconds.
/// Call this on a periodic interval (e.g. every 30 s) from a background task.
pub async fn run_recovery_pass(
    client: &Client,
    stream: &str,
    group: &str,
    recovery_consumer: &str,
    idle_ms: u64,
    handler: &EventHandlerFn,
) -> Result<(), RedisError> {
    use fred::types::streams::XReadValue;

    let (cursor, values): (String, Vec<XReadValue<String, String, String>>) =
        client
            .xautoclaim_values(
                stream,
                group,
                recovery_consumer,
                idle_ms,
                "0-0",
                Some(50u64),
                false,
            )
            .await
            .map_err(RedisError::RedisStreamError)?;

    if !cursor.is_empty() && cursor != "0-0" {
        tracing::debug!(
            stream,
            cursor,
            "xautoclaim returned mid-PEL cursor, next pass will continue"
        );
    }

    for (msg_id, fields) in values {
        let payload = fields.get("payload").cloned().unwrap_or_default();
        tracing::warn!(stream, msg_id = %msg_id, "reclaiming stale message");
        if handler(msg_id.clone(), payload).await {
            let _: Result<i64, _> = client.xack(stream, group, msg_id).await;
        }
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RideCanceledEvent {
    pub event_id: String,
    pub correlation_id: Option<String>,
    pub timestamp: i64,
    pub event_type: String,
    pub ride_id: String,
    pub driver_id: String,
    pub rider_id: String,
    pub priority: i32,
    pub ack_required: bool,
    pub event_payload: EventPayload,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EventPayload {
    #[serde(rename = "RideCancel")]
    pub ride_cancel: RideCancelPayload,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RideCancelPayload {
    pub reason: String,
    pub canceled_by: i32,
    pub refund_amount: f64,
    pub cancellation_fee: String,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverArrivedEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub event_type: String,
    pub ride_id: String,
    pub driver_id: String,
    pub rider_id: String,
    pub priority: i32,
    pub ack_required: bool,
    pub event_payload: DriverArrivedEventPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverArrivedEventPayload {
    #[serde(rename = "DriverArrived")]
    pub driver_arrived: DriverArrivedPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverArrivedPayload {
    pub arrival_location: GeoLocation,
    pub actual_arrival_time: i64,
    pub wait_time_seconds: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RideStartEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub event_type: String,
    pub ride_id: String,
    pub driver_id: String,
    pub rider_id: String,
    pub priority: i32,
    pub ack_required: bool,
    pub event_payload: RideStartEventPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RideStartEventPayload {
    #[serde(rename = "RideStart")]
    pub ride_start: RideStartPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverInfo {
    pub driver_id: String,
    pub name: String,
    pub phone: String,
    pub photo_url: String,
    pub rating: f64,
    pub license_plate: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RideStartPayload {
    pub start_location: GeoLocation,
    pub destination: GeoLocation,
    pub estimated_fare: f64,
    pub vehicle_type: String,
    pub vehicle_number: String,
    pub driver_info: Option<DriverInfo>,
    pub estimated_duration: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RideEndEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub event_type: String,
    pub ride_id: String,
    pub driver_id: String,
    pub rider_id: String,
    pub priority: i32,
    pub ack_required: bool,
    pub event_payload: RideEndEventPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RideEndEventPayload {
    #[serde(rename = "RideEnd")]
    pub ride_end: RideEndPayload,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RideEndPayload {
    pub end_location: GeoLocation,
    pub distance_km: f64,
    pub duration_seconds: i32,
    pub final_fare: f64,
    pub rider_rating: Option<Rating>,
    pub driver_rating: Option<Rating>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rating {
    pub score: i32,
    pub comment: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub place_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NextDriverOfferEventPayload {
    pub rider_id: String,
    pub driver_image: String,
    pub driver_name: String,
    pub driver_rating: f64,
    pub ride_id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub distance_km: f64,
    pub estimated_arrival_time: f64,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NextDriverOfferEvent {
    pub event_id: String,
    pub timestamp: i64,
    pub ride_request_id: String,
    pub status: i32,
    pub payload: Option<NextDriverOfferEventPayload>,
}
