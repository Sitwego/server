//! Inbound action handlers.
//!
//! Each handler does the same things, in order:
//!   1. verify the Ed25519 signature over the RAW request bytes (Step 3),
//!   2. parse the request body into `{ context, message }`,
//!   3. run the [`crate::validators`] gate for its action,
//!   4. return ACK (valid) or NACK (invalid) — and nothing else synchronously.
//!
//! The real work happens asynchronously and is delivered via the `on_*`
//! callback (Inviolable Rule #2). Step 4 wired `search` → `on_search`; Step 5
//! wires `select`/`init` → `on_select`/`on_init`. The remaining actions still
//! stop at the ACK.
//!
//! Bodies are taken as raw [`Bytes`] rather than `Json<…>` so the signature is
//! verified over the *unmodified* bytes (Inviolable Rule #7).

use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::Value;

use crate::AppState;
use crate::adapters::{
    CancelOutcome, RideDispatch, beckn_vehicle_category,
    vehicle_category_from_tier,
};
use crate::callbacks::Callback;
use crate::context::{Action, Context};
use crate::correlation::NewBecknOrder;
use crate::internal::{parse_assigned, status_order_for_row};
use crate::schemas::order::{
    ConfirmOrder, OrderSelection, build_confirm_order, build_order,
    parse_confirm_order, parse_order_id, parse_order_selection,
};
use crate::schemas::search::{SearchIntent, parse_search_intent};
use crate::schemas::{AckResponse, BecknError, on_search};
use crate::validators::{validate_inbound, verify_signature};

/// A full inbound Beckn message: the `context` plus the action-specific
/// `message` (typed per action as each step lands).
#[derive(Debug, Deserialize)]
pub struct Envelope {
    pub context: Context,
    #[serde(default)]
    pub message: Option<Value>,
}

/// Parse + validate, returning the parsed envelope on success or a NACK to send.
fn parse_and_validate(
    body: &Bytes,
    expected: Action,
    supported_version: &str,
) -> Result<Envelope, AckResponse> {
    let envelope: Envelope = serde_json::from_slice(body).map_err(|e| {
        AckResponse::nack(BecknError::context(
            "30001",
            format!("malformed request body or context: {e}"),
        ))
    })?;
    validate_inbound(&envelope.context, expected, supported_version)?;
    Ok(envelope)
}

/// The full inbound gate shared by every action handler: authenticate over the
/// raw bytes, parse, validate, and pin the context's `bap_id` to the signing
/// subscriber. Returns the response to send when any check fails.
async fn gate(
    state: &AppState,
    headers: &HeaderMap,
    body: &Bytes,
    action: Action,
) -> Result<Envelope, Response> {
    gate_inner(state, headers, body, action)
        .await
        .inspect_err(|_| {
            crate::metrics::bump(&state.metrics.inbound_rejected);
        })
        .inspect(|_| {
            crate::metrics::bump(&state.metrics.inbound_accepted);
        })
}

async fn gate_inner(
    state: &AppState,
    headers: &HeaderMap,
    body: &Bytes,
    action: Action,
) -> Result<Envelope, Response> {
    // Authenticate BEFORE parsing: Ed25519 over the raw bytes, signer's key
    // resolved from the network registry.
    let verified = if state.config.beckn_verify_signatures {
        match verify_signature(headers, body, state.registry.as_ref()).await {
            Ok(caller) => Some(caller),
            Err(rejection) => {
                return Err(
                    rejection.into_response(&state.config.beckn_subscriber_id)
                );
            }
        }
    } else {
        // Only tolerable in dev; `main` refuses to boot like this outside a
        // dev environment.
        None
    };

    let envelope =
        parse_and_validate(body, action, &state.config.beckn_core_version)
            .map_err(IntoResponse::into_response)?;

    // The signature proves who sent it; the context claims who sent it. They
    // must agree, or a valid network member could impersonate another BAP.
    if let Some(caller) = &verified
        && caller.subscriber_id != envelope.context.bap_id
    {
        return Err(AckResponse::nack(BecknError::context(
            "30001",
            "context.bap_id does not match the signing subscriber",
        ))
        .into_response());
    }

    Ok(envelope)
}

