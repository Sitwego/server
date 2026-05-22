use axum::{Extension, Json, extract};
use redis_store::r_types::GeoPoint;
use std::sync::Arc;
// use geo::{Distance, Haversine, Point};
use crate::{APIContext, api::ride_request::RequestRideData, types::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriverCurrentLocationInfo {
    pub coords: GeoPoint,
    pub timestamp: TimeStamp,
    pub driver_id: DriverId,
    pub distance: f64,
    pub vehicle_category: VehicleCategory,
}

#[derive(Debug, Deserialize)]
pub struct NearbydriverInput {
    from: RequestRideData,
    pub vehicle_type: Option<Vec<VehicleCategory>>,
    pub radius: Radius,
}

pub async fn get_nearby_drivers(
    Extension(ctx): Extension<Arc<APIContext>>,
    extract::Json(body): extract::Json<NearbydriverInput>,
) -> Result<Json<Vec<DriverCurrentLocationInfo>>, AppError> {
    let res: Vec<DriverCurrentLocationInfo> =
        crate::dispatch::state_machine::find_nearest_driver(
            ctx.redis.clone(),
            body.from.geo_point,
            &body.vehicle_type,
            &body.radius,
        )
        .await?;
    Ok(Json(res))
}
