use dashmap::DashMap;
use fred::types::geo::{GeoPosition, GeoValue};
use redis_store::{RedisConnectionPool, r_types::*};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::Receiver;
use tokio::time::interval;
use tracing::{error, info};
use utils::executor::Executor;

use super::read_writer::drain_driver_locations;
use crate::helper::create_bucket_key;
use crate::{
    cache::keys::on_driver_location_key,
    types::{self, DriverId, DriverLocationEvent, LocationProfile, TimeStamp},
};

pub struct RideLocationInfoDrainner {
    pub executor: Executor,
    pub delay: i64,
    pub capacity: usize,
    pub nearby_driver_threshold: i64,
    pub map_size: i64,
}

impl RideLocationInfoDrainner {
    pub async fn run(
        &self,
        rx: Receiver<DriverLocationEvent>,
        redis: Arc<RedisConnectionPool>,
    ) {
        let future = Self::ride_location_info_drainner(
            rx,
            Arc::clone(&redis),
            self.delay,
            self.capacity,
            self.nearby_driver_threshold,
            self.map_size,
        );

        self.executor.spawn_detached_task(future);
    }

    async fn ride_location_info_drainner(
        mut rx: Receiver<DriverLocationEvent>,
        redis: Arc<RedisConnectionPool>,
        delay: i64,
        capacity: usize,
        nearby_driver_threshold: i64,
        map_size: i64,
    ) {
        let mut driver_locations_map: DriverLocationMap = DashMap::default();
        let mut timer = interval(Duration::from_secs(delay as u64));
        let mut drainer_size = 0;

        let exp = map_size * nearby_driver_threshold;

        loop {
            tokio::select! {
              incoming_location = rx.recv() => {
                match incoming_location {
                    Some(DriverLocationEvent {latitude, longitude, driver_id, location_profile: LocationProfile { vehicle_category, created_at }, ..}) => {
                    let DriverId (driver_id) = driver_id;
                    let Latitude(latitude) = latitude;
                    let Longitude(longitude) = longitude;
                    let TimeStamp(time) = created_at;
                    let bucket_key = create_bucket_key(map_size, TimeStamp(time));
                    info!("Draining Location data to redis");
                    driver_locations_map.entry(
                    on_driver_location_key(&bucket_key, &vehicle_category)
                    )
                    .or_default()
                    .push(GeoValue {
                    member: driver_id.into(),
                    coordinates: GeoPosition {
                        latitude,
                        longitude,
                    }
                    });

                    drainer_size += 1;

                    if drainer_size >= capacity {
                    info!(tag = "[Force Draining Driver Locations to Redis]", length = %drainer_size);
                    Self::empty_driver_locations(&exp, &driver_locations_map, &redis).await;
                    Self::cleanup(&mut drainer_size, &mut driver_locations_map);
                    }
                    },
                    None => {},
                }
              },
              _ = timer.tick() => {
                if drainer_size > 0 {
                    info!(tag = "[Draining Driver Locations to Redis]", length = %drainer_size);
                    Self::empty_driver_locations(&exp, &driver_locations_map, &redis).await;
                    Self::cleanup(&mut drainer_size, &mut driver_locations_map);
                }
              }
            }
        }
    }

    fn cleanup(
        drainer_size: &mut usize,
        driver_locations_map: &mut types::DriverLocationMap,
    ) {
        *drainer_size = 0;
        *driver_locations_map = DashMap::default();
    }

    async fn empty_driver_locations(
        exp: &i64,
        driver_locations_map: &DashMap<String, Vec<GeoValue>>,
        redis: &RedisConnectionPool,
    ) {
        if let Err(err) =
            drain_driver_locations(redis, driver_locations_map, exp).await
        {
            error!(tag = "[Error Pushing To Redis]", error = %err);
        }
    }
}
