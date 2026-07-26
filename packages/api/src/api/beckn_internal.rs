//! Private internal plane for the Beckn BPP adapter.
//!
//! The adapter is a separate binary, but dispatch state (offer channels,
//! driver responses) lives in THIS process — so network bookings enter
//! dispatch through this endpoint instead of the rider-app route. It is
//! served on the same private listener as the admin plane (see `main.rs`)
//! and gated by its own shared token in `X-Internal-Token`
//! (`BECKN_INTERNAL_TOKEN`), so it is never reachable from the public
//! internet.
//!
//! The handler mirrors the rider flow exactly: route the trip (OSRM), stash
//! the path under `ride_path_key_id`, register the dispatch channel and
//! enqueue the job — the dispatch core itself is only *called*, never
//! changed. The rider identity is supplied by the adapter: a pre-provisioned
//! "network rider" profile id anchors the DB foreign keys, while the real
//! customer's name/phone (from the Beckn order) ride along in
//! `RiderDataInfo` so the driver sees who they are picking up.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::post,
};
use chrono::Utc;
use redis_store::events::{
    EventPayload, EventsManger, RIDE_EVENTS_STREAM, RideCancelPayload,
    RideCanceledEvent,
};
use redis_store::r_types::{AppError, Radius};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use utils::gen_strings::ulid_string;
use utils::hashing_algo::extract_contact_info;

use crate::APIContext;
use crate::api::ride_request::{
    DispatchJob, RequestDriver, RequestRideData, RiderDataInfo,
};
use crate::cache::keys::ride_path_key_id;
use crate::cache::read_writer::ride_clean_up;
use crate::dispatch::state_machine::DispatchEvent;
use crate::queries::{drivers::DriverQueries, ride::RideQueries};
use crate::schemas::ride_request::RideRequestStatus;
use crate::types::{DriverId, RideId, VehicleCategory};

/// Gate the plane on the Beckn-adapter token. Same transport-level trust
/// model as the admin plane, with a separate secret so the adapter and the
/// admin BFF don't share credentials. An empty configured token disables the
/// plane outright (the middleware rejects everything).
pub async fn beckn_internal_auth(
    State(ctx): State<Arc<APIContext>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected = ctx.config.beckn_internal_token.as_bytes();
    if expected.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let provided = req
        .headers()
        .get("x-internal-token")
        .and_then(|v| v.to_str().ok())
        .map(str::as_bytes);

    match provided {
        Some(tok) if super::admin::constant_time_eq(tok, expected) => {
            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// A network booking ready for dispatch. Built by the beckn-bpp-adapter from
/// a confirmed Beckn order; `request_id` is the adapter-minted ride id and
/// doubles as the idempotency anchor — dispatch refuses a duplicate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BecknRideRequest {
    /// Ride/request id (ULID). Also the `ride_requests.id` once a driver is
    /// offered the job.
    pub request_id: String,
    /// The rider shown to the driver. `id` must be the pre-provisioned
    /// network-rider profile id (FK anchor); name/phone are the real
    /// customer's, straight from the Beckn order.
    pub rider: RiderDataInfo,
    pub from: RequestRideData,
    pub to: RequestRideData,
    /// Quoted fare in whole KES — the fare the BAP's customer already agreed to.
    pub fare: i32,
    pub vehicle_category: VehicleCategory,
    /// Driver search radius around the pickup, metres.
    pub radius_m: f64,
}

#[derive(Debug, Serialize)]
pub struct BecknRideResponse {
    pub request_id: String,
}

/// Pushed to the adapter's `/internal/dispatch/assigned` webhook once a driver
/// has durably accepted the booking (Step 7 — completes dispatch-timing
/// decision B). The adapter turns it into the unsolicited `on_status`
/// RIDE_ASSIGNED. Field names are the wire contract with the adapter's
/// `AssignedInfo`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BecknDriverAssigned {
    pub request_id: String,
    pub driver_id: String,
    pub driver_name: String,
    pub driver_phone: String,
    pub vehicle_plate: Option<String>,
    pub vehicle_model: Option<String>,
    pub vehicle_color: Option<String>,
    pub vehicle_make: Option<String>,
    /// Ride-start OTP minted at offer time; the BAP relays it to the customer.
    pub otp: Option<String>,
}

/// Pushed to `/internal/dispatch/failed` when dispatch ends without an
/// accepted driver. The adapter answers with an `on_cancel` (provider side).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BecknDispatchFailed {
    pub request_id: String,
    pub reason: String,
}

