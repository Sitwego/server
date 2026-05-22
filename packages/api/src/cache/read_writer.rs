use super::keys::{
    driver_location_info_key, driver_max_radius_key,
    driver_opted_categories_key, on_driver_location_key,
    on_going_ride_coordinates_key, ride_info_key, ride_path_key_id,
};
use crate::{
    DriverId, DriverLocation, DriverLocationIfo, DriverLocationMap, GeoPoint,
    Radius, RideId, RideInfo, VehicleCategory,
    schemas::ride_request::RideRequestStatus,
    types::{DriverLocationCoordinates, Meters, RideNotificationState},
};
use fred::types::{SortOrder, geo::*};
use redis_store::{RedisConnectionPool, r_types::*};
use std::{future::Future, sync::Arc};
use tracing::{error, info, warn};
use utils::collections::FxHashSet;

pub async fn with_redis_lock<F, Args, Fut, R>(
    redis: Arc<RedisConnectionPool>,
    args: Args,
    c: F,
    key: &str,
    exp: i64,
) -> Result<R, AppError>
where
    F: Fn(Args) -> Fut,
    Args: Send + 'static,
    Fut: Future<Output = Result<R, AppError>>,
{
    // Generate a unique token so release_lock can verify ownership.
    let token = uuid::Uuid::new_v4().to_string();

    // Single atomic SET key token NX EX expiry — no pre-check round-trip needed.
    let acquired = redis
        .acquire_lock(key, &token, exp)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    if !acquired {
        warn!("Lock contention on key: {}", key);
        return Err(AppError::LockContention);
    }

    info!("Acquired lock for key: {}", key);
    let res = c(args).await;

    // Release only if the token still matches — safe even if the TTL expired
    // and another owner grabbed the lock while the callback was running.
    if let Err(e) = redis.release_lock(key, &token).await {
        error!("Failed to release lock {}: {}", key, e);
    } else {
        info!("Released lock for key: {}", key);
    }

    res
}

pub async fn set_driver_location_info(
    redis: &RedisConnectionPool,
    key: String,
    position_info: &DriverLocation,
    ride_status: &Option<RideRequestStatus>,
    ride_notification_state: &Option<RideNotificationState>,
    pickup_location_distance: &Option<Meters>,
    exp: &i64,
) -> Result<(), AppError> {
    let data = DriverLocationIfo {
        ride_status: ride_status.to_owned(),
        ride_notification_state: (*ride_notification_state)
            .or(Some(RideNotificationState::Idle)),
        pickup_location_distance: *pickup_location_distance,
        position_info: position_info.clone(),
    };
    redis
        .set_key(&key, data, *exp)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

    Ok(())
}

pub async fn get_driver_location_info(
    redis: &RedisConnectionPool,
    id: DriverId,
) -> Option<DriverLocationIfo> {
    redis
        .get_key::<DriverLocationIfo>(&driver_location_info_key(id))
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))
        .ok()?
}

pub async fn get_ride_info(
    redis: &RedisConnectionPool,
    id: DriverId,
) -> Option<RideInfo> {
    redis
        .get_key::<RideInfo>(&ride_info_key(id))
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))
        .ok()?
}

pub async fn set_ride_info(
    redis: &RedisConnectionPool,
    driver_id: &DriverId,
    ride_info: &RideInfo,
    exp: &i64,
) -> Result<(), AppError> {
    redis
        .set_key(&ride_info_key(driver_id.to_owned()), &ride_info, *exp)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))
}

pub async fn drain_driver_locations(
    redis: &RedisConnectionPool,
    geo_entries: &DriverLocationMap,
    exp: &i64,
) -> Result<(), AppError> {
    redis
        .mgeo_add_with_expiry(geo_entries, None, false, *exp)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))
}

pub async fn set_on_going_ride_coordinates(
    redis: &RedisConnectionPool,
    exp: &i64,
    driver_id: &DriverId,
    ride_id: &RideId,
    coordinates: Vec<GeoPoint>,
) -> Result<i64, AppError> {
    let key = on_going_ride_coordinates_key(driver_id, ride_id);

    redis
        .rpush_with_expiration(&key, coordinates, exp)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))
}

pub async fn get_on_going_ride_coordinates(
    redis: &RedisConnectionPool,
    driver_id: &DriverId,
    ride_id: &RideId,
    max: &i64,
) -> Result<Vec<GeoPoint>, AppError> {
    redis
        .lrange::<GeoPoint>(
            &on_going_ride_coordinates_key(driver_id, ride_id),
            0,
            *max,
        )
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))
}

pub async fn get_llen(
    redis: &RedisConnectionPool,
    driver_id: &DriverId,
    ride_id: &RideId,
) -> Result<i64, AppError> {
    redis
        .llen(&on_going_ride_coordinates_key(driver_id, ride_id))
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))
}

pub async fn get_and_delete_on_going_ride_loactions(
    redis: &RedisConnectionPool,
    driver_id: &DriverId,
    ride_id: &RideId,
    count: &i64,
) -> Result<Vec<GeoPoint>, AppError> {
    let key = on_going_ride_coordinates_key(driver_id, ride_id);
    redis
        .lpop::<GeoPoint>(&key, Some(*count as usize))
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))
}

