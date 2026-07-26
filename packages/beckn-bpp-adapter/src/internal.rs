//! Private internal plane of the ADAPTER: dispatch-outcome webhooks pushed by
//! the api service (Step 7).
//!
//! Decision B's second half: `on_confirm` said "confirmed, allocating"; when
//! dispatch resolves, the api service POSTs here —
//! `/internal/dispatch/assigned` (driver claimed; becomes the unsolicited
//! `on_status` RIDE_ASSIGNED) or `/internal/dispatch/failed` (no driver;
//! becomes an `on_cancel` by PROVIDER). Both routes are gated by the same
//! shared `BECKN_INTERNAL_TOKEN` the adapter uses towards the api, and both
//! must stay off the public internet — bind the adapter privately or firewall
//! `/internal/*` at the ingress.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};

use crate::AppState;
use crate::callbacks::Callback;
use crate::context::CallbackContext;
use crate::correlation::BecknOrder;
use crate::schemas::order::{
    AssignedInfo, StatusOrderArgs, build_status_order,
};

/// Constant-time byte comparison for the shared token; both length and
/// content differences cost the same.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Reject the request unless it carries the configured internal token. No
/// token configured ⇒ the plane is disabled and everything is rejected.
fn authorize(state: &AppState, headers: &HeaderMap) -> Result<(), StatusCode> {
    let Some(expected) = state.config.beckn_internal_token.as_deref() else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if expected.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let provided = headers
        .get("x-internal-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// `POST /internal/dispatch/assigned` — a driver durably accepted the
/// booking. Persist the snapshot, then push the unsolicited `on_status`
/// RIDE_ASSIGNED to the BAP.
pub async fn dispatch_assigned(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(assigned): Json<AssignedInfo>,
) -> StatusCode {
    if let Err(status) = authorize(&state, &headers) {
        return status;
    }

    let row = match state
        .correlation
        .find_confirm_by_ride_id(&assigned.request_id)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            tracing::warn!(
                request_id = %assigned.request_id,
                "assigned webhook for a ride no beckn order owns"
            );
            return StatusCode::NOT_FOUND;
        }
        Err(e) => {
            tracing::error!(
                request_id = %assigned.request_id,
                "correlation lookup failed on assigned webhook: {e:#}"
            );
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let json = match serde_json::to_string(&assigned) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("assigned payload unserializable: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    if let Err(e) = state
        .correlation
        .set_fulfillment(
            &assigned.request_id,
            "ACTIVE",
            "RIDE_ASSIGNED",
            Some(json),
        )
        .await
    {
        tracing::error!(
            request_id = %assigned.request_id,
            "failed to persist RIDE_ASSIGNED: {e:#}"
        );
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    tracing::info!(
        request_id = %assigned.request_id,
        transaction_id = %row.transaction_id,
        "driver assigned; pushing unsolicited on_status RIDE_ASSIGNED"
    );
    push_on_status(&state, &row, "ACTIVE", "RIDE_ASSIGNED", Some(&assigned))
        .await;
    StatusCode::OK
}

/// Body of `/internal/dispatch/failed` (api crate `BecknDispatchFailed`).
#[derive(Debug, serde::Deserialize)]
pub struct DispatchFailed {
    pub request_id: String,
    #[serde(default)]
    pub reason: String,
}

/// `POST /internal/dispatch/failed` — dispatch ended without a driver. The
/// booking cannot be fulfilled, so the provider cancels: `on_cancel`,
/// `cancelled_by = PROVIDER`.
pub async fn dispatch_failed(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(failed): Json<DispatchFailed>,
) -> StatusCode {
    if let Err(status) = authorize(&state, &headers) {
        return status;
    }

    let row = match state
        .correlation
        .find_confirm_by_ride_id(&failed.request_id)
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(e) => {
            tracing::error!(
                request_id = %failed.request_id,
                "correlation lookup failed on failed webhook: {e:#}"
            );
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    if let Err(e) = state
        .correlation
        .set_fulfillment(
            &failed.request_id,
            "CANCELLED",
            "RIDE_CANCELLED",
            None,
        )
        .await
    {
        tracing::error!(
            request_id = %failed.request_id,
            "failed to persist dispatch failure: {e:#}"
        );
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    tracing::warn!(
        request_id = %failed.request_id,
        reason = %failed.reason,
        "dispatch failed; pushing on_cancel (PROVIDER)"
    );
    push_on_cancel(&state, &row, "PROVIDER").await;
    StatusCode::OK
}

/// Context for an unsolicited callback about `row`'s order.
pub fn unsolicited_ctx(
    state: &AppState,
    row: &BecknOrder,
    action: &'static str,
) -> CallbackContext {
    CallbackContext::unsolicited(
        action,
        &row.domain,
        &row.bap_id,
        &row.bap_uri,
        &state.config.beckn_subscriber_id,
        &state.config.beckn_subscriber_uri,
        &row.transaction_id,
        &state.config.beckn_core_version,
    )
}

/// The lifecycle `order` for a correlation row in a given state.
pub fn status_order_for_row(
    state: &AppState,
    row: &BecknOrder,
    order_status: &str,
    state_code: &str,
    assigned: Option<&AssignedInfo>,
    cancelled_by: Option<&str>,
) -> serde_json::Value {
    let order = build_status_order(&StatusOrderArgs {
        provider_id: &state.config.beckn_subscriber_id,
        currency: &state.config.beckn_currency,
        order_id: row.beckn_order_id.as_deref().unwrap_or_default(),
        order_status,
        state_code,
        ride_id: row.ride_id.as_deref(),
        fare: row.quoted_fare,
        tier_code: row.quoted_item_id.as_deref(),
        pickup_gps: row.pickup_gps.as_deref(),
        dropoff_gps: row.dropoff_gps.as_deref(),
        assigned,
        cancelled_by,
    });
    serde_json::json!({ "order": order })
}

/// Push an unsolicited `on_status` for `row`.
pub async fn push_on_status(
    state: &AppState,
    row: &BecknOrder,
    order_status: &str,
    state_code: &str,
    assigned: Option<&AssignedInfo>,
) {
    let message = status_order_for_row(
        state,
        row,
        order_status,
        state_code,
        assigned,
        None,
    );
    let callback =
        Callback::new(unsolicited_ctx(state, row, "on_status"), message);
    if let Err(e) = state.callbacks.send(callback).await {
        tracing::error!(
            transaction_id = %row.transaction_id,
            "unsolicited on_status delivery failed: {e}"
        );
    }
}

/// Push an unsolicited `on_cancel` for `row`.
pub async fn push_on_cancel(
    state: &AppState,
    row: &BecknOrder,
    cancelled_by: &str,
) {
    let assigned = parse_assigned(row);
    let message = status_order_for_row(
        state,
        row,
        "CANCELLED",
        "RIDE_CANCELLED",
        assigned.as_ref(),
        Some(cancelled_by),
    );
    let callback =
        Callback::new(unsolicited_ctx(state, row, "on_cancel"), message);
    if let Err(e) = state.callbacks.send(callback).await {
        tracing::error!(
            transaction_id = %row.transaction_id,
            "unsolicited on_cancel delivery failed: {e}"
        );
    }
}

/// The persisted driver snapshot, when a driver was ever assigned.
pub fn parse_assigned(row: &BecknOrder) -> Option<AssignedInfo> {
    row.assigned_json
        .as_deref()
        .and_then(|json| serde_json::from_str(json).ok())
}
