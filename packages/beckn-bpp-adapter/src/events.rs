//! Ride lifecycle events → unsolicited callbacks (Step 7).
//!
//! The api service publishes every ride's lifecycle to the
//! `swg:stream:ride_events` Redis stream (arrived / started / ended /
//! cancelled). The adapter joins that stream with its OWN consumer group —
//! the notification service's group is untouched, each group receives every
//! message — filters for rides that belong to a Beckn order, and translates
//! them into `on_status` / `on_cancel` callbacks. Purely a consumer: the
//! dispatch core and the event producers are never modified (Rule #3).

use std::sync::Arc;
use std::time::Duration;

use redis_store::RedisConnectionPool;
use redis_store::events::{
    EventHandlerFn, RIDE_EVENTS_STREAM, run_event_consumer, run_recovery_pass,
};
use serde_json::Value;

use crate::AppState;
use crate::internal::{parse_assigned, push_on_cancel, push_on_status};

/// Our consumer group on the ride-events stream. Distinct from the
/// notification service's group so both see every event.
pub const BECKN_RIDE_EVENTS_GROUP: &str = "beckn-adapter-workers";

/// How often the PEL recovery pass runs, and how long a pending message must
/// have sat unacked before it is reclaimed. Idle > one consumer BLOCK cycle so
/// a healthy in-flight message is never stolen from the live consumer.
const RECOVERY_INTERVAL_SECS: u64 = 30;
const RECOVERY_IDLE_MS: u64 = 60_000;

/// What a ride event means for the Beckn order lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RideEventKind {
    Arrived,
    Started,
    Ended,
    Cancelled { by_consumer: bool },
}

/// Map a raw stream payload to `(ride_id, kind)`. `None` = not a lifecycle
/// event we translate (unknown type or malformed) — ack and move on.
pub fn translate_ride_event(payload: &str) -> Option<(String, RideEventKind)> {
    let value: Value = serde_json::from_str(payload).ok()?;
    let ride_id = value.get("ride_id")?.as_str()?.to_string();
    let kind = match value.get("event_type")?.as_str()? {
        "DriverArrivedEvent" => RideEventKind::Arrived,
        "RideStartEvent" => RideEventKind::Started,
        "RideEndEvent" => RideEventKind::Ended,
        "RideCancelEvent" => {
            // canceled_by: 1 = customer, 0 = driver, 2 = admin (see the api's
            // cancel_ride) — anything not the customer is the provider side.
            let by_consumer = value
                .pointer("/event_payload/RideCancel/canceled_by")
                .and_then(Value::as_i64)
                == Some(1);
            RideEventKind::Cancelled { by_consumer }
        }
        _ => return None,
    };
    Some((ride_id, kind))
}

/// Handle one ride event: if the ride belongs to a Beckn order, advance the
/// order and push the matching callback.
///
/// Returns the ack decision: `true` = done with this message (processed, not
/// ours, or junk), `false` = a transient DB failure — leave it in the PEL for
/// the recovery pass to replay. Replays are safe: `set_fulfillment` is an
/// idempotent update and a duplicated `on_status` is within Beckn's
/// at-least-once callback semantics.
pub async fn handle_ride_event(state: &AppState, payload: &str) -> bool {
    let Some((ride_id, kind)) = translate_ride_event(payload) else {
        return true;
    };
    let row = match state.correlation.find_confirm_by_ride_id(&ride_id).await {
        Ok(Some(row)) => row,
        // Not a network booking — the overwhelmingly common case.
        Ok(None) => return true,
        Err(e) => {
            tracing::error!(
                ride_id,
                "correlation lookup failed for ride event: {e:#}"
            );
            crate::metrics::bump(&state.metrics.ride_events_requeued);
            return false;
        }
    };

    let (order_status, state_code) = match &kind {
        RideEventKind::Arrived => ("ACTIVE", "RIDE_ARRIVED_PICKUP"),
        RideEventKind::Started => ("ACTIVE", "RIDE_STARTED"),
        RideEventKind::Ended => ("COMPLETE", "RIDE_ENDED"),
        RideEventKind::Cancelled { .. } => ("CANCELLED", "RIDE_CANCELLED"),
    };
    if let Err(e) = state
        .correlation
        .set_fulfillment(&ride_id, order_status, state_code, None)
        .await
    {
        tracing::error!(ride_id, "failed to persist {state_code}: {e:#}");
        crate::metrics::bump(&state.metrics.ride_events_requeued);
        return false;
    }

    tracing::info!(
        ride_id,
        transaction_id = %row.transaction_id,
        state_code,
        "ride lifecycle event → unsolicited callback"
    );
    match kind {
        RideEventKind::Cancelled { by_consumer } => {
            let by = if by_consumer { "CONSUMER" } else { "PROVIDER" };
            push_on_cancel(state, &row, by).await;
        }
        _ => {
            let assigned = parse_assigned(&row);
            push_on_status(
                state,
                &row,
                order_status,
                state_code,
                assigned.as_ref(),
            )
            .await;
        }
    }
    crate::metrics::bump(&state.metrics.ride_events_handled);
    true
}

