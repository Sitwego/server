//! Driver referral programme (public driver-facing endpoints).
//!
//! A driver gets one immutable referral code at registration and shares it; a
//! new driver enters it at sign-up. When the referred driver is activated
//! (KYC approved / go-live, see the admin plane) the referrer is rewarded
//! exactly once and push-notified.
//!
//! Routes mounted under the authenticated driver router (see
//! [`crate::api::handlers`]):
//!   * `GET /driver/referral/code`    — this driver's referral code.
//!   * `GET /driver/referral/stats`   — aggregate counts + days earned.
//!   * `GET /driver/referral/history` — paginated list of referred drivers.
//!
//! Code generation, validation at registration, and the reward trigger live in
//! the queries layer (`crate::queries::referral`) so they can be reused from
//! the registration and admin-activation paths.

use std::sync::Arc;

use axum::{Extension, Json, extract::Query};
use redis_store::r_types::AppError;
use serde::{Deserialize, Serialize};

use crate::{
    APIContext,
    queries::referral::{ReferralHistoryItem, ReferralQueries, ReferralStats},
};

#[derive(Debug, Serialize)]
pub struct ReferralCodeResponse {
    pub code: String,
}

/// `GET /driver/referral/code`
///
/// The authenticated driver's shareable referral code. Generated at
/// registration; this also heals older accounts by creating one on first read.
pub async fn get_referral_code(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(profile_id): Extension<String>,
) -> Result<Json<ReferralCodeResponse>, AppError> {
    let code = ctx
        .db
        .get_or_create_referral_code(&profile_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(ReferralCodeResponse { code }))
}

/// `GET /driver/referral/stats`
///
/// Aggregate referral counts for the authenticated driver's dashboard.
pub async fn get_referral_stats(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(profile_id): Extension<String>,
) -> Result<Json<ReferralStats>, AppError> {
    let stats = ctx
        .db
        .get_referral_stats(&profile_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(stats))
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// `GET /driver/referral/history?limit=&offset=`
///
/// Paginated list of the drivers this driver referred, newest first.
pub async fn get_referral_history(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(profile_id): Extension<String>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Vec<ReferralHistoryItem>>, AppError> {
    let limit = q.limit.unwrap_or(20).min(100);
    let offset = q.offset.unwrap_or(0);
    let items = ctx
        .db
        .get_referral_history(&profile_id, limit, offset)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
    Ok(Json(items))
}
