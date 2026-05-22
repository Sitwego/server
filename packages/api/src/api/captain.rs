use std::sync::Arc;

use axum::{Extension, Json, http::StatusCode};

use hyper::HeaderMap;
use redis_store::r_types::AppError;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeLocal;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utils::{
    Error, Result, hashing_algo::extract_contact_info,
    round_to_2_decimal_places,
};

use crate::{
    APIContext,
    api_responses::responces::Response,
    auth_token::Claims,
    queries::{
        bussines::{SubscriptionStatus, SubscriptionsPlans},
        drivers::{DriverQueries, DriverVehicleAndCategories},
    },
    types::{DriverId, VehicleCategory},
};

#[derive(Debug, Deserialize)]
pub struct InputData {
    pub photo_id: String,
    pub photo_nonce: Vec<u8>,
    /// KMS ciphertext blob returned from the upload endpoint — required to
    /// decrypt the photo after a process restart.
    pub photo_encrypted_key: Vec<u8>,
}
pub async fn set_driver_photo(
    Extension(state): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Json(body): Json<InputData>,
) -> Result<StatusCode, AppError> {
    info!("body {:?}", body);
    let _ = state
        .db
        .set_driver_photo_tx(
            DriverId(driver_id),
            body.photo_id,
            &body.photo_nonce,
            &body.photo_encrypted_key,
        )
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SimpleProfileResponse {
    data: DriverSimpleProfileResponse,
}
pub async fn get_driver_simple_profile(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
) -> Result<Result<Response<SimpleProfileResponse>>, AppError> {
    let simple_driver_profile = ctx
        .db
        .get_driver_simple_profile(&DriverId(driver_id.to_string()))
        .await?;

    match simple_driver_profile {
        Some((profile, vc)) => {
            // extract the contact data and convert it to a string
            let contact_data = profile.contact_data.unwrap();
            let nonce = profile.nonce.unwrap();
            // Use the stored encrypted_key blob (not the KMS key ID) so the
            // correct data key is recovered via kms:Decrypt after any restart.
            let encrypted_key = profile.encrypted_key.unwrap();
            // Empty blob means this profile was created before envelope encryption
            // was deployed — the data key was never stored and cannot be recovered.
            if encrypted_key.is_empty() {
                return Ok(Err(Error::Http(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "Contact data unrecoverable: account predates key storage (re-register to fix)".to_string(),
                    HeaderMap::new(),
                )));
            }
            let (email, phone) = extract_contact_info(
                &contact_data,
                &nonce,
                &encrypted_key,
            )
            .await
            .map_err(|err| {
                error!(
                    "Failed to extract contact info for driver {}: {:?}",
                    driver_id, err
                );
                AppError::InternalError(format!(
                    "Failed to extract contact info: {:?}",
                    err
                ))
            })?;

            let data = DriverSimpleProfileResponse {
                driver_id: profile.driver_id,
                photo_id: profile.photo_id,
                email,
                phone,
                first_name: profile.first_name,
                is_new: profile.is_new,
                verified: profile.verified,
                sub_id: profile.sub_id,
                plan_id: profile.plan_id,
                is_on_free_trial: profile.is_on_free_trial,
                free_trial_end_date: profile.free_trial_end_date,
                is_logged_in: profile.is_logged_in,
                has_onboarded: profile.has_onboarded,
                rating: profile.rating.map(round_to_2_decimal_places),
                total_earnings: profile.total_earnings,
                total_rides: profile.total_rides,
                activated: profile.activated,
                amount_due: profile.amount_due,
                is_plan_active: profile.is_plan_active,
                last_billed_at: profile.last_billed_at,
                plan_end_date: profile.plan_end_date,
                categories: vc,
            };

            Ok(Ok(Response::OK(SimpleProfileResponse { data })))
        }
        None => Ok(Err(Error::Http(
            StatusCode::NOT_FOUND,
            "Driver profile not found".to_string(),
            HeaderMap::new(),
        ))),
    }
}

pub async fn get_driver_subscription_status(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
) -> Result<Result<Response<SubscriptionStatus>>, AppError> {
    let status = ctx.db.get_driver_subscription_status(&driver_id).await?;

    match status {
        Some(s) => Ok(Ok(Response::OK(s))),
        None => Ok(Err(Error::Http(
            StatusCode::NOT_FOUND,
            "Subscription not found".to_string(),
            HeaderMap::new(),
        ))),
    }
}

#[derive(Serialize, Debug, Deserialize, PartialEq)]
pub struct DriverLoginResponse {
    pub token: Option<String>,
    pub profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    pub phone_number: String,
    pub password: String,
    pub device_id: Option<String>,
}

pub async fn login_driver(
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<LoginData>,
) -> Result<Response<DriverLoginResponse>, AppError> {
    let login_resp = ctx
        .db
        .login_driver(
            &body.phone_number,
            body.device_id.as_deref(),
            &body.password,
        )
        .await?;

    if login_resp.is_none() {
        return Ok(Response::OK(DriverLoginResponse {
            token: None,
            profile_id: None,
        }));
    }

    let token = Claims::create_token(
        &ctx.config.jwt_secrete_key,
        &login_resp.to_owned().unwrap().id,
    )
    .map_err(|err| {
        AppError::InternalError(format!(
            "Failed to create driver token {:?}",
            err
        ))
    })?;

    Ok(Response::OK(DriverLoginResponse {
        token: Some(token),
        profile_id: Some(login_resp.unwrap().id),
    }))
}

#[derive(Debug, Deserialize)]
pub struct SetDriverHasCompletedOnboardingBody {
    pub has_completed_onboarding: bool,
}

pub async fn set_driver_has_completed_onboarding(
    Extension(state): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Json(body): Json<SetDriverHasCompletedOnboardingBody>,
) -> Result<StatusCode, AppError> {
    info!("Setting driver {} as completed onboarding", driver_id);
    state
        .db
        .set_has_onboarded(&driver_id, body.has_completed_onboarding)
        .await?;
    Ok(StatusCode::OK)
}

pub async fn get_driver_categories(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
) -> Result<Response<Vec<VehicleCategory>>, AppError> {
    let categories = ctx.db.get_driver_categories(&DriverId(driver_id)).await?;
    Ok(Response::OK(categories))
}

#[derive(Debug, Deserialize)]
pub struct SetCategoriesBody {
    pub vehicle_id: String,
    pub categories: Vec<VehicleCategory>,
}

pub async fn set_driver_categories(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Json(body): Json<SetCategoriesBody>,
) -> Result<StatusCode, AppError> {
    ctx.db
        .set_driver_categories(
            &DriverId(driver_id),
            &body.vehicle_id,
            &body.categories,
        )
        .await?;
    Ok(StatusCode::OK)
}

pub async fn get_driver_vehicle_and_categories(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
) -> Result<Response<Option<DriverVehicleAndCategories>>, AppError> {
    let data =
        ctx.db.get_driver_vehicle_and_categories(&DriverId(driver_id)).await?;

    Ok(Response::OK(data))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DriverSimpleProfileResponse {
    pub driver_id: String,
    pub photo_id: Option<String>,
    pub email: String,
    pub phone: String,
    pub first_name: Option<String>,
    pub is_new: Option<bool>,
    pub verified: Option<bool>,
    pub sub_id: Option<String>,
    pub is_on_free_trial: Option<bool>,
    pub free_trial_end_date: Option<DateTimeLocal>,
    pub is_logged_in: Option<bool>,
    pub has_onboarded: Option<bool>,
    pub rating: Option<f64>,
    pub total_earnings: Option<f64>,
    pub total_rides: Option<i32>,
    pub activated: Option<bool>,
    pub amount_due: Option<Decimal>,
    pub is_plan_active: Option<bool>,
    pub last_billed_at: Option<DateTimeLocal>,
    pub plan_end_date: Option<DateTimeLocal>,
    pub categories: Vec<VehicleCategory>,
    pub plan_id: Option<String>,
}
