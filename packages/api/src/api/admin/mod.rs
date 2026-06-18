//! Private admin plane.
//!
//! These routes are served on a SEPARATE listener bound to a private interface
//! (see `main.rs` + `Config::admin_bind_addr`), so they are never reachable
//! from the public internet. Every request must carry the shared
//! `X-Internal-Token` (sent by the admin BFF); authn/authz of the human admin
//! and the audit log live in the BFF, so handlers here are thin delegations to
//! existing business logic — they never reimplement billing rules.

use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use redis_store::r_types::AppError;
use sea_orm::Iterable;
use sea_orm::prelude::{DateTimeWithTimeZone, Decimal};
use serde::{Deserialize, Serialize};
use utils::hashing_algo::extract_contact_info;

use crate::APIContext;
use crate::queries::admin::docs::{AdminDocuments, DocKind};
use crate::queries::admin::drivers::AdminDrivers;
use crate::queries::bussines::SubscriptionsPlans;
use crate::queries::drivers::DriverQueries;
use crate::queries::referral::{ReferralQueries, RewardOutcome};
use crate::schemas::docs::DocumentReviewStatus;
use crate::schemas::referral_rewards::ReferralRewardType;
use crate::types::{DriverId, VehicleCategory};

/// Gate the admin plane on the shared internal token. This is transport-level
/// trust between the BFF and the core — the BFF has already authenticated the
/// human admin and checked their permissions before calling us.
pub async fn admin_internal_auth(
    State(ctx): State<Arc<APIContext>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected = ctx.config.admin_internal_token.as_bytes();
    let provided = req
        .headers()
        .get("x-internal-token")
        .and_then(|v| v.to_str().ok())
        .map(str::as_bytes);

    match provided {
        Some(tok) if constant_time_eq(tok, expected) => Ok(next.run(req).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Length-checked constant-time comparison so token validation doesn't leak
/// length/prefix information through timing.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn admin_handlers(ctx: Arc<APIContext>) -> Router {
    Router::new()
        .route("/admin/drivers/{driver_id}/suspend", post(suspend_driver))
        .route(
            "/admin/drivers/{driver_id}/reactivate",
            post(reactivate_driver),
        )
        .route(
            "/admin/subscriptions/{sub_id}/reset",
            post(reset_subscription),
        )
        .route("/admin/subscriptions/{sub_id}/adjust-due", post(adjust_due))
        .route("/admin/billing/run", post(run_billing))
        // --- driver directory ---
        .route("/admin/drivers", get(list_drivers))
        .route("/admin/drivers/{driver_id}/profile", get(driver_profile))
        .route("/admin/drivers/{driver_id}/photo", get(driver_photo))
        .route(
            "/admin/drivers/{driver_id}/photo/review",
            post(review_driver_photo),
        )
        // --- vehicle info + category eligibility ---
        .route("/admin/vehicle-categories", get(list_vehicle_categories))
        .route("/admin/drivers/{driver_id}/vehicle", get(driver_vehicle))
        .route(
            "/admin/drivers/{driver_id}/categories",
            put(set_qualifying_categories),
        )
        // --- driver onboarding document verification ---
        .route("/admin/plans", get(list_plans))
        .route("/admin/drivers/{driver_id}/documents", get(list_documents))
        .route(
            "/admin/drivers/{driver_id}/readiness",
            get(driver_readiness),
        )
        .route("/admin/drivers/{driver_id}/activate", post(activate_driver))
        .route(
            "/admin/documents/{kind}/{doc_id}/review",
            post(review_document),
        )
        .route(
            "/admin/documents/{kind}/{doc_id}/image",
            get(document_image),
        )
        .layer(middleware::from_fn_with_state(
            ctx.clone(),
            admin_internal_auth,
        ))
        .layer(Extension(ctx))
}

/// Suspend a driver's billing (stops further accrual).
async fn suspend_driver(
    Path(driver_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .set_plan_active(&driver_id, false)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

/// Reactivate a previously suspended driver (resumes accrual from the watermark).
async fn reactivate_driver(
    Path(driver_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .set_plan_active(&driver_id, true)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
struct ResetReq {
    driver_id: String,
}

/// Reset a subscription after settlement (zeroes amount_due, rolls the window,
/// advances both last_billed_at and last_accrued_at). Same path the payment
/// callback uses.
async fn reset_subscription(
    Path(sub_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<ResetReq>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .reset_subscription(&body.driver_id, &sub_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
struct AdjustDueReq {
    amount_due: Decimal,
}

/// Manually set a subscription's outstanding balance (credit/correction).
async fn adjust_due(
    Path(sub_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<AdjustDueReq>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .admin_adjust_amount_due(&sub_id, body.amount_due)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

/// Trigger the daily accrual job on demand (same logic as the cron run).
async fn run_billing(
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .update_due_amount()
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// Driver directory
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct DriverListQuery {
    limit: Option<u64>,
    offset: Option<u64>,
}

/// List drivers (newest first) for the admin directory. Cheap fields only —
/// name + status + photo ref; contact/photo decryption happens per-driver.
async fn list_drivers(
    Query(q): Query<DriverListQuery>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let limit = q.limit.unwrap_or(100).min(500);
    let offset = q.offset.unwrap_or(0);
    let drivers = ctx
        .db
        .list_drivers(limit, offset)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(drivers).into_response())
}

#[derive(Debug, Serialize)]
struct DriverProfileResponse {
    id: String,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    photo_id: Option<String>,
    activated: Option<bool>,
    has_onboarded: Option<bool>,
    created_at: DateTimeWithTimeZone,
}

/// One driver's detail header — name, decrypted email/phone, status. Contact is
/// envelope-decrypted here on the private plane only, never on a public route.
async fn driver_profile(
    Path(driver_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let row = ctx
        .db
        .get_driver_detail(&driver_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("driver not found".into()))?;

    // Decrypt contact only when the envelope material is present. Accounts that
    // predate key storage have an empty key blob and can't be recovered — we
    // return the rest of the profile without contact rather than erroring.
    let (email, phone) = match (row.contact_data, row.nonce, row.encrypted_key)
    {
        (Some(cd), Some(nonce), Some(key)) if !key.is_empty() => {
            match extract_contact_info(&cd, &nonce, &key).await {
                Ok((email, phone)) => (Some(email), Some(phone)),
                Err(e) => {
                    tracing::warn!(
                        "admin: failed to decrypt contact for {driver_id}: {e:?}"
                    );
                    (None, None)
                }
            }
        }
        _ => (None, None),
    };

    Ok(Json(DriverProfileResponse {
        id: row.id,
        first_name: row.first_name,
        last_name: row.last_name,
        email,
        phone,
        photo_id: row.photo_id,
        activated: row.activated,
        has_onboarded: row.has_onboarded,
        created_at: row.created_at,
    })
    .into_response())
}

/// Stream a driver's decrypted profile photo. Decryption stays on the private
/// plane (same envelope scheme as the document images).
async fn driver_photo(
    Path(driver_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let blob = ctx
        .db
        .get_driver_photo_ref(&driver_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("driver photo not found".into()))?;

    let bucket = &ctx.config.bucket;
    let path_id = format!("driver-docs/{}/{}", driver_id, blob.photo_id);
    let data = ctx
        .config
        .aws_credentials()
        .get_uploaded_file_from_s3(
            &path_id,
            bucket,
            &blob.encrypted_key,
            &blob.nonce,
        )
        .await?;

    Ok((
        [
            (header::CONTENT_TYPE, "image/jpeg".to_string()),
            (header::CACHE_CONTROL, "private, no-store".to_owned()),
        ],
        data,
    )
        .into_response())
}

/// Approve or reject a driver's profile photo — reviewed like a document, but
/// keyed by driver since the photo has no document row.
async fn review_driver_photo(
    Path(driver_id): Path<String>,
    headers: HeaderMap,
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<ReviewReq>,
) -> Result<StatusCode, AppError> {
    let reviewed_by = acting_admin(&headers)?;

    let status = if body.approve {
        DocumentReviewStatus::Approved
    } else {
        if body.reason.as_deref().unwrap_or("").trim().is_empty() {
            return Err(AppError::ValidationError(
                "a reason is required when rejecting a photo".into(),
            ));
        }
        DocumentReviewStatus::Rejected
    };

    ctx.db
        .review_driver_photo(
            &driver_id,
            status,
            &reviewed_by,
            body.reason.as_deref(),
        )
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// Driver onboarding document verification
// ---------------------------------------------------------------------------

/// Parse the `{kind}` path segment into a [`DocKind`].
fn parse_kind(kind: &str) -> Result<DocKind, AppError> {
    match kind.to_ascii_lowercase().as_str() {
        "document" | "documents" => Ok(DocKind::Document),
        "identity" => Ok(DocKind::Identity),
        other => Err(AppError::ValidationError(format!(
            "unknown document kind '{other}'"
        ))),
    }
}

/// Read the acting admin's id from the `x-acting-admin` header the BFF sets.
/// The BFF has already authenticated the human; we only record who acted.
fn acting_admin(headers: &HeaderMap) -> Result<String, AppError> {
    headers
        .get("x-acting-admin")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            AppError::ValidationError("missing x-acting-admin header".into())
        })
}

/// List a driver's active documents with their current review state.
async fn list_documents(
    Path(driver_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let docs = ctx
        .db
        .list_driver_documents(&driver_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(docs).into_response())
}

#[derive(Debug, Deserialize)]
struct ReviewReq {
    /// `true` approves the document, `false` rejects it.
    approve: bool,
    /// Required on rejection — shown to the driver so they can re-upload.
    reason: Option<String>,
}

/// Approve or reject one uploaded document.
async fn review_document(
    Path((kind, doc_id)): Path<(String, i64)>,
    headers: HeaderMap,
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<ReviewReq>,
) -> Result<StatusCode, AppError> {
    let kind = parse_kind(&kind)?;
    let reviewed_by = acting_admin(&headers)?;

    let status = if body.approve {
        DocumentReviewStatus::Approved
    } else {
        if body.reason.as_deref().unwrap_or("").trim().is_empty() {
            return Err(AppError::ValidationError(
                "a reason is required when rejecting a document".into(),
            ));
        }
        DocumentReviewStatus::Rejected
    };

    ctx.db
        .review_document(
            kind,
            doc_id,
            status,
            &reviewed_by,
            body.reason.as_deref(),
        )
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

/// All billing plans, for the admin to choose from when activating a driver.
async fn list_plans(
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let plans = ctx
        .db
        .list_plans()
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(plans).into_response())
}

#[derive(Debug, Serialize)]
struct ReadinessResponse {
    /// `true` when `blockers` is empty — the driver is clear to go live.
    ready: bool,
    /// Human-readable reasons activation is still blocked (empty when ready).
    blockers: Vec<String>,
}

/// Report whether a driver can be activated yet, and if not, why. Lets the admin
/// UI render a readiness checklist before attempting the (hard-guarded) activate.
async fn driver_readiness(
    Path(driver_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let blockers = ctx
        .db
        .driver_activation_blockers(&driver_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(ReadinessResponse {
        ready: blockers.is_empty(),
        blockers,
    })
    .into_response())
}

#[derive(Debug, Deserialize)]
struct ActivateReq {
    activate: bool,
    /// Required when activating: the billing plan the driver's free trial starts
    /// on (the driver does not pick a plan during onboarding). Ignored on revoke.
    #[serde(default)]
    plan_id: Option<String>,
}

/// Confirm (or revoke) a driver's go-live activation. Going live is hard-guarded
/// server-side: every required document + the photo must be APPROVED and at
/// least one vehicle category assigned (see `driver_activation_blockers`).
/// Revoking activation is never blocked — an admin can always pull a driver.
async fn activate_driver(
    Path(driver_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<ActivateReq>,
) -> Result<StatusCode, AppError> {
    if body.activate {
        let blockers = ctx
            .db
            .driver_activation_blockers(&driver_id)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        if !blockers.is_empty() {
            return Err(AppError::ValidationError(format!(
                "Cannot activate driver: {}",
                blockers.join("; ")
            )));
        }

        // Going live starts the driver's 14-day free trial on the plan the admin
        // selected (the driver picks no plan during onboarding, so it must be
        // supplied here).
        let plan_id =
            body.plan_id.as_deref().filter(|s| !s.is_empty()).ok_or_else(
                || {
                    AppError::ValidationError(
                        "Cannot activate driver: a subscription plan must be \
                     selected"
                            .to_string(),
                    )
                },
            )?;
        ctx.db
            .start_driver_trial(&driver_id, plan_id, 14)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;
    }

    ctx.db
        .activate_driver(&driver_id, body.activate)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    // Going live is the "KYC approved" event for the referral programme: if
    // this driver was referred, reward the referrer now. Idempotent (a
    // re-activation never double-rewards) and never fails the activation —
    // reward problems are logged and recoverable, a blocked go-live is not.
    if body.activate {
        let reward_type =
            ReferralRewardType::from_config(&ctx.config.referral_reward_type);
        let reward_value = Decimal::try_from(ctx.config.referral_reward_value)
            .unwrap_or_default();
        match ctx
            .db
            .reward_referral_on_activation(
                &driver_id,
                reward_type,
                reward_value,
            )
            .await
        {
            Ok(Some(outcome)) => notify_referrer(&ctx, outcome),
            Ok(None) => {}
            Err(e) => tracing::error!(
                driver_id,
                "failed to issue referral reward on activation: {e:?}"
            ),
        }
    }

    Ok(StatusCode::OK)
}

/// Push the "you earned a reward" notification to the referrer. Fired only on
/// a FRESH reward issuance; runs in the background and never blocks or fails
/// the activation request.
fn notify_referrer(ctx: &Arc<APIContext>, outcome: RewardOutcome) {
    let RewardOutcome {
        referrer_id,
        referred_name,
        reward_type,
        reward_value,
    } = outcome;

    let message = match reward_type {
        ReferralRewardType::CashCredit => format!(
            "{referred_name} just went live. KES {reward_value} has been \
             added to your wallet."
        ),
        ReferralRewardType::SubscriptionDays => format!(
            "{referred_name} just went live. You earned {reward_value} free \
             subscription days."
        ),
        ReferralRewardType::Badge => {
            format!("{referred_name} just went live. You earned a new badge!")
        }
    };

    crate::notif::spawn_notify(
        ctx.db.clone(),
        ctx.notif.clone(),
        referrer_id.clone(),
        move |b| {
            b.title("You earned a referral reward! 🎉")
                .message(message)
                .android_channel("referral-reward")
                .android_color("#ed1380")
                .android_tag(format!("referral-reward-{referrer_id}"))
                .click_action("OPEN_REFERRALS")
        },
    );
}

/// One selectable vehicle category for the dashboard's "Qualifying categories"
/// picker.
#[derive(Debug, Serialize)]
struct CategoryOption {
    /// The value to send back to `PUT .../categories` — the canonical enum
    /// value (e.g. `"Auto"`), so the picker can never drift from what the core
    /// accepts.
    value: VehicleCategory,
    /// Human-readable label for the checkbox.
    label: String,
    /// Exclusive categories (Bike, Women, Auto) serve only their own request
    /// type and cannot be combined with the tiered set (Swift–Executive).
    exclusive: bool,
}

/// The canonical set of vehicle categories the admin can assign, sourced from
/// the `VehicleCategory` enum (not the DB) so the list always matches the
/// values the core accepts and automatically includes new categories like
/// `Auto`. Ordered as declared: the tiered set first, then the exclusive ones.
async fn list_vehicle_categories() -> Result<Response, AppError> {
    let categories: Vec<CategoryOption> = VehicleCategory::iter()
        .map(|c| CategoryOption {
            label: c.to_string(),
            // A category is exclusive when it serves only itself. Swift also
            // serves only itself but is the base tier of the shared chain, so
            // it is explicitly excluded.
            exclusive: c != VehicleCategory::Swift
                && c.eligible_serving_categories() == vec![c],
            value: c,
        })
        .collect();

    Ok(Json(categories).into_response())
}

/// The driver's vehicle info + the categories it currently qualifies for. The
/// admin reviews this alongside the documents to decide which categories to
/// assign.
async fn driver_vehicle(
    Path(driver_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let data =
        ctx.db.get_driver_vehicle_and_categories(&DriverId(driver_id)).await?;
    Ok(Json(data).into_response())
}

#[derive(Debug, Deserialize)]
struct SetQualifyingReq {
    vehicle_id: String,
    /// The categories the vehicle qualifies for. Replaces the prior set; the
    /// driver's active choice is preserved for any retained category.
    categories: Vec<VehicleCategory>,
}

/// Assign the categories a driver's vehicle qualifies for, after reviewing the
/// vehicle info + documents. The driver then chooses which to serve.
async fn set_qualifying_categories(
    Path(driver_id): Path<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<SetQualifyingReq>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .set_driver_qualifying_categories(
            &driver_id,
            &body.vehicle_id,
            &body.categories,
        )
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
struct ImageQuery {
    /// `back` selects the reverse side of an identity document.
    side: Option<String>,
}

/// Stream a decrypted document image. Decryption happens here, on the private
/// plane only — the plaintext never leaves through a public route.
async fn document_image(
    Path((kind, doc_id)): Path<(String, i64)>,
    Query(q): Query<ImageQuery>,
    Extension(ctx): Extension<Arc<APIContext>>,
) -> Result<Response, AppError> {
    let kind = parse_kind(&kind)?;
    let back = q.side.as_deref() == Some("back");

    let blob = ctx
        .db
        .get_document_blob_ref(kind, doc_id, back)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("document not found".into()))?;

    let bucket = &ctx.config.bucket;
    let path_id = format!("driver-docs/{}/{}", blob.driver_id, blob.file_id);
    let data = ctx
        .config
        .aws_credentials()
        .get_uploaded_file_from_s3(
            &path_id,
            bucket,
            &blob.encrypted_key,
            &blob.nonce,
        )
        .await?;

    Ok((
        [
            (header::CONTENT_TYPE, "image/jpeg".to_string()),
            (header::CONTENT_DISPOSITION, "inline".to_owned()),
            (header::CACHE_CONTROL, "private, no-store".to_owned()),
        ],
        data,
    )
        .into_response())
}
