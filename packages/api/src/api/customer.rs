use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, Query},
};
use redis_store::r_types::AppError;
use serde::{Deserialize, Serialize};
use time;
use tracing::info;
use utils::Result;

use utils::hashing_algo::extract_contact_info;

use crate::APIContext;
use crate::api_responses::responces::Response;
use crate::auth_token::Claims;
use crate::queries::customer::{
    get_customer_profile::{GetRiderProfile, RiderProfile},
    get_ride_detail::{GetRideDetail, RideDetail},
    get_ride_history::{GetRiderRideHistory, RideHistoryGroup},
    login::LoginCustomer,
    update_customer_profile::{
        AddressInput, LinkGoogleAccount, LinkGoogleInput,
        UpdateCustomerProfile, UpdateRiderProfileInput, UpsertCustomerAddress,
    },
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CustomerLogin {
    pub phone_number: String,
    pub device_id: String,
    pub password: Option<String>,
}
// /// Handles customer login by validating phone_number i.e 0700000000 and device_id.
/// If password is provided, it will be used for authentication.
/// Returns a response with customer data, token and id if successful, or false if not found.
/// /// # Arguments
/// * `state` - The application state containing the database connection.
/// * `body` - The JSON body containing phone_number, device_id, and optional password
pub async fn login_customer(
    Extension(state): Extension<Arc<APIContext>>,
    Json(body): Json<CustomerLogin>,
) -> Result<Response<LoginResponse>, AppError> {
    let customer_opt = state
        .db
        .login_customer(
            &body.phone_number,
            &body.device_id,
            body.password.as_deref(),
        )
        .await?;
    if let Some(customer) = customer_opt {
        let token =
            Claims::create_token(&state.config.jwt_secrete_key, &customer.id)
                .map_err(|e| {
                tracing::error!("Failed to create access_token: {:?}", e);
                AppError::InternalError(e.to_string())
            })?;
        let response = CustomerLoginResponse {
            id: customer.id,
            phone_number: body.phone_number,
            //TODO::
            email: None,
            first_name: None,
            last_name: None,
            token,
        };
        return Ok(Response::OK(LoginResponse {
            data: Some(response),
            success: true,
        }));
    }
    Ok(Response::OK(LoginResponse {
        data: None,
        success: false,
    }))
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CustomerLoginResponse {
    pub id: String,
    pub phone_number: String,
    pub email: Option<String>,
    pub token: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LoginResponse {
    pub data: Option<CustomerLoginResponse>,
    pub success: bool,
}

// ── Ride history ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RideHistoryParams {
    #[serde(default)]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page_size() -> u32 {
    20
}

// ── Update profile ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PatchRiderProfileBody {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub mobile_country_code: Option<String>,
    /// ISO 8601 date string e.g. "1990-01-15"
    pub dob: Option<String>,
    pub avatar_url: Option<String>,
}

/// PATCH /api/customer/profile
pub async fn patch_customer_profile(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(customer_id): Extension<String>,
    Json(body): Json<PatchRiderProfileBody>,
) -> Result<(), AppError> {
    let dob = body
        .dob
        .as_deref()
        .map(|s| {
            time::Date::parse(
                s,
                &time::macros::format_description!("[year]-[month]-[day]"),
            )
            .map_err(|_| {
                AppError::ValidationError(
                    "Invalid date format, expected YYYY-MM-DD".to_string(),
                )
            })
        })
        .transpose()?;

    let input = UpdateRiderProfileInput {
        first_name: body.first_name,
        last_name: body.last_name,
        mobile_country_code: body.mobile_country_code,
        dob,
    };

    ctx.db
        .update_customer_profile(&customer_id, input)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(())
}

// ── Address ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpsertAddressBody {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
}

/// PUT /api/customer/profile/address
pub async fn upsert_customer_address(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(customer_id): Extension<String>,
    Json(body): Json<UpsertAddressBody>,
) -> Result<(), AppError> {
    ctx.db
        .upsert_customer_address(
            &customer_id,
            AddressInput {
                street: body.street,
                city: body.city,
                state: body.state,
                zip: body.zip,
            },
        )
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(())
}

/// GET /api/customer/profile
pub async fn get_customer_profile(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(customer_id): Extension<String>,
) -> Result<Json<RiderProfile>, AppError> {
    let row = ctx
        .db
        .get_rider_profile(&customer_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or_else(|| {
            AppError::NotFound("Customer profile not found".to_string())
        })?;

    // Use the per-record encrypted_key blob (not the KMS key ID) so the correct
    // data key is recovered via kms:Decrypt regardless of process restarts.
    let (email, phone_number) =
        extract_contact_info(&row.contact_data, &row.nonce, &row.encrypted_key)
            .await
            .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(Json(row.into_profile(email, phone_number)))
}

/// GET /api/customer/rides/{ride_id}
pub async fn get_ride_detail(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(customer_id): Extension<String>,
    Path(ride_id): Path<String>,
) -> Result<Json<RideDetail>, AppError> {
    info!(
        "Fetching ride detail for ride_id: {} and customer_id: {}",
        ride_id, customer_id
    );
    let detail = ctx
        .db
        .get_ride_detail(&ride_id, &customer_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Ride not found".to_string()))?;

    Ok(Json(detail))
}

// ── Link Google ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LinkGoogleBody {
    pub google_linked: bool,
    pub google_email: Option<String>,
    pub id_token: Option<String>,
}

/// PUT /api/customer/link-google
pub async fn link_google(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(customer_id): Extension<String>,
    Json(body): Json<LinkGoogleBody>,
) -> Result<(), AppError> {
    ctx.db
        .link_google_account(
            &customer_id,
            LinkGoogleInput {
                google_linked: body.google_linked,
                google_email: body.google_email,
                id_token: body.id_token,
            },
        )
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(())
}

/// GET /api/customer/ride-history?page=0&page_size=20
pub async fn get_ride_history(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(customer_id): Extension<String>,
    Query(params): Query<RideHistoryParams>,
) -> Result<Json<Vec<RideHistoryGroup>>, AppError> {
    let groups = ctx
        .db
        .get_rider_ride_history(&customer_id, params.page, params.page_size)
        .await?;

    Ok(Json(groups))
}