/// Join the ride-events stream and translate forever. Spawn once at boot when
/// Redis is available.
pub async fn run_ride_event_consumer(
    state: AppState,
    redis: Arc<RedisConnectionPool>,
) {
    if let Err(e) = redis
        .ensure_consumer_group(RIDE_EVENTS_STREAM, BECKN_RIDE_EVENTS_GROUP)
        .await
    {
        tracing::error!(
            "could not create ride-events consumer group — lifecycle \
             on_status DISABLED: {e:?}"
        );
        return;
    }
    let client = redis.pool.next().clone();
    tracing::info!(
        stream = RIDE_EVENTS_STREAM,
        group = BECKN_RIDE_EVENTS_GROUP,
        "consuming ride lifecycle events"
    );

    // Recovery pass: reclaim messages another (crashed) consumer read but
    // never acked, plus our own handler-declined ones, and replay them. Without
    // this, a crash between delivery and ack silently loses the on_status.
    let recovery_state = state.clone();
    let recovery_client = redis.pool.next().clone();
    tokio::spawn(async move {
        let handler: EventHandlerFn = Box::new(move |_msg_id, payload| {
            let state = recovery_state.clone();
            Box::pin(async move {
                crate::metrics::bump(&state.metrics.ride_events_recovered);
                handle_ride_event(&state, &payload).await
            })
        });
        let mut tick =
            tokio::time::interval(Duration::from_secs(RECOVERY_INTERVAL_SECS));
        loop {
            tick.tick().await;
            if let Err(e) = run_recovery_pass(
                &recovery_client,
                RIDE_EVENTS_STREAM,
                BECKN_RIDE_EVENTS_GROUP,
                "beckn-recovery",
                RECOVERY_IDLE_MS,
                &handler,
            )
            .await
            {
                tracing::warn!(
                    stream = RIDE_EVENTS_STREAM,
                    "ride-events recovery pass failed: {e:?}"
                );
            }
        }
    });

    run_event_consumer(
        client,
        RIDE_EVENTS_STREAM,
        BECKN_RIDE_EVENTS_GROUP,
        "beckn-0".to_string(),
        Box::new(move |_msg_id, payload| {
            let state = state.clone();
            Box::pin(async move { handle_ride_event(&state, &payload).await })
        }),
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_lifecycle_events() {
        let started = serde_json::json!({
            "event_type": "RideStartEvent",
            "ride_id": "01RIDE",
            "driver_id": "01D",
            "rider_id": "01R",
        })
        .to_string();
        assert_eq!(
            translate_ride_event(&started),
            Some(("01RIDE".to_string(), RideEventKind::Started))
        );

        let cancelled = serde_json::json!({
            "event_type": "RideCancelEvent",
            "ride_id": "01RIDE",
            "event_payload": { "RideCancel": { "canceled_by": 1 } },
        })
        .to_string();
        assert_eq!(
            translate_ride_event(&cancelled),
            Some((
                "01RIDE".to_string(),
                RideEventKind::Cancelled { by_consumer: true }
            ))
        );

        let driver_cancelled = serde_json::json!({
            "event_type": "RideCancelEvent",
            "ride_id": "01RIDE",
            "event_payload": { "RideCancel": { "canceled_by": 0 } },
        })
        .to_string();
        assert_eq!(
            translate_ride_event(&driver_cancelled),
            Some((
                "01RIDE".to_string(),
                RideEventKind::Cancelled { by_consumer: false }
            ))
        );

        // Unknown types and junk are skipped, not errors.
        let unknown = r#"{"event_type":"SomethingElse","ride_id":"01RIDE"}"#;
        assert_eq!(translate_ride_event(unknown), None);
        assert_eq!(translate_ride_event("not json"), None);
    }
}