/// The three order-reference actions (`status`/`track`/`cancel`) share the
/// same inbound shape: gate, extract `message.order_id`, ACK, then run the
/// action-specific async half.
async fn order_ref_action<F, Fut>(
    state: AppState,
    headers: HeaderMap,
    body: Bytes,
    action: Action,
    process: F,
) -> Response
where
    F: FnOnce(AppState, Context, String) -> Fut,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let envelope = match gate(&state, &headers, &body, action).await {
        Ok(envelope) => envelope,
        Err(response) => return response,
    };
    let order_id = match envelope
        .message
        .as_ref()
        .ok_or_else(|| "message missing".to_string())
        .and_then(parse_order_id)
    {
        Ok(order_id) => order_id,
        Err(reason) => {
            return AckResponse::nack(BecknError::protocol(
                "30001",
                format!("invalid {}: {reason}", action.as_str()),
            ))
            .into_response();
        }
    };

    tracing::info!(
        action = action.as_str(),
        transaction_id = %envelope.context.transaction_id,
        message_id = %envelope.context.message_id,
        order_id = %order_id,
        "ACK; answering asynchronously"
    );
    tokio::spawn(process(state, envelope.context, order_id));
    AckResponse::ack().into_response()
}

/// `status` (Step 7): answer with the order's persisted lifecycle state.
pub async fn status(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    order_ref_action(state, headers, body, Action::Status, process_status).await
}

/// `track` (Step 7): answer with a tracking URL — never GPS (Rule #5).
pub async fn track(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    order_ref_action(state, headers, body, Action::Track, process_track).await
}

/// `cancel` (Step 7): cancel the booking through the api's internal plane,
/// then answer `on_cancel`.
pub async fn cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    order_ref_action(state, headers, body, Action::Cancel, process_cancel).await
}

/// `update` (Step 7): synchronously NACKed. The network update events
/// (PAYMENT_COMPLETED / EDIT_LOCATION / ADD_STOP / EDIT_STOP) don't apply to
/// Sitwego v1 — payment is cash/M-Pesa on fulfilment and mid-ride edits
/// aren't offered to network customers.
pub async fn update(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    match gate(&state, &headers, &body, Action::Update).await {
        Ok(envelope) => {
            tracing::info!(
                action = "update",
                transaction_id = %envelope.context.transaction_id,
                "NACK: update unsupported"
            );
            AckResponse::nack(BecknError::protocol(
                "30001",
                "update is not supported by this BPP",
            ))
            .into_response()
        }
        Err(response) => response,
    }
}

/// `search` (Step 4): gate → parse the intent → ACK immediately → quote and
/// deliver the catalog asynchronously as `on_search`.
pub async fn search(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let envelope = match gate(&state, &headers, &body, Action::Search).await {
        Ok(envelope) => envelope,
        Err(response) => return response,
    };

    let intent = match envelope
        .message
        .as_ref()
        .ok_or_else(|| "message missing".to_string())
        .and_then(parse_search_intent)
    {
        Ok(intent) => intent,
        Err(reason) => {
            return AckResponse::nack(BecknError::protocol(
                "30001",
                format!("invalid search intent: {reason}"),
            ))
            .into_response();
        }
    };

    tracing::info!(
        action = "search",
        transaction_id = %envelope.context.transaction_id,
        message_id = %envelope.context.message_id,
        "ACK; quoting asynchronously"
    );
    tokio::spawn(process_search(state, envelope.context, intent));
    AckResponse::ack().into_response()
}