/// Outcome of an internal cancel request, as a machine-readable code the
/// adapter maps onto the protocol reply.
#[derive(Debug, Serialize, Deserialize)]
pub struct BecknCancelResponse {
    /// `dispatch_cancelled` | `cancelled` | `already_closed` | `active_ride`
    /// | `not_found`
    pub outcome: String,
}

pub fn beckn_internal_handlers(ctx: Arc<APIContext>) -> Router {
    Router::new()
        .route("/internal/beckn/ride-request", post(beckn_ride_request))
        .route(
            "/internal/beckn/ride-request/{id}/cancel",
            post(beckn_cancel_ride_request),
        )
        .layer(middleware::from_fn_with_state(
            ctx.clone(),
            beckn_internal_auth,
        ))
        .layer(Extension(ctx))
}

/// Enter a confirmed network booking into dispatch: route → cache path →
/// register channel → enqueue. Mirrors `ride_fair_estimation` +
/// `send_ride_request`, minus the rider-profile lookup (the adapter already
/// carries the customer).
async fn beckn_ride_request(
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<BecknRideRequest>,
) -> Result<(StatusCode, Json<BecknRideResponse>), AppError> {
    let (line_str, distance_km, duration_s) = {
        use crate::request::RidesApiClient;
        use crate::simd_json::parse_from_string;
        use utils::meters_to_km;

        let base_url = if ctx.config.is_dev() {
            "http://127.0.0.1:5000".to_string()
        } else {
            std::env::var("ROUTES_API_URL").expect("ROUTES_API_URL must be set")
        };
        let route_query = RidesApiClient::new_with_retry(&base_url, None);
        let coords = [
            (body.from.geo_point.lon.0, body.from.geo_point.lat.0),
            (body.to.geo_point.lon.0, body.to.geo_point.lat.0),
        ];
        let route = route_query
            .get_ride_path_and_distance(
                &coords,
                "overview=full&steps=true&geometries=geojson",
            )
            .await
            .map_err(|err| {
                AppError::InternalError(format!("routing failed: {err:?}"))
            })?;
        match parse_from_string(&route) {
            Ok((line_str, distance_m, duration_s)) => {
                (line_str, meters_to_km(distance_m), duration_s)
            }
            Err(err) => {
                return Err(AppError::InternalError(format!(
                    "route parse failed: {err}"
                )));
            }
        }
    };

    // Same cache entry the rider flow writes after its estimate — dispatch
    // and the driver's accept path read it back by request id.
    ctx.redis
        .set_key(
            &ride_path_key_id(&body.request_id),
            (
                line_str.clone(),
                distance_km,
                duration_s,
                body.from.geo_point,
                body.to.geo_point,
            ),
            ctx.config.default_ttl,
        )
        .await
        .map_err(|err| {
            AppError::InternalError(format!("Failed to set ride path: {err:?}"))
        })?;

    // Register the dispatch response channel. A duplicate request id means a
    // retried confirm slipped past the adapter's idempotency — refuse rather
    // than risk a second driver claim.
    let rider_id = body.rider.id.clone();
    let response_rx = ctx
        .dispatch_api_manager
        .register_ride_request(body.request_id.clone(), rider_id.clone())
        .map_err(|err| {
            tracing::warn!(
                tag = "beckn_ride_request",
                request_id = %body.request_id,
                error = %err,
                "Duplicate Beckn ride request rejected"
            );
            AppError::LockContention
        })?;

    // Watch this booking's dispatch channel so the adapter learns the outcome
    // (driver assigned → unsolicited on_status; no driver → on_cancel). The
    // second broadcast receiver observes the same DriverResponse events the
    // state machine consumes — the dispatch core is still only *called*.
    if ctx.config.beckn_adapter_url.is_empty() {
        tracing::warn!(
            tag = "beckn_ride_request",
            request_id = %body.request_id,
            "BECKN_ADAPTER_URL not set — the BAP will never be told the \
             assigned driver"
        );
    } else if let Some(watch_rx) = ctx
        .dispatch_api_manager
        .requests
        .get(&body.request_id)
        .map(|state| state.tx.subscribe())
    {
        tokio::spawn(watch_beckn_dispatch(
            ctx.clone(),
            body.request_id.clone(),
            watch_rx,
        ));
    }

    let ride_search_result = Some((
        line_str,
        distance_km,
        duration_s,
        body.from.geo_point,
        body.to.geo_point,
    ));

    let request = RequestDriver {
        from: body.from,
        to: body.to,
        // FOLLOW-UP: network bookings don't carry intermediate stops yet;
        // Beckn TRV10 models them as extra fulfillment stops when we need it.
        stops: Vec::new(),
        fare: body.fare,
        dx: distance_km,
        duration: duration_s as i32,
        vehicle_type: Some(vec![body.vehicle_category]),
        radius: Radius(body.radius_m),
        rider_profile: body.rider,
    };

    let dispatch_job = DispatchJob {
        request_id: body.request_id.clone(),
        rider_id,
        request,
        ride_search_result,
        response_rx,
    };
    ctx.dispatcher_queue.enqueue_dispatch(dispatch_job).await.map_err(
        |err| {
            AppError::InternalError(format!(
                "Failed to enqueue dispatch job: {err:?}"
            ))
        },
    )?;

    tracing::info!(
        tag = "beckn_ride_request",
        request_id = %body.request_id,
        "Beckn booking entered dispatch"
    );
    Ok((
        StatusCode::ACCEPTED,
        Json(BecknRideResponse {
            request_id: body.request_id,
        }),
    ))
}

