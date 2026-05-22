use std::sync::Arc;

use axum::{Extension, Json, extract::Path, http::StatusCode};
use redis_store::r_types::AppError;
use serde::{Deserialize, Serialize};

use utils::Result;

use crate::{
    APIContext,
    queries::{
        rating::{
            CreateRateData, CreateRiderRateData, DriverRating,
            DriverRatingSummary, RatingSummaryData, RiderRating,
        },
        ride::RideQueries,
    },
    types::{CustomerId, DriverId, RideId},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ReqBody {
    pub rating_value: i32, // To be removed once granular sub-scores are fully supported on the frontend
    pub punctuality: Option<i32>,
    pub driving_behavior: Option<i32>,
    pub safety_compliance: Option<i32>,
    pub vehicle_cleanliness: Option<i32>,
    pub feedback_details: Option<String>,
    pub was_offered_assistance: Option<bool>,
    pub attachment_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReqPath {
    pub driver_id: DriverId,
    pub ride_id: RideId,
}
/// `GET /api/v1/driver/{driver_id}/rating-summary`
///
/// Returns aggregated rating data for the given driver: overall average,
/// total review count, recommendation percentage, per-star breakdown, and
/// the 10 most recent reviews with masked rider names.
pub async fn get_rating_summary(
    Extension(ctx): Extension<Arc<APIContext>>,
    Path(driver_id): Path<String>,
) -> Result<Json<RatingSummaryData>, AppError> {
    if driver_id.is_empty() {
        return Err(AppError::ValidationError("Invalid driver ID".to_string()));
    }

    ctx.db
        .get_driver_rating_summary(&driver_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Driver not found".to_string()))
        .map(Json)
}

pub async fn rate_driver(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(customer_id): Extension<String>,
    Path(req_path): Path<ReqPath>,
    Json(req_body): Json<ReqBody>,
) -> Result<StatusCode, AppError> {
    let ReqPath { driver_id, ride_id } = req_path;
    let ReqBody {
        feedback_details,
        was_offered_assistance,
        attachment_id,
        rating_value,
        punctuality,
        driving_behavior,
        safety_compliance,
        vehicle_cleanliness,
    } = req_body;

    ctx.db
        .rate_driver_for_ride(CreateRateData {
            ride_id,
            driver_id: driver_id.to_owned(),
            customer_id: CustomerId(customer_id),
            feedback_details,
            was_offered_assistance,
            attachment_id,
            rating_value, // To be removed once granular sub-scores are fully supported on the frontend
            punctuality,
            driving_behavior,
            safety_compliance,
            vehicle_cleanliness,
        })
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

    // ctx.stats_tx.send(driver_id).await.map_err(|_| {
    //     AppError::InternalError("Failed to send driver stats".to_string())
    // })?;

    Ok(StatusCode::OK)
}

// ── Rider (driver rates the customer) ────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct RiderReqBody {
    pub punctuality: Option<i32>,
    pub respectfulness: Option<i32>,
    pub fare_readiness: Option<i32>,
    pub feedback_details: Option<String>,
    pub attachment_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RiderReqPath {
    pub ride_id: RideId,
}

pub async fn rate_rider(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Path(req_path): Path<RiderReqPath>,
    Json(req_body): Json<RiderReqBody>,
) -> Result<StatusCode, AppError> {
    let RiderReqPath { ride_id } = req_path;
    let RiderReqBody {
        punctuality,
        respectfulness,
        fare_readiness,
        feedback_details,
        attachment_id,
    } = req_body;

    // Check if ride exists and is completed before allowing rating
    let ride =
        ctx.db.get_completed_ride_by_id(&ride_id).await.map_err(|err| {
            AppError::InternalError(format!(
                "Failed to fetch ride details: {}",
                err
            ))
        })?;
    if let Some(ride) = ride {
        // Ensure that the driver is associated with the ride
        if ride.driver_id != driver_id {
            return Err(AppError::Unauthorized(
                "Driver not associated with this ride".to_string(),
            ));
        }
        ctx.db
            .rate_rider_for_ride(CreateRiderRateData {
                ride_id,
                driver_id: DriverId(driver_id),
                customer_id: CustomerId(ride.customer_id),
                punctuality,
                respectfulness,
                fare_readiness,
                feedback_details,
                attachment_id,
            })
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;
    } else {
        return Err(AppError::NotFound("Ride not found".to_string()));
    }
    Ok(StatusCode::OK)
}