/// The asynchronous half of `search`. Never touches the HTTP response — by the
/// time this runs, the ACK is already on the wire.
async fn process_search(state: AppState, ctx: Context, intent: SearchIntent) {
    // Idempotency (Inviolable Rule #4): first arrival of this
    // (transaction_id, message_id) wins; replays stop here, so a retried
    // search never produces a second on_search.
    let record = NewBecknOrder {
        transaction_id: ctx.transaction_id.clone(),
        message_id: ctx.message_id.clone(),
        bap_id: ctx.bap_id.clone(),
        bap_uri: ctx.bap_uri.clone(),
        domain: ctx.domain.clone(),
        last_action: "search".to_string(),
        network_id: None,
        beckn_order_id: None,
        beckn_status: None,
        ride_id: None,
        quoted_fare: None,
        quoted_item_id: None,
        fulfillment_state: None,
        pickup_gps: None,
        dropoff_gps: None,
    };
    match state.correlation.upsert_or_get(record).await {
        Ok((_, true)) => {}
        Ok((_, false)) => {
            tracing::info!(
                transaction_id = %ctx.transaction_id,
                message_id = %ctx.message_id,
                "replayed search; on_search already handled, skipping"
            );
            return;
        }
        Err(e) => {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                "correlation store failed, aborting on_search: {e:#}"
            );
            return;
        }
    }

    let quote = match state.discovery.quote(intent.pickup, intent.dropoff).await
    {
        Ok(quote) => quote,
        Err(e) => {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                "discovery quote failed, no on_search sent: {e:#}"
            );
            return;
        }
    };
    if quote.options.is_empty() {
        tracing::info!(
            transaction_id = %ctx.transaction_id,
            "no drivers available; sending empty catalog"
        );
    }

    let catalog = on_search::build_catalog(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_provider_name,
        &state.config.beckn_currency,
        &quote,
        intent.pickup,
        intent.dropoff,
    );
    let callback_ctx = ctx.to_callback(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_subscriber_uri,
    );
    let callback =
        Callback::new(callback_ctx, serde_json::json!({ "catalog": catalog }));
    if let Err(e) = state.callbacks.send(callback).await {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            "on_search delivery to BAP failed: {e}"
        );
    }
}

/// The two quote-phase order actions share one pipeline; the phase decides the
/// recorded `last_action` and whether a durable order id is minted (`init`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderPhase {
    Select,
    Init,
}

impl OrderPhase {
    fn action_name(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Init => "init",
        }
    }
}

/// `select` (Step 5): gate → parse the chosen item + trip → ACK → re-price and
/// answer asynchronously as `on_select` with the quote breakup + payment terms.
pub async fn select(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    order_action(state, headers, body, Action::Select, OrderPhase::Select).await
}

/// `init` (Step 5): same pipeline as `select`, but the `on_init` order carries
/// the durable Beckn order id that `confirm` (Step 6) will act on.
pub async fn init(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    order_action(state, headers, body, Action::Init, OrderPhase::Init).await
}

async fn order_action(
    state: AppState,
    headers: HeaderMap,
    body: Bytes,
    action: Action,
    phase: OrderPhase,
) -> Response {
    let envelope = match gate(&state, &headers, &body, action).await {
        Ok(envelope) => envelope,
        Err(response) => return response,
    };

    let selection = match envelope
        .message
        .as_ref()
        .ok_or_else(|| "message missing".to_string())
        .and_then(parse_order_selection)
    {
        Ok(selection) => selection,
        Err(reason) => {
            return AckResponse::nack(BecknError::protocol(
                "30001",
                format!("invalid {} order: {reason}", phase.action_name()),
            ))
            .into_response();
        }
    };

    tracing::info!(
        action = phase.action_name(),
        transaction_id = %envelope.context.transaction_id,
        message_id = %envelope.context.message_id,
        item_id = %selection.item_id,
        "ACK; quoting asynchronously"
    );
    tokio::spawn(process_order(state, envelope.context, selection, phase));
    AckResponse::ack().into_response()
}