/// How long we keep watching a network booking's dispatch before declaring an
/// outcome ourselves. The state machine's own deadline is 300 s (set at its
/// construction in `ride_request.rs`, plus a 15 s grace window), so at 2× that
/// the normal exit is always `Closed`, never this guard — it only fires if the
/// state machine wedged without cleanup.
const BECKN_WATCH_TIMEOUT: Duration = Duration::from_secs(600);

/// Follow one network booking through dispatch and push the terminal outcome
/// to the adapter. A raw `accepted` event is only a *hint* — the state machine
/// may still reject a stale accept — so assignment is confirmed against the
/// `ride_requests` row (status `Accepted`, same driver) before anything is
/// reported.
async fn watch_beckn_dispatch(
    ctx: Arc<APIContext>,
    request_id: String,
    mut rx: broadcast::Receiver<DispatchEvent>,
) {
    let deadline = tokio::time::Instant::now() + BECKN_WATCH_TIMEOUT;
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            // A driver hit accept; wait for the DB to agree before telling
            // the network.
            Ok(Ok(DispatchEvent::DriverResponse(response)))
                if response.accepted =>
            {
                if let Some(assigned) =
                    confirm_assignment(&ctx, &request_id, &response.driver_id)
                        .await
                {
                    push_to_adapter(&ctx, "assigned", &assigned).await;
                    return;
                }
                // Stale/rejected accept — keep watching.
            }
            Ok(Ok(DispatchEvent::CancelRequest)) => {
                // Cancelled through our own cancel endpoint; the adapter
                // already knows and answers the BAP itself.
                tracing::info!(
                    tag = "beckn_watch",
                    request_id = %request_id,
                    "network booking cancelled during dispatch"
                );
                return;
            }
            Ok(Ok(_)) => {}
            Ok(Err(broadcast::error::RecvError::Lagged(skipped))) => {
                tracing::warn!(
                    tag = "beckn_watch",
                    request_id = %request_id,
                    skipped,
                    "beckn dispatch watcher lagged"
                );
            }
            // Channel closed = the state machine finished and cleaned up;
            // timeout = it never did. Either way the DB has the last word.
            Ok(Err(broadcast::error::RecvError::Closed)) | Err(_) => {
                report_terminal_state(&ctx, &request_id).await;
                return;
            }
        }
    }
}