pub async fn search_for_nearby_drivers_within_radius(
    redis: &RedisConnectionPool,
    buckect_key: &i64,
    bckt_threshold: &i64,
    vehicle_type: &VehicleCategory,
    Radius(radius): &Radius,
    coordinate: &GeoPoint,
    // _region: &TownName,
) -> Result<Vec<DriverLocationCoordinates>, AppError> {
    let keys: Vec<String> = (0..*bckt_threshold)
        .map(|idx| on_driver_location_key(&(buckect_key - idx), vehicle_type))
        .collect();
    let result = redis
        .mgeo_search(
            keys,
            GeoPosition::from((coordinate.lon.0, coordinate.lat.0)),
            (*radius, GeoUnit::Meters),
            SortOrder::Asc,
        )
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

    let nearby_drivers = result
        .into_iter()
        .map(|(d, p, distance)| DriverLocationCoordinates {
            coords: p,
            dirver_id: DriverId(d),
            distance,
        })
        .collect::<Vec<DriverLocationCoordinates>>();

    let mut res: Vec<DriverLocationCoordinates> =
        Vec::with_capacity(nearby_drivers.len());
    let mut ids: FxHashSet<DriverId> = FxHashSet::default();
    for l in nearby_drivers.into_iter() {
        if !(ids.contains(&l.dirver_id)) {
            ids.insert(l.dirver_id.clone());
            res.push(l);
        }
    }
    Ok(res)
}

pub async fn get_all_drivers_last_locations(
    redis: &RedisConnectionPool,
    driver_ids: &[DriverId],
) -> Result<Vec<Option<DriverLocation>>, AppError> {
    let keys = driver_ids
        .iter()
        .map(|driver_id| driver_location_info_key(driver_id.clone()))
        .collect::<Vec<String>>();

    let driver_locations = redis
        .mget_keys::<DriverLocationIfo>(keys)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?
        .into_iter()
        .map(|opt_l| opt_l.map(|l| l.position_info))
        .collect::<Vec<Option<DriverLocation>>>();

    Ok(driver_locations)
}

pub async fn ride_clean_up(
    redis: &RedisConnectionPool,
    driver_id: &DriverId,
    ride_id: &RideId,
    ride_path_id: &str,
) -> Result<(), AppError> {
    redis
        .delete_keys(vec![
            &on_going_ride_coordinates_key(driver_id, ride_id),
            &driver_location_info_key(driver_id.to_owned()),
            &ride_path_key_id(ride_path_id),
            &ride_info_key(driver_id.to_owned()),
        ])
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;
    Ok(())
}

/// Stores a driver's opted-in optional categories in Redis HSET.
/// Replaces any previously stored selection.
/// Call this when the driver updates their category preferences in the app.
pub async fn set_driver_opted_categories(
    redis: &RedisConnectionPool,
    driver_id: &DriverId,
    categories: &[VehicleCategory],
) -> Result<(), AppError> {
    let key = driver_opted_categories_key(driver_id);
    let fields: Vec<String> =
        categories.iter().map(|c| c.to_string()).collect();
    redis
        .hset_fields(&key, fields)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))
}

/// Writes a driver's max ride radius (km) to Redis, or deletes the key when
/// `radius_km` is `None` (meaning the driver cleared their restriction).
pub async fn set_driver_max_radius(
    redis: &RedisConnectionPool,
    driver_id: &DriverId,
    radius_km: Option<f64>,
    exp: &i64,
) -> Result<(), AppError> {
    let key = driver_max_radius_key(driver_id);
    match radius_km {
        Some(r) => redis
            .set_key(&key, r, *exp)
            .await
            .map_err(|e| AppError::InternalError(e.to_string())),
        None => redis
            .delete_key(&key)
            .await
            .map_err(|e| AppError::InternalError(e.to_string())),
    }
}

/// Batch-fetches the max ride radii for a slice of drivers.
/// Drivers without a stored preference are absent from the returned map.
pub async fn get_drivers_max_radii(
    redis: &RedisConnectionPool,
    driver_ids: &[DriverId],
) -> std::collections::HashMap<String, f64> {
    if driver_ids.is_empty() {
        return std::collections::HashMap::new();
    }
    let keys: Vec<String> =
        driver_ids.iter().map(driver_max_radius_key).collect();
    match redis.mget_keys::<f64>(keys).await {
        Ok(values) => driver_ids
            .iter()
            .zip(values)
            .filter_map(|(id, opt_r)| opt_r.map(|r| (id.0.clone(), r)))
            .collect(),
        Err(e) => {
            tracing::warn!("Failed to fetch driver max radii: {e}");
            std::collections::HashMap::new()
        }
    }
}

/// Returns the driver's opted-in optional categories from Redis HSET.
/// Returns an empty Vec if the driver has no optional categories set.
pub async fn get_driver_opted_categories(
    redis: &RedisConnectionPool,
    driver_id: &DriverId,
) -> Result<Vec<VehicleCategory>, AppError> {
    let key = driver_opted_categories_key(driver_id);
    let fields = redis
        .hkeys(&key)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;
    Ok(fields
        .into_iter()
        .filter_map(|f| f.parse::<VehicleCategory>().ok())
        .collect())
}