/// The asynchronous half of `select`/`init`: idempotency-guard, re-price the
/// selected tier, and deliver `on_select`/`on_init` — or an error callback if
/// the tier is no longer serviceable.
async fn process_order(
    state: AppState,
    ctx: Context,
    selection: OrderSelection,
    phase: OrderPhase,
) {
    // `init` mints the durable order id up front so it lives on the same
    // correlation row that wins the idempotency race (Rule #4): a retried
    // init can never mint a second order id.
    let beckn_order_id = match phase {
        OrderPhase::Init => Some(utils::gen_strings::ulid_string()),
        OrderPhase::Select => None,
    };
    let record = NewBecknOrder {
        transaction_id: ctx.transaction_id.clone(),
        message_id: ctx.message_id.clone(),
        bap_id: ctx.bap_id.clone(),
        bap_uri: ctx.bap_uri.clone(),
        domain: ctx.domain.clone(),
        last_action: phase.action_name().to_string(),
        network_id: None,
        beckn_order_id,
        beckn_status: None,
        ride_id: None,
        quoted_fare: None,
        quoted_item_id: None,
        fulfillment_state: None,
        pickup_gps: None,
        dropoff_gps: None,
    };
    let row = match state.correlation.upsert_or_get(record).await {
        Ok((row, true)) => row,
        Ok((_, false)) => {
            tracing::info!(
                transaction_id = %ctx.transaction_id,
                message_id = %ctx.message_id,
                action = phase.action_name(),
                "replayed message; on_{} already handled, skipping",
                phase.action_name()
            );
            return;
        }
        Err(e) => {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                "correlation store failed, aborting on_{}: {e:#}",
                phase.action_name()
            );
            return;
        }
    };

    let callback_ctx = ctx.to_callback(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_subscriber_uri,
    );

    let quote = match state
        .discovery
        .quote(selection.pickup, selection.dropoff)
        .await
    {
        Ok(quote) => quote,
        Err(e) => {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                "discovery quote failed, no on_{} sent: {e:#}",
                phase.action_name()
            );
            return;
        }
    };

    // The item id is a tier code from our own on_search catalog. If that tier
    // has no drivers around anymore (or the id is foreign), the failure is
    // discovered after the ACK — so it travels as `{context, error}` on the
    // callback, the async counterpart of a NACK.
    let Some(option) = quote
        .options
        .iter()
        .find(|option| option.tier_code == selection.item_id)
    else {
        tracing::warn!(
            transaction_id = %ctx.transaction_id,
            item_id = %selection.item_id,
            "selected item not serviceable; sending error callback"
        );
        let error = BecknError::protocol(
            "30001",
            format!("item `{}` is not available", selection.item_id),
        );
        if let Err(e) =
            state.callbacks.send(Callback::error(callback_ctx, &error)).await
        {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                "error callback delivery to BAP failed: {e}"
            );
        }
        return;
    };

    // `init` persists the quote it is about to promise, so `confirm` can
    // honour it later without re-pricing (the customer already agreed to it).
    if phase == OrderPhase::Init
        && let Err(e) = state
            .correlation
            .set_quote(
                &ctx.transaction_id,
                &ctx.message_id,
                option.fare.round() as i32,
                &option.tier_code,
            )
            .await
    {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            "failed to persist init quote, aborting on_init: {e:#}"
        );
        return;
    }

    let order = build_order(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_currency,
        option,
        selection.pickup,
        selection.dropoff,
        row.beckn_order_id,
    );
    let callback =
        Callback::new(callback_ctx, serde_json::json!({ "order": order }));
    if let Err(e) = state.callbacks.send(callback).await {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            "on_{} delivery to BAP failed: {e}",
            phase.action_name()
        );
    }
}

/// `confirm` (Step 6): gate → parse the committed order → ACK → asynchronously
/// hand the booking to dispatch and reply `on_confirm` ("confirmed,
/// allocating" — decision B). The assigned driver follows later as an
/// unsolicited `on_status` (Step 7).
pub async fn confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let envelope = match gate(&state, &headers, &body, Action::Confirm).await {
        Ok(envelope) => envelope,
        Err(response) => return response,
    };

    let confirm_order = match envelope
        .message
        .as_ref()
        .ok_or_else(|| "message missing".to_string())
        .and_then(parse_confirm_order)
    {
        Ok(confirm_order) => confirm_order,
        Err(reason) => {
            return AckResponse::nack(BecknError::protocol(
                "30001",
                format!("invalid confirm order: {reason}"),
            ))
            .into_response();
        }
    };

    tracing::info!(
        action = "confirm",
        transaction_id = %envelope.context.transaction_id,
        message_id = %envelope.context.message_id,
        order_id = %confirm_order.order_id,
        "ACK; dispatching asynchronously"
    );
    tokio::spawn(process_confirm(state, envelope.context, confirm_order));
    AckResponse::ack().into_response()
}

