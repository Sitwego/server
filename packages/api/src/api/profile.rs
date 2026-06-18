use std::sync::Arc;

use axum::{Extension, Json, extract::Path, http::StatusCode};

use redis_store::r_types::AppError;
use serde::{Deserialize, Serialize};
use time::Date;
use time::macros::format_description;
use tracing::info;
use utils::{
    Result,
    hashing_algo::{EncryptedRecord, SensitiveData, encrypt_data},
};

use crate::{
    APIContext,
    auth_token::Claims,
    helper::hash_password,
    queries::profile::{PersonalDetailsUpdate, ProfileQueries},
    queries::referral::ReferralQueries,
    types::ContactData,
};

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceInfoRequest {
    pub device_type: String,
    pub device_token: String,
}

/// `PUT /api/profile/device-info`
///
/// Updates the authenticated profile's device type and push notification token.
pub async fn update_device_info(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(profile_id): Extension<String>,
    Json(body): Json<UpdateDeviceInfoRequest>,
) -> Result<(), AppError> {
    info!("PUT device-info for profile {profile_id}");

    ctx.db
        .update_device_info(&profile_id, body.device_type, body.device_token)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileReqObject {
    pub contact_data: ContactData,
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub mobile_country_code: Option<String>,
    pub password: String,
    /// Referral code of an existing driver (driver sign-ups only). Invalid
    /// codes reject the registration with 400.
    #[serde(default)]
    pub referral_code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProfileCreateObject {
    pub id: String,
    pub nonce: Vec<u8>,
    pub encrypted_key: Vec<u8>,
    pub contact_data: Vec<u8>,
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub mobile_country_code: String,
    pub password: String,
    pub phone_hash: String,
    pub email_hash: String,
}

#[derive(Debug, Serialize)]
pub struct CreateProfileResponse {
    pub profile_id: String,
    pub token: String,
}

#[axum_macros::debug_handler]
pub async fn create_profile(
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(c): Path<String>,
    Json(profile_obj): Json<ProfileReqObject>,
) -> Result<Json<CreateProfileResponse>, StatusCode> {
    let is_profile_driver = c == "driver";
    let phone_hash =
        utils::hashing_algo::hash_value(&profile_obj.contact_data.phone_number);
    let email_hash =
        utils::hashing_algo::hash_value(&profile_obj.contact_data.email);
    let exists = ctx
        .db
        .check_hash_exists(
            phone_hash.to_owned(),
            email_hash.to_owned(),
            is_profile_driver,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to check hash exists: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if exists {
        info!("Hash exists: {:?}", exists);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Pre-validate the referral code BEFORE the profile is created, so an
    // invalid code rejects the registration cleanly (driver sign-ups only).
    let referral_code = profile_obj
        .referral_code
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_uppercase);
    if is_profile_driver && let Some(ref code) = referral_code {
        let valid = ctx.db.referral_code_exists(code).await.map_err(|e| {
            tracing::error!("Failed to check referral code: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        if !valid {
            info!("Rejected unknown referral code at registration: {code}");
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let sensitive_data = serde_json::to_vec(&SensitiveData {
        email: profile_obj.contact_data.email,
        phone: profile_obj.contact_data.phone_number,
    })
    .map_err(|e| {
        tracing::error!("SensitiveData Failed to convert!: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    // encrypt_data returns a nonce (ChaCha20 IV) AND an encrypted_key blob
    // (the KMS-encrypted form of the data key). Both must be stored in the DB
    // so the contact data can be decrypted after a process restart.
    let EncryptedRecord {
        ciphertext,
        nonce,
        encrypted_key,
        ..
    } = encrypt_data(&ctx.config.kms_key_id, &sensitive_data).await.map_err(
        |e| {
            tracing::error!("Failed to encrypt data: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        },
    )?;
    let password = hash_password(&profile_obj.password).map_err(|e| {
        tracing::error!("Failed to hash password: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let profile_obj = ProfileCreateObject {
        id: utils::gen_strings::ulid_string(),
        nonce,
        encrypted_key,
        contact_data: ciphertext,
        first_name: profile_obj.first_name,
        last_name: profile_obj.last_name,
        gender: profile_obj.gender,
        password,
        phone_hash,
        email_hash,
        mobile_country_code: profile_obj
            .mobile_country_code
            .unwrap_or_else(|| "+254".to_string()),
    };

    let profile_id = ctx
        .db
        .create_profile(profile_obj, is_profile_driver)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create profile: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .0;

    if is_profile_driver {
        // Give the new driver their own shareable referral code. Non-fatal:
        // get_or_create_referral_code is idempotent, so a miss here heals the
        // first time the driver opens the referral screen.
        if let Err(e) = ctx.db.get_or_create_referral_code(&profile_id).await {
            tracing::error!(
                "Failed to create referral code for {profile_id}: {e:?}"
            );
        }

        // Record who referred this driver. The code was validated above, so a
        // failure here is a lost race — registration itself must still succeed.
        if let Some(code) = referral_code
            && let Err(e) = ctx.db.create_referral(&profile_id, &code).await
        {
            tracing::error!(
                "Failed to record referral for {profile_id}: {e:?}"
            );
        }
    }

    let token = Claims::create_token(&ctx.config.jwt_secrete_key, &profile_id)
        .map_err(|e| {
            tracing::error!("Failed to create access_token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(CreateProfileResponse { profile_id, token }))
}

#[derive(Debug, Deserialize)]
pub struct UpdatePersonalDetailsRequest {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub date_of_birth: String,
}

/// `PUT /api/profile/personal-details`
///
/// Updates the authenticated driver's personal details (name and date of birth).
#[axum_macros::debug_handler]
pub async fn update_personal_details(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(profile_id): Extension<String>,
    Json(body): Json<UpdatePersonalDetailsRequest>,
) -> Result<(), AppError> {
    info!("PUT personal-details for profile {profile_id}");

    let format = format_description!("[year]-[month]-[day]");
    let dob = Date::parse(&body.date_of_birth, &format).map_err(|e| {
        AppError::ValidationError(format!("Invalid date_of_birth: {e}"))
    })?;

    ctx.db
        .update_driver_personal_details(
            &profile_id,
            PersonalDetailsUpdate {
                first_name: body.first_name,
                middle_name: body.middle_name,
                last_name: body.last_name,
                date_of_birth: dob,
            },
        )
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(())
}
