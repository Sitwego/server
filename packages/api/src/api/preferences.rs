use std::sync::Arc;

use axum::{Extension, Json, extract::Path};
use redis_store::r_types::AppError;
use serde::Deserialize;
use tracing::info;

use crate::{
    APIContext, DriverId,
    cache::read_writer::set_driver_max_radius,
    queries::preferences::PreferencesQueries,
    schemas::travel_preferences::{TravelPreferences, TravelPreferencesUpdate},
};

#[derive(Debug, Deserialize)]
pub struct SaveBioRequest {
    pub bio: String,
}

/// `GET /api/driver/{driver_id}/preferences`
///
/// Returns the driver's current travel preferences. If the profile exists but
/// no preferences have been set yet, the response will be an empty JSON object
/// (`{}`), which deserializes to a default `TravelPreferences` value.
pub async fn get_preferences(
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(driver_id): Path<String>,
) -> Result<Json<TravelPreferences>, AppError> {
    info!("GET preferences for driver {driver_id}");

    let prefs = ctx
        .db
        .get_driver_preferences(&driver_id)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?
        .ok_or_else(|| {
            AppError::NotFound(format!("Driver not found: {driver_id}"))
        })?;

    Ok(Json(prefs))
}

/// `PUT /api/profile/bio`
///
/// Saves the bio for the authenticated profile.
pub async fn save_bio(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(profile_id): Extension<String>,
    Json(body): Json<SaveBioRequest>,
) -> Result<(), AppError> {
    info!("PUT bio for profile {profile_id}");

    ctx.db
        .save_bio(&profile_id, body.bio)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    Ok(())
}

pub async fn update_preferences(
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(driver_id): Path<String>,
    Json(updates): Json<TravelPreferencesUpdate>,
) -> Result<Json<TravelPreferences>, AppError> {
    info!("PUT preferences for driver {driver_id}: {updates:?}");

    // Extract before `updates` is consumed by the DB call.
    // `Option<Option<f64>>` is Copy so this doesn't move the struct.
    let radius_update = updates.max_ride_radius_km;

    let updated =
        ctx.db.update_driver_preferences(&driver_id, updates).await.map_err(
            |e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    AppError::NotFound(format!("Driver not found: {driver_id}"))
                } else {
                    AppError::InternalError(msg)
                }
            },
        )?;

    // Sync to Redis so the dispatch layer can filter without a DB hit.
    // Only act when the field was present in the request (outer Some).
    if let Some(radius_km) = radius_update {
        set_driver_max_radius(
            &ctx.redis,
            &DriverId(driver_id),
            radius_km,
            &ctx.config.exp_ttl,
        )
        .await?;
    }

    Ok(Json(updated))
}