/// The asynchronous half of `confirm`. Rule #4 is CRITICAL here: the ride id
/// is minted inside the idempotency insert, and dispatch is only called when
/// this process wins that insert — a retried confirm can never claim a second
/// driver.
async fn process_confirm(state: AppState, ctx: Context, confirm: ConfirmOrder) {
    let ride_id = utils::gen_strings::ulid_string();
    let record = NewBecknOrder {
        transaction_id: ctx.transaction_id.clone(),
        message_id: ctx.message_id.clone(),
        bap_id: ctx.bap_id.clone(),
        bap_uri: ctx.bap_uri.clone(),
        domain: ctx.domain.clone(),
        last_action: "confirm".to_string(),
        network_id: None,
        beckn_order_id: Some(confirm.order_id.clone()),
        beckn_status: Some("ACTIVE".to_string()),
        ride_id: Some(ride_id),
        quoted_fare: None,
        quoted_item_id: None,
        // Decision B: allocating. The trip is persisted so later on_status
        // orders can repeat the stops.
        fulfillment_state: Some("NEW".to_string()),
        pickup_gps: Some(on_search::gps_text(confirm.pickup)),
        dropoff_gps: Some(on_search::gps_text(confirm.dropoff)),
    };
    let row = match state.correlation.upsert_or_get(record).await {
        Ok((row, true)) => row,
        Ok((_, false)) => {
            tracing::info!(
                transaction_id = %ctx.transaction_id,
                message_id = %ctx.message_id,
                "replayed confirm; dispatch already handled, skipping"
            );
            return;
        }
        Err(e) => {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                "correlation store failed, aborting on_confirm: {e:#}"
            );
            return;
        }
    };

    let callback_ctx = ctx.to_callback(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_subscriber_uri,
    );
    let send_error = |error: BecknError| {
        let callbacks = state.callbacks.clone();
        let callback = Callback::error(callback_ctx.clone(), &error);
        let transaction_id = ctx.transaction_id.clone();
        async move {
            if let Err(e) = callbacks.send(callback).await {
                tracing::error!(
                    transaction_id = %transaction_id,
                    "error callback delivery to BAP failed: {e}"
                );
            }
        }
    };

    // The order must be one WE minted at init, in THIS transaction, with the
    // quote persisted — otherwise there is nothing to honour.
    let init_row = match state
        .correlation
        .find_init_by_order_id(&confirm.order_id)
        .await
    {
        Ok(Some(init_row)) if init_row.transaction_id == ctx.transaction_id => {
            init_row
        }
        Ok(_) => {
            tracing::warn!(
                transaction_id = %ctx.transaction_id,
                order_id = %confirm.order_id,
                "confirm references an unknown or foreign order id"
            );
            send_error(BecknError::protocol(
                "30001",
                format!("unknown order id `{}`", confirm.order_id),
            ))
            .await;
            return;
        }
        Err(e) => {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                "correlation lookup failed, no on_confirm sent: {e:#}"
            );
            return;
        }
    };
    // Copy the honoured quote onto the confirm row: `status`/`track`/`cancel`
    // reference the order by id and read fare/tier from this row.
    if let (Some(fare), Some(tier)) =
        (init_row.quoted_fare, init_row.quoted_item_id.as_deref())
        && let Err(e) = state
            .correlation
            .set_quote(&ctx.transaction_id, &ctx.message_id, fare, tier)
            .await
    {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            "failed to copy quote onto confirm row: {e:#}"
        );
    }

    let (Some(fare), Some(tier_code)) =
        (init_row.quoted_fare, init_row.quoted_item_id.clone())
    else {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            order_id = %confirm.order_id,
            "init row has no persisted quote; cannot honour confirm"
        );
        send_error(BecknError::protocol(
            "30001",
            format!("no quote on record for order `{}`", confirm.order_id),
        ))
        .await;
        return;
    };

    // Hand the booking to the dispatch core (via the api service's private
    // internal plane). The ride id doubles as the dispatch request id, so
    // even a duplicate slipping through is refused on the api side.
    let dispatch = RideDispatch {
        request_id: row.ride_id.clone().unwrap_or_default(),
        customer_name: confirm
            .customer_name
            .clone()
            .unwrap_or_else(|| "Network rider".to_string()),
        customer_phone: confirm.customer_phone.clone().unwrap_or_default(),
        pickup: confirm.pickup,
        dropoff: confirm.dropoff,
        fare,
        tier_code: tier_code.clone(),
    };
    if let Err(e) = state.dispatch.request_ride(dispatch).await {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            order_id = %confirm.order_id,
            "dispatch handoff failed: {e:#}"
        );
        send_error(BecknError::protocol(
            "30001",
            "could not enter the booking into dispatch",
        ))
        .await;
        return;
    }

    // "Confirmed, allocating" goes out immediately (decision B); the driver
    // follows via on_status once dispatch claims one.
    let vehicle_category = vehicle_category_from_tier(&tier_code);
    let order = build_confirm_order(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_currency,
        &confirm.order_id,
        &tier_code,
        &vehicle_category
            .map(|c| c.to_string())
            .unwrap_or_else(|| tier_code.clone()),
        vehicle_category.as_ref().map(beckn_vehicle_category).unwrap_or("CAB"),
        fare,
        confirm.pickup,
        confirm.dropoff,
    );
    let callback =
        Callback::new(callback_ctx, serde_json::json!({ "order": order }));
    if let Err(e) = state.callbacks.send(callback).await {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            "on_confirm delivery to BAP failed: {e}"
        );
    }
}

