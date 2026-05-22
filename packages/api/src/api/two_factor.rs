use std::sync::Arc;

use axum::http::HeaderMap;
use axum::{Extension, Json, http::StatusCode};
use redis_store::r_types::AppError;
use serde::{Deserialize, Serialize};
use sms_api::VerifyChannel;

use crate::APIContext;

// ── shared guards ─────────────────────────────────────────────────────────────

/// Max OTP sends per phone number within `OTP_RATE_WINDOW_SECS`.
const OTP_RATE_LIMIT: i64 = 5;
/// 10-minute sliding window (resets from the first request in the window).
const OTP_RATE_WINDOW_SECS: i64 = 600;

/// Validate the `X-Api-Key` header against the configured secret.
fn check_api_key(
    headers: &HeaderMap,
    expected: &str,
) -> Result<(), (StatusCode, AppError)> {
    let provided =
        headers.get("x-api-key").and_then(|v| v.to_str().ok()).unwrap_or("");

    if provided != expected {
        return Err((
            StatusCode::UNAUTHORIZED,
            AppError::ValidationError("Missing or invalid API key".into()),
        ));
    }
    Ok(())
}

/// Increment the per-phone rate-limit counter and reject when the cap is hit.
async fn check_rate_limit(
    ctx: &APIContext,
    phone: &str,
) -> Result<(), (StatusCode, AppError)> {
    let key = format!("2fa:rl:{}", phone);
    match ctx.redis.incr_with_expiry(&key, OTP_RATE_WINDOW_SECS).await {
        Ok(count) if count > OTP_RATE_LIMIT => Err((
            StatusCode::TOO_MANY_REQUESTS,
            AppError::ValidationError(
                "Too many OTP requests. Please wait before trying again."
                    .into(),
            ),
        )),
        Ok(_) => Ok(()),
        Err(e) => {
            // Redis failure: log it but don't block the user
            tracing::error!(error = %e, "2FA rate-limit Redis error; allowing request");
            Ok(())
        }
    }
}

// ── send OTP ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SendOtpRequest {
    /// Recipient phone number in E.164 format, e.g. `"+254711000111"`.
    pub phone_number: String,
    /// Delivery channel. Defaults to `sms` when omitted.
    pub channel: Option<OtpChannel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OtpChannel {
    Sms,
    Call,
    WhatsApp,
}

impl From<OtpChannel> for VerifyChannel {
    fn from(c: OtpChannel) -> Self {
        match c {
            OtpChannel::Sms => VerifyChannel::Sms,
            OtpChannel::Call => VerifyChannel::Call,
            OtpChannel::WhatsApp => VerifyChannel::WhatsApp,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SendOtpResponse {
    pub sid: Option<String>,
    pub status: Option<String>,
}

/// `POST /api/2fa/send`
///
/// Requires `X-Api-Key` header. Rate-limited to 5 requests per phone per 10 min.
pub async fn send_otp(
    Extension(ctx): Extension<Arc<APIContext>>,
    headers: HeaderMap,
    Json(body): Json<SendOtpRequest>,
) -> Result<Json<SendOtpResponse>, (StatusCode, AppError)> {
    check_api_key(&headers, &ctx.config.otp_api_key)?;
    check_rate_limit(&ctx, &body.phone_number).await?;

    let channel = body.channel.unwrap_or(OtpChannel::Sms).into();

    tracing::info!(
        phone = %body.phone_number,
        "2FA: sending OTP"
    );

    let resp = ctx.verify.send_otp(&body.phone_number, channel).await.map_err(
        |e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::InternalError(e.to_string()),
            )
        },
    )?;

    Ok(Json(SendOtpResponse {
        sid: resp.sid,
        status: resp.status,
    }))
}

// ── verify OTP ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub phone_number: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyOtpResponse {
    pub approved: bool,
    pub status: Option<String>,
}

/// `POST /api/2fa/verify`
///
/// Requires `X-Api-Key` header.
/// Returns `approved: true` when the code is correct and not expired.
pub async fn verify_otp(
    Extension(ctx): Extension<Arc<APIContext>>,
    headers: HeaderMap,
    Json(body): Json<VerifyOtpRequest>,
) -> Result<Json<VerifyOtpResponse>, (StatusCode, AppError)> {
    check_api_key(&headers, &ctx.config.otp_api_key)?;

    tracing::info!(
        phone = %body.phone_number,
        "2FA: checking OTP"
    );

    let resp =
        ctx.verify.check_otp(&body.phone_number, &body.code).await.map_err(
            |e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::InternalError(e.to_string()),
                )
            },
        )?;

    if !resp.is_approved() {
        tracing::warn!(phone = %body.phone_number, "2FA: OTP rejected");
        return Err((
            StatusCode::UNAUTHORIZED,
            AppError::ValidationError("Invalid or expired OTP".into()),
        ));
    }

    Ok(Json(VerifyOtpResponse {
        approved: true,
        status: resp.status,
    }))
}
