use std::sync::Arc;

use axum::{Extension, Json, extract::Path, http::StatusCode};
use redis_store::r_types::AppError;
use serde::Serialize;

use crate::{
    APIContext, queries::ride_fare::RideFareQueries, schemas::ride_fare,
};

#[derive(Debug, Serialize)]
pub struct FareComponentValue {
    pub key: String,
    pub value: serde_json::Value,
}

/// Trimmed view of a fare snapshot for history responses — omits `id` and
/// `ride_id` which are redundant in a per-ride history call.
#[derive(Debug, Serialize)]
pub struct FareHistoryEntry {
    pub components: serde_json::Value,
    pub total: rust_decimal::Decimal,
    pub status: String,
    pub reason: Option<String>,
    pub recorded_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<ride_fare::Model> for FareHistoryEntry {
    fn from(m: ride_fare::Model) -> Self {
        Self {
            components: m.components,
            total: m.total,
            status: m.status,
            reason: m.reason,
            recorded_at: m.recorded_at,
        }
    }
}

/// GET /api/rides/{ride_id}/fare
/// Returns the latest fare snapshot for a ride.
pub async fn get_current_fare(
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(ride_id): Path<String>,
) -> Result<Json<ride_fare::Model>, AppError> {
    let fare = ctx.db.get_current_fare(&ride_id).await?.ok_or_else(|| {
        AppError::NotFound(format!("No fare found for ride {}", ride_id))
    })?;
    Ok(Json(fare))
}

/// GET /api/rides/{ride_id}/fare/history
/// Returns every fare snapshot recorded for a ride, oldest first.
pub async fn get_fare_history(
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(ride_id): Path<String>,
) -> Result<Json<Vec<FareHistoryEntry>>, AppError> {
    let history = ctx.db.get_fare_history(&ride_id).await?;
    Ok(Json(
        history.into_iter().map(FareHistoryEntry::from).collect(),
    ))
}

/// GET /api/rides/{ride_id}/fare/components/{key}
/// Returns a single component value from the latest fare snapshot.
pub async fn get_fare_component(
    Extension(ctx): Extension<Arc<APIContext>>,
    Path((ride_id, key)): Path<(String, String)>,
) -> Result<Json<FareComponentValue>, AppError> {
    let fare = ctx.db.get_current_fare(&ride_id).await?.ok_or_else(|| {
        AppError::NotFound(format!("No fare found for ride {}", ride_id))
    })?;

    let value = fare.components.get(&key).cloned().ok_or_else(|| {
        AppError::NotFound(format!(
            "Component '{}' not present in current fare",
            key
        ))
    })?;

    Ok(Json(FareComponentValue { key, value }))
}

#[allow(dead_code)]
pub async fn _options() -> StatusCode {
    StatusCode::OK
}