/// Send the async error `{context, error}` for an order-reference action.
async fn send_ref_error(
    state: &AppState,
    ctx: &Context,
    code: &str,
    message: String,
) {
    let callback_ctx = ctx.to_callback(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_subscriber_uri,
    );
    let error = BecknError::protocol(code, message);
    if let Err(e) =
        state.callbacks.send(Callback::error(callback_ctx, &error)).await
    {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            "error callback delivery to BAP failed: {e}"
        );
    }
}

/// Shared async prologue of `status`/`track`/`cancel`: idempotency-guard the
/// message, then resolve the referenced order to its `confirm` row. The row
/// must belong to the calling BAP *and* this transaction — an order id is not
/// a capability another subscriber can replay.
async fn guard_and_find_order(
    state: &AppState,
    ctx: &Context,
    order_id: &str,
    action: &'static str,
) -> Option<crate::correlation::BecknOrder> {
    let record = NewBecknOrder {
        transaction_id: ctx.transaction_id.clone(),
        message_id: ctx.message_id.clone(),
        bap_id: ctx.bap_id.clone(),
        bap_uri: ctx.bap_uri.clone(),
        domain: ctx.domain.clone(),
        last_action: action.to_string(),
        network_id: None,
        beckn_order_id: Some(order_id.to_string()),
        beckn_status: None,
        ride_id: None,
        quoted_fare: None,
        quoted_item_id: None,
        fulfillment_state: None,
        pickup_gps: None,
        dropoff_gps: None,
    };
    match state.correlation.upsert_or_get(record).await {
        Ok((_, true)) => {}
        Ok((_, false)) => {
            tracing::info!(
                transaction_id = %ctx.transaction_id,
                message_id = %ctx.message_id,
                action,
                "replayed {action}; already handled, skipping"
            );
            return None;
        }
        Err(e) => {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                "correlation store failed, aborting on_{action}: {e:#}"
            );
            return None;
        }
    }

    match state.correlation.find_confirm_by_order_id(order_id).await {
        Ok(Some(row))
            if row.transaction_id == ctx.transaction_id
                && row.bap_id == ctx.bap_id =>
        {
            Some(row)
        }
        Ok(_) => {
            tracing::warn!(
                transaction_id = %ctx.transaction_id,
                order_id,
                "{action} references an unknown or foreign order id"
            );
            send_ref_error(
                state,
                ctx,
                "30001",
                format!("unknown order id `{order_id}`"),
            )
            .await;
            None
        }
        Err(e) => {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                "correlation lookup failed, no on_{action} sent: {e:#}"
            );
            None
        }
    }
}

/// The asynchronous half of `status`: replay the order's persisted lifecycle
/// state — no fresh pricing, no dispatch calls.
async fn process_status(state: AppState, ctx: Context, order_id: String) {
    let Some(row) =
        guard_and_find_order(&state, &ctx, &order_id, "status").await
    else {
        return;
    };

    let assigned = parse_assigned(&row);
    let order_status =
        row.beckn_status.clone().unwrap_or_else(|| "ACTIVE".into());
    let state_code =
        row.fulfillment_state.clone().unwrap_or_else(|| "NEW".into());
    let cancelled_by = (order_status == "CANCELLED").then_some("PROVIDER");

    let message = status_order_for_row(
        &state,
        &row,
        &order_status,
        &state_code,
        assigned.as_ref(),
        cancelled_by,
    );
    let callback_ctx = ctx.to_callback(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_subscriber_uri,
    );
    if let Err(e) =
        state.callbacks.send(Callback::new(callback_ctx, message)).await
    {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            "on_status delivery to BAP failed: {e}"
        );
    }
}