/// Read the terminal dispatch state from the DB and push it to the adapter.
async fn report_terminal_state(ctx: &Arc<APIContext>, request_id: &str) {
    let row = ctx
        .db
        .get_ride_request_by_id(&RideId(request_id.to_string()))
        .await
        .ok()
        .flatten();
    match row {
        Some(row)
            if matches!(
                row.request_status,
                RideRequestStatus::Accepted | RideRequestStatus::Inprogress
            ) =>
        {
            // Accepted, but the event outran us (e.g. grace-period accept
            // right before cleanup). Recover the assignment from the row.
            let driver_id = row.driver_id.clone();
            if let Some(assigned) =
                confirm_assignment(ctx, request_id, &driver_id).await
            {
                push_to_adapter(ctx, "assigned", &assigned).await;
            }
        }
        _ => {
            push_to_adapter(
                ctx,
                "failed",
                &BecknDispatchFailed {
                    request_id: request_id.to_string(),
                    reason: "no driver accepted the booking".to_string(),
                },
            )
            .await;
        }
    }
}

/// Confirm a driver's acceptance against the DB and assemble the webhook
/// payload (driver identity, vehicle, ride OTP). The accept handler writes the
/// `Accepted` status shortly after the broadcast event, hence the retry loop.
async fn confirm_assignment(
    ctx: &Arc<APIContext>,
    request_id: &str,
    driver_id: &str,
) -> Option<BecknDriverAssigned> {
    let ride_id = RideId(request_id.to_string());
    let mut row = None;
    for _ in 0..10 {
        match ctx.db.get_ride_request_by_id(&ride_id).await {
            Ok(Some(r))
                if r.driver_id == driver_id
                    && matches!(
                        r.request_status,
                        RideRequestStatus::Accepted
                            | RideRequestStatus::Inprogress
                    ) =>
            {
                row = Some(r);
                break;
            }
            Ok(_) => {}
            Err(e) => {
                tracing::error!(
                    tag = "beckn_watch",
                    request_id,
                    error = %e,
                    "ride_request lookup failed while confirming assignment"
                );
            }
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    let row = row?;

    // Best-effort enrichment: a booking with a driver but no phone/vehicle is
    // still worth announcing.
    let name = match ctx.db.get_driver_info(driver_id).await {
        Ok(Some(info)) => format!(
            "{} {}",
            info.first_name.as_deref().unwrap_or(""),
            info.last_name.as_deref().unwrap_or("")
        )
        .trim()
        .to_string(),
        _ => String::new(),
    };
    let phone = match ctx
        .db
        .get_driver_simple_profile(&DriverId(driver_id.to_string()))
        .await
    {
        Ok(Some((profile, _))) => {
            match (
                &profile.contact_data,
                &profile.nonce,
                &profile.encrypted_key,
            ) {
                (Some(data), Some(nonce), Some(key)) => {
                    extract_contact_info(data, nonce, key)
                        .await
                        .map(|(_, phone)| phone)
                        .unwrap_or_default()
                }
                _ => String::new(),
            }
        }
        _ => String::new(),
    };
    let vehicle = ctx
        .db
        .get_driver_vehicle_and_categories(&DriverId(driver_id.to_string()))
        .await
        .ok()
        .flatten();

    Some(BecknDriverAssigned {
        request_id: request_id.to_string(),
        driver_id: driver_id.to_string(),
        driver_name: name,
        driver_phone: phone,
        vehicle_plate: vehicle.as_ref().and_then(|v| v.plate_number.clone()),
        vehicle_model: vehicle.as_ref().and_then(|v| v.model.clone()),
        vehicle_color: vehicle.as_ref().and_then(|v| v.color.clone()),
        vehicle_make: vehicle.as_ref().and_then(|v| v.make.clone()),
        otp: row.otp,
    })
}

/// POST a dispatch outcome to the adapter's internal webhook, authenticated
/// with the same shared token the adapter uses towards us.
async fn push_to_adapter<T: Serialize>(
    ctx: &Arc<APIContext>,
    event: &str,
    payload: &T,
) {
    let base = ctx.config.beckn_adapter_url.trim_end_matches('/');
    if base.is_empty() {
        return;
    }
    let url = format!("{base}/internal/dispatch/{event}");
    let result = utils::http_reqwest::ReqwClient::new()
        .post(&url)
        .header("x-internal-token", &ctx.config.beckn_internal_token)
        .json(payload)
        .send()
        .await;
    match result {
        Ok(resp) if resp.status().is_success() => {
            tracing::info!(tag = "beckn_watch", url, "dispatch outcome pushed");
        }
        Ok(resp) => tracing::error!(
            tag = "beckn_watch",
            url,
            status = resp.status().as_u16(),
            "adapter rejected dispatch outcome"
        ),
        Err(e) => tracing::error!(
            tag = "beckn_watch",
            url,
            error = %e,
            "failed to push dispatch outcome to adapter"
        ),
    }
}

/// Cancel a network booking on the BAP customer's behalf. Mirrors the rider
/// app's two cancel paths: a dispatch still in flight gets the CancelRequest
/// signal; an accepted (not yet started) booking is torn down like
/// `cancel_ride`'s customer branch. A ride already in progress is refused —
/// v1 policy: mid-ride cancellation stays between rider and driver.
async fn beckn_cancel_ride_request(
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(request_id): Path<String>,
) -> Result<Json<BecknCancelResponse>, AppError> {
    let outcome = |s: &str| {
        Json(BecknCancelResponse {
            outcome: s.to_string(),
        })
    };

    // In-flight dispatch: the state machine handles teardown itself.
    if ctx
        .dispatch_api_manager
        .send_driver_response(&request_id, DispatchEvent::CancelRequest)
        .is_ok()
    {
        tracing::info!(
            tag = "beckn_cancel",
            request_id,
            "cancel signal delivered to dispatcher"
        );
        return Ok(outcome("dispatch_cancelled"));
    }

    let ride_id = RideId(request_id.clone());
    let Some(row) = ctx
        .db
        .get_ride_request_by_id(&ride_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
    else {
        return Ok(outcome("not_found"));
    };

    match row.request_status {
        RideRequestStatus::Canceled | RideRequestStatus::Completed => {
            Ok(outcome("already_closed"))
        }
        RideRequestStatus::Inprogress => Ok(outcome("active_ride")),
        _ => {
            // Same teardown as cancel_ride's customer branch: mark the
            // request canceled (persisting why), clear the ride cache, tell
            // the driver.
            let cancelled = ctx
                .db
                .cancel_ride_request(
                    &ride_id,
                    "cancelled by network customer",
                    None,
                    "customer",
                )
                .await
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            if !cancelled {
                return Ok(outcome("already_closed"));
            }
            ride_clean_up(
                &ctx.redis,
                &DriverId(row.driver_id.clone()),
                &ride_id,
                &request_id,
            )
            .await?;

            let event = RideCanceledEvent {
                event_id: ulid_string(),
                correlation_id: Some(ulid_string()),
                timestamp: Utc::now().timestamp_millis(),
                event_type: "RideCancelEvent".to_string(),
                ride_id: request_id.clone(),
                driver_id: row.driver_id,
                rider_id: row.customer_id,
                priority: 1,
                ack_required: false,
                event_payload: EventPayload {
                    ride_cancel: RideCancelPayload {
                        reason: "cancelled by network customer".to_string(),
                        canceled_by: 1,
                        refund_amount: 0.0,
                        cancellation_fee: "0".to_string(),
                        note: String::new(),
                    },
                },
            };
            if let Err(e) = EventsManger::new(RIDE_EVENTS_STREAM)
                .publish_event(Some(&event), &ctx.redis)
                .await
            {
                tracing::error!(
                    tag = "beckn_cancel",
                    request_id,
                    error = ?e,
                    "failed to publish ride canceled event"
                );
            }
            Ok(outcome("cancelled"))
        }
    }
}
