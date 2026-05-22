use axum::{
    Extension, Json,
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use chrono_tz::Africa::Nairobi;
use num_traits::ToPrimitive;
use redis_store::r_types::AppError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use tracing::info;
use utils::{Result, gen_strings::ulid_string, round_to_nearest_ten};

use crate::{
    APIContext,
    api_responses::responces::Response,
    queries::{
        driver_earnings::{DailyEarningsReport, DriverEarnings},
        driver_stats::DriverStatsQueries,
        ride::RideQueries,
    },
    schemas::driver_earning,
    types::{DriverId, RideId, VehicleCategory},
};
pub mod payment;

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverStats {
    pub score: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: String,
}

pub async fn go_online(
    headers: HeaderMap,
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Json(stats): Json<DriverStats>,
) -> Result<Response<u32>, StatusCode> {
    let vehicle_category = headers
        .get("vc")
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|vt_str| VehicleCategory::from_str(vt_str).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    info!(
        "Driver {} going online with vehicle category {:?}",
        driver_id, vehicle_category
    );
    let _ = ctx
        .driver_pool_manager
        .add_driver(&driver_id, stats.score)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Response::OK(0))
}

pub async fn go_offline(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
) -> Result<Response<u32>, StatusCode> {
    info!("Driver {} going offline", driver_id);
    ctx.driver_pool_manager
        .remove_driver(&driver_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Response::OK(0))
}

pub async fn confirm_collected_cash(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Path((ride_id, is_discounted, discount)): Path<(RideId, bool, i32)>,
) -> Result<Response<u32>, AppError> {
    // check if ride exists
    let ride = ctx.db.ride_exist(&ride_id).await?;
    if let Some(r) = ride {
        let fare: Decimal = r.fare.unwrap_or(Decimal::default());
        let amount =
            round_to_nearest_ten::<f64>(fare.try_into().unwrap_or(0.0));
        let dr_e_mdl = driver_earning::Model {
            id: ulid_string(),
            driver_id: driver_id.to_owned(),
            amount: Decimal::from(amount as i32),
            is_discounted,
            discount: Decimal::from(discount),
            payment_status: "completed".to_string(),
            currency: "KES".to_string(),
            ride_id: ride_id.0,
            created_at: chrono::Utc::now()
                .with_timezone(&Nairobi)
                .fixed_offset(),
            updated_at: chrono::Utc::now()
                .with_timezone(&Nairobi)
                .fixed_offset(),
            ..Default::default()
        };

        ctx.db.insert_driver_earning_for_ride(&dr_e_mdl).await?;
        ctx.db
            .update_driver_total_earnings(
                &DriverId(driver_id),
                &amount.to_i32().unwrap_or(0),
            )
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        Ok(Response::OK(0))
    } else {
        return Err(AppError::NotFound(format!(
            "Ride for ride id {:?} not found",
            ride_id.inner()
        )));
    }
}

pub async fn get_driver_daily_earnings(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Path(date): Path<String>,
) -> Result<Response<DailyEarningsReport>, AppError> {
    let report =
        ctx.db.get_daily_earnings_for_driver(&driver_id, &date).await?;
    Ok(Response::OK(report))
}

pub async fn get_driver_weekly_earnings(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Path(date): Path<String>,
) -> Result<
    Response<crate::queries::driver_earnings::WeeklyEarningsReport>,
    AppError,
> {
    let earnings =
        ctx.db.get_weekly_earnings_for_driver(&driver_id, &date).await?;

    info!("Weekly earnings for driver {}: {:?}", driver_id, earnings);

    Ok(Response::OK(earnings))
}