/// The asynchronous half of `track`: a tracking URL from the configured
/// template — never coordinates (Inviolable Rule #5).
async fn process_track(state: AppState, ctx: Context, order_id: String) {
    let Some(row) =
        guard_and_find_order(&state, &ctx, &order_id, "track").await
    else {
        return;
    };

    let Some(template) = state.config.beckn_tracking_url_template.clone()
    else {
        send_ref_error(
            &state,
            &ctx,
            "30001",
            "tracking is not available for this order".to_string(),
        )
        .await;
        return;
    };
    let url =
        template.replace("{ride_id}", row.ride_id.as_deref().unwrap_or(""));
    // A live URL while the ride can still move; expired once it can't.
    let tracking_status = match row.fulfillment_state.as_deref() {
        Some("RIDE_ENDED") | Some("RIDE_CANCELLED") => "INACTIVE",
        _ => "ACTIVE",
    };

    let callback_ctx = ctx.to_callback(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_subscriber_uri,
    );
    let message = serde_json::json!({
        "tracking": { "url": url, "status": tracking_status }
    });
    if let Err(e) =
        state.callbacks.send(Callback::new(callback_ctx, message)).await
    {
        tracing::error!(
            transaction_id = %ctx.transaction_id,
            "on_track delivery to BAP failed: {e}"
        );
    }
}

/// The asynchronous half of `cancel`: tear the booking down through the api's
/// internal plane, persist RIDE_CANCELLED, and reply `on_cancel`.
async fn process_cancel(state: AppState, ctx: Context, order_id: String) {
    let Some(row) =
        guard_and_find_order(&state, &ctx, &order_id, "cancel").await
    else {
        return;
    };

    let callback_ctx = ctx.to_callback(
        &state.config.beckn_subscriber_id,
        &state.config.beckn_subscriber_uri,
    );
    let send_cancelled = |cancelled_by: &'static str| {
        let state = state.clone();
        let row = row.clone();
        let callback_ctx = callback_ctx.clone();
        let transaction_id = ctx.transaction_id.clone();
        async move {
            let assigned = parse_assigned(&row);
            let message = status_order_for_row(
                &state,
                &row,
                "CANCELLED",
                "RIDE_CANCELLED",
                assigned.as_ref(),
                Some(cancelled_by),
            );
            if let Err(e) =
                state.callbacks.send(Callback::new(callback_ctx, message)).await
            {
                tracing::error!(
                    transaction_id = %transaction_id,
                    "on_cancel delivery to BAP failed: {e}"
                );
            }
        }
    };

    match row.fulfillment_state.as_deref() {
        // Cancelling a cancelled order is answerable — repeat the outcome.
        Some("RIDE_CANCELLED") => {
            send_cancelled("CONSUMER").await;
            return;
        }
        Some("RIDE_ENDED") => {
            send_ref_error(
                &state,
                &ctx,
                "30001",
                format!("order `{order_id}` is already completed"),
            )
            .await;
            return;
        }
        _ => {}
    }

    let ride_id = row.ride_id.clone().unwrap_or_default();
    match state.dispatch.cancel_ride(&ride_id).await {
        Ok(
            CancelOutcome::DispatchCancelled
            | CancelOutcome::Cancelled
            | CancelOutcome::AlreadyClosed
            // No trace in dispatch = the booking never landed or is long
            // gone; the order itself is still cancellable.
            | CancelOutcome::NotFound,
        ) => {
            if let Err(e) = state
                .correlation
                .set_fulfillment(&ride_id, "CANCELLED", "RIDE_CANCELLED", None)
                .await
            {
                tracing::error!(
                    transaction_id = %ctx.transaction_id,
                    "failed to persist RIDE_CANCELLED: {e:#}"
                );
            }
            send_cancelled("CONSUMER").await;
        }
        Ok(CancelOutcome::ActiveRide) => {
            // v1 policy: once the trip is underway, cancellation stays
            // between rider and driver.
            send_ref_error(
                &state,
                &ctx,
                "30001",
                format!("order `{order_id}` is already in progress"),
            )
            .await;
        }
        Err(e) => {
            tracing::error!(
                transaction_id = %ctx.transaction_id,
                order_id,
                "cancel handoff failed: {e:#}"
            );
            send_ref_error(
                &state,
                &ctx,
                "30001",
                "could not cancel the booking".to_string(),
            )
            .await;
        }
    }
}
