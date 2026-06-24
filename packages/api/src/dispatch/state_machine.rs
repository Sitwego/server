use chrono::Utc;
use num_traits::ToPrimitive;
use redis_store::{
    RedisConnectionPool,
    events::{
        EventsManger, NEXT_DRIVER_OFFERS_STREAM, NextDriverOfferEvent,
        NextDriverOfferEventPayload,
    },
    r_types::{AppError, GeoPoint, Radius},
};
use rust_decimal::Decimal;
use sea_orm::{ActiveValue, Iterable};
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::{collections::HashSet, sync::Arc};
use time::OffsetDateTime;
use tokio::{
    sync::broadcast,
    time::{Duration, Instant},
};
use tracing::{error, info, warn};
use utils::{
    collections::HashMap,
    gen_strings::{self, cal_hash, ulid_string},
};
type RideResultOptions = Option<(Vec<(f64, f64)>, f64, u64)>;
type RideResult = utils::Result<RideResultOptions>;
pub type RideSearchResult =
    Option<(Vec<(f64, f64)>, f64, u64, GeoPoint, GeoPoint)>;

use crate::{
    APIContext,
    api::{
        nearby_drivers::DriverCurrentLocationInfo,
        ride_request::{RequestDriver, RequestRideData, RiderDataInfo},
    },
    cache::read_writer::{
        get_all_drivers_last_locations, get_drivers_max_radii,
        search_for_nearby_drivers_within_radius,
    },
    dispatch::dispatch::DriverPoolManager,
    helper::{create_bucket_key, ttl_to_datetime},
    queries::{
        driver_stats::DriverStatsQueries, drivers::DriverQueries,
        get_fair_estimates::GetFairEstimates, ride::RideQueries,
    },
    schemas::{driver_stats, location, ride_request},
    types::{DriverId, NearbyDriverWithStat, TimeStamp, VehicleCategory},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RideRequestNotification {
    id: String,
    from: RequestRideData,
    to: RequestRideData,
    fare: i32,
    distance: f64,
    vc: Option<VehicleCategory>,
    duration: i32,
    distance_to_pickup: f64,
    duration_to_pickup: f64,
    driver_to_pickup_line_str: Option<Vec<(f64, f64)>>,
    ride_line_str: Option<Vec<(f64, f64)>>,
    msg: Option<String>,
    search_request_id: String,
    search_request_valid_till: OffsetDateTime,
    estimated_start_time: Option<OffsetDateTime>,
    rider_info: Option<RiderDataInfo>,
}

#[derive(Debug, Clone)]
pub enum DispatchState {
    Idle,
    Searching {
        radius_km: f64,
    },
    AcquiringDriver,
    OfferingRide {
        driver_id: String,
        deadline: Instant,
        offer_sent_at: Instant,
        distance_km: f64,
        coordinates: GeoPoint,
        vc: Option<VehicleCategory>,
    },
}

#[derive(Debug)]
pub enum DispatchResult {
    Success { driver_id: String, distance_km: f64 },
    NoDriversAvailable,
    MaxAttemptsReached,
    DeadlineExceeded,
    ChannelClosed,
    CanceledRequest,
}

#[derive(Debug, Clone)]
pub enum DispatchEvent {
    DriverResponse(DriverResponse),
    CancelRequest,
}

pub struct DispatchStateMachine {
    pub state: DispatchState,
    pub event_receiver: broadcast::Receiver<DispatchEvent>,
    pub request_id: String,
    pub ttl: Duration,
    pub mgr: Arc<DriverPoolManager>,
    pub pickup_location: GeoPoint,
    pub vehicle_type: Option<Vec<VehicleCategory>>,
    pub api_ctx: Arc<APIContext>,
    pub ride_request: RequestDriver,
    pub rider_id: String,
    pub ride_data: RideSearchResult,
    pub ranked_candidates: Vec<DriverCandidate>,
    pub otp: String,
    dispatch_deadline: Instant,
    max_attempts: usize,
    attempts: usize,
    current_radius: f64,
    max_radius: f64,
    cancelled: bool, // Track if request was cancelled
}

#[derive(Debug, Clone)]
pub struct DriverResponse {
    pub driver_id: String,
    pub accepted: bool,
    pub response_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct DriverCandidate {
    pub driver_id: String,
    pub score: f64,
    pub distance_km: f64,
    pub vehicle_category: VehicleCategory,
    pub coords: GeoPoint,
}

#[derive(Debug, Clone)]
pub struct DriverAcquisitionResult {
    pub driver_id: String,
    pub distance_km: f64,
    pub score: f64,
    pub vehicle_category: VehicleCategory,
    pub coords: GeoPoint,
}

impl DispatchStateMachine {
    /// Creates a new DispatchStateMachine instance.
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        mgr: Arc<DriverPoolManager>,
        request_id: String,
        ttl_ms: u64,
        rx: broadcast::Receiver<DispatchEvent>,
        max_attempts: usize,
        dispatch_timeout_secs: u64,
        pickup_location: GeoPoint,
        vc: Option<Vec<VehicleCategory>>,
        api_ctx: Arc<APIContext>,
        ride_request: RequestDriver,
        rider_id: String,
        ride_data: RideSearchResult,
    ) -> Self {
        let now = Instant::now();
        let otp = gen_strings::generate_otp(4);

        DispatchStateMachine {
            state: DispatchState::Idle,
            otp,
            api_ctx,
            ride_request,
            rider_id,
            ride_data,
            ranked_candidates: Vec::new(),
            event_receiver: rx,
            request_id,
            ttl: Duration::from_millis(ttl_ms),
            mgr,
            pickup_location,
            vehicle_type: vc,
            dispatch_deadline: now + Duration::from_secs(dispatch_timeout_secs),
            max_attempts,
            attempts: 0,
            current_radius: 2_000.0, // Start with 2km in meters
            max_radius: 20_000.0,    // Max 20km in meters
            cancelled: false,
        }
    }

    /// Cleanup current driver if in OfferingRide state
    async fn cleanup_current_offer(&mut self, penalty: Option<f64>) {
        self.cancelled = true;
        if let DispatchState::OfferingRide { driver_id, .. } = &self.state {
            warn!(
                request_id = %self.request_id,
                driver_id = %driver_id,
                "Cleaning up current offer"
            );
            let penalty = penalty.unwrap_or(0.0);

            if let Err(e) =
                self.mgr.release_driver(driver_id, penalty, 500).await
            {
                error!(
                    request_id = %self.request_id,
                    driver_id = %driver_id,
                    error = ?e,
                    "Failed to release driver during cleanup"
                );
            }
        }
    }

    pub async fn search_nearby_drivers(
        &mut self,
        pickup_location: GeoPoint,
        radius: Radius,
    ) -> Result<Option<bool>, AppError> {
        let vehicle_type = &self.vehicle_type;
        info!(
            request_id = %self.request_id,
            pickup = ?pickup_location,
            vehicle_type = ?vehicle_type,
            radius = ?radius,
            "Starting geo-aware driver acquisition"
        );

        let mut nearby_drivers = find_nearest_driver(
            self.mgr.redis.clone(),
            pickup_location,
            vehicle_type,
            &radius,
        )
        .await?;

        if nearby_drivers.is_empty() {
            warn!(
                request_id = %self.request_id,
                "No nearby drivers found"
            );
            return Ok(None);
        }

        info!(
            request_id = %self.request_id,
            nearby_count = nearby_drivers.len(),
            "Found nearby drivers"
        );
        // remove dublicates while preserving order in nearby drivers
        let mut seen = HashSet::new();
        nearby_drivers
            .retain(|driver| seen.insert(driver.driver_id.0.to_string()));

        // Filter out drivers whose personal radius preference excludes this pickup.
        // `distance` is in metres from the geo search; `max_ride_radius_km` is in km.
        // Drivers with no stored preference fall back to DEFAULT_DRIVER_RADIUS_M (2 km).
        const DEFAULT_DRIVER_RADIUS_M: f64 = 2_000.0;
        let driver_ids_typed: Vec<DriverId> =
            nearby_drivers.iter().map(|d| d.driver_id.clone()).collect();
        let max_radii =
            get_drivers_max_radii(&self.mgr.redis, &driver_ids_typed).await;
        nearby_drivers.retain(|d| {
            let limit_m = max_radii
                .get(&d.driver_id.0)
                .map(|&km| km * 1_000.0)
                .unwrap_or(DEFAULT_DRIVER_RADIUS_M);
            d.distance <= limit_m
        });
        if nearby_drivers.is_empty() {
            warn!(
                request_id = %self.request_id,
                "All nearby drivers are outside their personal radius preference"
            );
            return Ok(None);
        }

        // Create ranked candidates
        let mut ranked_candidates =
            self.drivers_candidates_with_scores(&nearby_drivers).await;

        // Sort candidates by score (lower is better)
        ranked_candidates.sort_by(|a, b| {
            a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal)
        });

        #[cfg(debug_assertions)]
        {
            println!("🏆 Top 5 drivers (best to worst):");
            for (i, candidate) in ranked_candidates.iter().enumerate() {
                println!(
                    "  {}. Score: {:.3} | Driver ID: {:?}",
                    i + 1,
                    candidate.score,
                    candidate.driver_id
                );
            }
        }

        // List expiry to be slightly longer than offer expiry to allow for acquisition attempts
        // This is to ensure that we have the candidate list available in Redis while we are trying to acquire drivers and offering the ride, and also to allow for retries in case of transient errors during acquisition or offering.
        let candidate_list_ttl_seconds: i64 = (self.ttl.as_secs() + 300) as i64; // Add 3 minutes buffer to offer TTL

        //lets add candidates to the head of a sorted list possibly using rpush comand in redis and then try
        // to acquire drivers one by one using the DriverPoolManager until we get a successful acquisition or
        // exhaust the list. This way we can ensure that we are trying to acquire drivers in the order of their scores.
        if let Err(e) = self
            .api_ctx
            .redis
            .rpush_with_expiration(
                &crate::cache::keys::request_candidate_list_key(
                    &self.request_id,
                ),
                ranked_candidates
                    .iter()
                    .map(|c| c.driver_id.to_string())
                    .collect::<Vec<String>>(),
                &candidate_list_ttl_seconds,
            )
            .await
        {
            error!(
                request_id = %self.request_id,
                error = ?e,
                "Failed to push candidate list to Redis"
            );
        }

        self.ranked_candidates = ranked_candidates;

        Ok(Some(true))
    }

    async fn pop_next_candidate(&mut self) -> Option<String> {
        let key =
            crate::cache::keys::request_candidate_list_key(&self.request_id);
        match self.api_ctx.redis.lpop::<String>(&key, None).await {
            Ok(candidates) if !candidates.is_empty() => {
                let driver_id = &candidates[0];
                info!(
                    request_id = %self.request_id,
                    driver_id = %driver_id,
                    "Popped next candidate for acquisition"
                );
                Some(driver_id.to_string())
            }
            Ok(_) => {
                // List was empty (drained between checks or key expired)
                None
            }
            Err(e) => {
                error!(
                    request_id = %self.request_id,
                    error = ?e,
                    "Failed to pop next candidate from Redis"
                );
                None
            }
        }
    }

    async fn acquire_driver(
        &mut self,
        driver_id: &str,
    ) -> Result<Option<DriverAcquisitionResult>, AppError> {
        let ttl_ms = self.ttl.as_millis();
        let args = vec![
            self.request_id.to_string(),
            ttl_ms.to_string(),
            driver_id.to_string(),
        ];
        let ranked_candidates = &self.ranked_candidates;
        match self.mgr.try_acquire_driver(&args).await {
            Ok(Some(dr)) => {
                info!(
                    request_id = %self.request_id,
                    driver_id = %dr,
                    "✅ Driver acquired successfully"
                );
                // Find the driver in ranked candidates
                if let Some(rcn) =
                    ranked_candidates.iter().find(|c| c.driver_id == dr)
                {
                    return Ok(Some(DriverAcquisitionResult {
                        driver_id: rcn.driver_id.to_string(),
                        distance_km: rcn.distance_km,
                        score: rcn.score,
                        vehicle_category: rcn.vehicle_category,
                        coords: rcn.coords,
                    }));
                } else {
                    error!(
                        request_id = %self.request_id,
                        driver_id = %dr,
                        "Acquired driver not found in candidates"
                    );
                    return Err(AppError::InternalError(
                        "Acquired driver not found in candidates".to_string(),
                    ));
                }
            }
            Ok(None) => {
                warn!(
                    request_id = %self.request_id,
                    "No drivers could be acquired"
                );
            }
            Err(er) => {
                warn!(
                    request_id = %self.request_id,
                    "No drivers could be acquired after trying all candidates"
                );
                return Err(AppError::InternalError(format!(
                    "Failed to acquire driver: {:?}",
                    er
                )));
            }
        }

        Ok(None)
    }

    async fn dispatch_offer_to_driver(
        &mut self,
        result: DriverAcquisitionResult,
    ) -> Result<DispatchState, AppError> {
        self.attempts += 1;
        let now = Instant::now();

        info!(
            request_id = %self.request_id,
            driver_id = %result.driver_id,
            distance_km = %result.distance_km,
            "✅Offering ride to driver🚕🚓"
        );

        // Fetch driver info
        //TODO:: Optimization: Cache driver info to reduce DB calls here
        let driver_info =
            self.api_ctx.db.get_driver_info(&result.driver_id).await?;
        // handle case where driver info is not found in DB
        if driver_info.is_none() {
            // return Apperror or log and return early from this function
            error!(
                request_id = %self.request_id,
                driver_id = %result.driver_id,
                "Driver info not found in DB, cannot send offer"
            );
            return Err(AppError::NotFound(
                "Driver info not found".to_string(),
            ));
        }
        let driver_info = driver_info.unwrap();

        let events_mgr = EventsManger::new(NEXT_DRIVER_OFFERS_STREAM);
        // get eta and assign driver
        let driver_to_pick_up_eta =
            match get_eta(self.pickup_location, result.coords).await {
                Ok(eta) => eta,
                Err(e) => {
                    error!(
                        request_id = %self.request_id,
                        error = ?e,
                        "Failed to get ETA, continuing without it"
                    );
                    None
                }
            };

        let (t, dx, dt) = if let Some((line, dx, dt)) = driver_to_pick_up_eta {
            (line, dx, dt)
        } else {
            (vec![], 0.0, 0)
        };

        // Persist this candidate's driver->pickup geometry (as computed:
        // pickup->driver order) so the accept handler can expose it to the
        // customer as p2 (approach-phase marker). Stored by reference here —
        // `t` is still moved into the notification below. Keyed by
        // (ride, driver) so an offer to a non-accepting candidate never
        // shadows the accepting driver's leg. Best-effort.
        if !t.is_empty() {
            let d2p_key = crate::cache::keys::driver_to_pickup_path_key(
                &crate::RideId(self.request_id.clone()),
                &crate::DriverId(result.driver_id.clone()),
            );
            if let Err(e) = self
                .api_ctx
                .redis
                .set_key(&d2p_key, &t, self.api_ctx.config.exp_ttl)
                .await
            {
                error!(
                    request_id = %self.request_id,
                    driver_id = %result.driver_id,
                    error = ?e,
                    "Failed to persist driver->pickup path for accept"
                );
            }
        }

        let estimated_ride_start_time =
            OffsetDateTime::now_utc() + time::Duration::milliseconds(dt as i64);

        let (ride_path, ride_dx, ride_dt, _, _) =
            self.ride_data.to_owned().expect("This can not be null");
        let notification = RideRequestNotification {
            id: ulid_string(),
            from: self.ride_request.from.clone(),
            to: self.ride_request.to.clone(),
            fare: self.ride_request.fare,
            distance: ride_dx,
            vc: Some(result.vehicle_category),
            duration: ride_dt as i32,
            distance_to_pickup: dx,
            duration_to_pickup: dt as f64 / 60.0,
            driver_to_pickup_line_str: Some(t),
            ride_line_str: Some(ride_path),
            msg: None,
            rider_info: Some(self.ride_request.rider_profile.clone()),
            search_request_id: self.request_id.to_string(),
            search_request_valid_till: OffsetDateTime::now_utc()
                + time::Duration::milliseconds(self.ttl.as_millis() as i64),
            estimated_start_time: Some(estimated_ride_start_time),
        };

        let shard = (cal_hash(&result.driver_id) % 128) as u64;
        let notif_ttl = (Utc::now()
            + chrono::Duration::milliseconds(self.ttl.as_millis() as i64))
        .to_rfc3339();

        let stream_data: smallvec::SmallVec<[(String, String); 8]> = smallvec![
            ("type".to_string(), "RideRequest".to_string()),
            ("category".to_string(), "NEW_RIDE_AVAILABLE".to_string()),
            ("ttl".to_string(), notif_ttl.to_owned()),
            ("id".to_string(), self.request_id.to_string()),
            (
                "data".to_string(),
                serde_json::to_string(&notification).unwrap() // Serialize notification data
            )
        ];
        let offering_ride = DispatchState::OfferingRide {
            driver_id: result.driver_id.to_string(),
            deadline: now + self.ttl,
            offer_sent_at: Instant::now(),
            distance_km: result.distance_km,
            coordinates: result.coords,
            vc: Some(result.vehicle_category),
        };

        // Create ride request in DB
        let model = ride_request::Model {
            id: self.request_id.to_string(),
            driver_id: result.driver_id.to_string(),
            customer_id: self.rider_id.to_string(),
            fare: Decimal::new(self.ride_request.fare as i64, 0),
            estimated_distance_to_pickup: Some(dx),
            otp: Some(self.otp.to_string()),
            estimated_duration_to_pickup: Some(dt as i32),
            estimated_distance: Some(ride_dx),
            estimated_duration: Some(ride_dt as i32),

            search_request_valid_till: Some(
                ttl_to_datetime(&notif_ttl).unwrap(),
            ),
            start_time: Some(estimated_ride_start_time),
            created_at: Utc::now(),
            ..Default::default()
        };

        // Push notification to Redis stream
        let notif_key =
            crate::cache::keys::notification_key(&result.driver_id, shard);
        if let Err(e) = self
            .api_ctx
            .redis
            .xadd(&notif_key, stream_data.to_vec(), 1000)
            .await
        {
            error!(
                request_id = %self.request_id,
                driver_id = %result.driver_id,
                error = ?e,
                "Failed to push ride request notification"
            );
        } else {
            warn!(
                request_id = %self.request_id,
                driver_id = %result.driver_id,
                "✅✅Ride request notification pushed"
            );
        }

        if let Err(e) = self.save_ride_request(model).await {
            error!(
                request_id = %self.request_id,
                driver_id = %result.driver_id,
                error = ?e,
                "Failed to save ride request in DB"
            );
        }

        let driver_name = format!(
            "{} {}.",
            driver_info.first_name.as_deref().unwrap_or("First"),
            driver_info
                .last_name
                .as_ref()
                .and_then(|s| s.chars().next())
                .unwrap_or('L')
        );
        let driver_avatar =
            format!("{}/{}", result.driver_id, driver_info.photo_id,);

        // Price this driver's approach leg so the rider sees the pickup fare
        // with the offer. dx is metres, dt seconds (OSRM units from get_eta).
        let pickup_fare = self
            .api_ctx
            .db
            .resolve_pickup_fare(
                &result.vehicle_category,
                Some(dx),
                Some(dt as i32),
            )
            .await;

        // publish ride request to rider next_driver_offers_events channel
        if let Err(e) = events_mgr
            .publish_event(
                Some(&NextDriverOfferEvent {
                    event_id: ulid_string(),
                    timestamp: Utc::now().timestamp_millis(),
                    ride_request_id: self.request_id.to_string(),
                    status: 2,
                    payload: Some(NextDriverOfferEventPayload {
                        rider_id: self.rider_id.to_string(),
                        driver_image: driver_avatar,
                        driver_name,
                        driver_rating: num_traits::ToPrimitive::to_f64(
                            &driver_info.rating.unwrap_or(Decimal::new(0, 0)),
                        )
                        .unwrap_or(0.0),
                        ride_id: self.request_id.to_string(),
                        latitude: result.coords.lat.0,
                        longitude: result.coords.lon.0,
                        distance_km: dx,
                        estimated_arrival_time: dt as f64, // driver arrival at pickup time
                        pickup_fare,
                        timestamp: now.elapsed().as_millis() as i64,
                    }),
                }),
                &self.api_ctx.redis,
            )
            .await
        {
            error!(
                request_id = %self.request_id,
                driver_id = %result.driver_id,
                error = ?e,
                "Failed to publish ride request event"
            );
        } else {
            info!(
                tag = "next_driver_offers_event",
                request_id = %self.request_id,
                driver_id = %result.driver_id,
                "✅✅Ride request event published to rider"
            );
        }
        Ok(offering_ride)
    }

    /// Get stats for a list of driver IDs
    async fn get_driver_stats(
        &self,
        driver_ids: &[String],
    ) -> HashMap<String, driver_stats::Model> {
        let driver_stats =
            self.api_ctx.db.get_drivers_stats(driver_ids.to_vec()).await;
        match driver_stats {
            Ok(stats) => stats,
            Err(err) => {
                error!(
                    request_id = %self.request_id,
                    error = ?err,
                    "Failed to get driver stats"
                );
                HashMap::new()
            }
        }
    }

    async fn drivers_candidates_with_scores(
        &self,
        drivers: &[DriverCurrentLocationInfo],
    ) -> Vec<DriverCandidate> {
        let mut candidates: Vec<DriverCandidate> = Vec::new();
        let now = Utc::now();
        let driver_ids = drivers
            .iter()
            .map(|d| d.driver_id.0.to_owned())
            .collect::<Vec<String>>();
        let driver_stats = self.get_driver_stats(&driver_ids).await;
        for driver in drivers {
            let (dx, rating_sc, acceptance, total_ernings, fatigue, idle_time) =
                match driver_stats.get(&driver.driver_id.0.to_string()) {
                    Some(stat) => {
                        let acceptance = calculate_acceptance_rate(
                            stat.total_rides_assigned.unwrap_or(0),
                            stat.rides_cancelled.unwrap_or(0),
                        );

                        let fatigue = calculate_fatigue(stat.total_rides, 50.0)
                            .clamp(0.0, 1.0);
                        let idle_time = ((now - stat.idle_since.to_utc())
                            .num_seconds()
                            as f64)
                            / 3600.0;

                        let total_rating_score = stat
                            .rating
                            .map(|r| r.to_f64().unwrap_or(0.0))
                            .unwrap_or(0.0);

                        (
                            driver.distance,
                            total_rating_score,
                            acceptance,
                            stat.total_earnings,
                            fatigue,
                            idle_time,
                        )
                    }
                    None => {
                        warn!(
                            request_id = %self.request_id,
                            driver_id = %driver.driver_id.0,
                            "No stats found for driver"
                        );
                        (driver.distance, 0.0, 0.8, 0.0, 0.0, 0.0)
                    }
                };
            let dx = dx / 1000.0; // convert to km
            // Process driver with stats
            let score = calculate_cost(&NearbyDriverWithStat {
                driver_id: driver.driver_id.clone(),
                geo_point: driver.coords,
                distance: crate::types::Meters(dx),
                total_rating_score: rating_sc,
                acceptance,
                earnings: total_ernings,
                fatigue,
                idle_time,
            });
            candidates.push(DriverCandidate {
                driver_id: driver.driver_id.0.to_string(),
                score,
                distance_km: dx,
                vehicle_category: driver.vehicle_category,
                coords: driver.coords,
            });
        }

        candidates
    }

    /// Run the dispatch state machine
    pub async fn run(mut self) -> DispatchResult {
        loop {
            // Hard stop: global dispatch timeout
            if Instant::now() >= self.dispatch_deadline {
                warn!(
                    request_id = %self.request_id,
                    "Dispatch deadline exceeded"
                );
                // publish event that dispatch failed due to deadline exceeded
                let event_msg = NextDriverOfferEvent {
                    event_id: ulid_string(),
                    timestamp: Utc::now().timestamp_millis(),
                    ride_request_id: self.request_id.to_string(),
                    status: 1, // 1 indicates deadline expired
                    payload: None,
                };
                let events_mgr = EventsManger::new(NEXT_DRIVER_OFFERS_STREAM);
                if let Err(e) = events_mgr
                    .publish_event(Some(&event_msg), &self.api_ctx.redis)
                    .await
                {
                    error!(
                        request_id = %self.request_id,
                        error = ?e,
                        "Failed to publish dispatch deadline exceeded event"
                    );
                }
                // Cleanup current offer if any
                self.cleanup_current_offer(Some(0.5)).await;
                return DispatchResult::DeadlineExceeded;
            }

            match &self.state {
                DispatchState::Idle => {
                    info!(request_id = %self.request_id, "Starting dispatch");
                    self.state = DispatchState::Searching {
                        radius_km: self.current_radius,
                    };
                }

                DispatchState::Searching { radius_km } => {
                    info!(
                        request_id = %self.request_id,
                        attempts = self.attempts,
                        max_attempts = self.max_attempts,
                        "Processing dispatch request"
                    );

                    if self.attempts >= self.max_attempts {
                        warn!(
                            request_id = %self.request_id,
                            "Max attempts reached"
                        );
                        let event_msg = NextDriverOfferEvent {
                            event_id: ulid_string(),
                            timestamp: Utc::now().timestamp_millis(),
                            ride_request_id: self.request_id.to_string(),
                            status: 1,
                            payload: None,
                        };
                        let events_mgr = EventsManger::new(
                            "rider:next_driver_offers_events",
                        );
                        if let Err(e) = events_mgr
                            .publish_event(
                                Some(&event_msg),
                                &self.api_ctx.redis,
                            )
                            .await
                        {
                            error!(
                                request_id = %self.request_id,
                                error = ?e,
                                "Failed to publish max attempts reached event"
                            );
                        }
                        // Cleanup current offer if any
                        self.cleanup_current_offer(None).await;
                        return DispatchResult::MaxAttemptsReached;
                    }

                    // Searching for drivers within Radius
                    match self
                        .search_nearby_drivers(
                            self.pickup_location,
                            Radius(*radius_km),
                        )
                        .await
                    {
                        Ok(Some(res)) => {
                            if res {
                                self.state = DispatchState::AcquiringDriver;
                            } else {
                                warn!(
                                    request_id = %self.request_id,
                                    "No drivers available to acquire"
                                );
                                return DispatchResult::NoDriversAvailable;
                            }
                        }
                        Ok(None) => {
                            if self.current_radius < self.max_radius {
                                self.current_radius += 3000.0; // Expand by 3km
                                info!(
                                    request_id = %self.request_id,
                                    new_radius = %self.current_radius,
                                    "Expanding search radius"
                                );
                                self.state = DispatchState::Searching {
                                    radius_km: self.current_radius,
                                };
                            } else {
                                warn!(
                                    request_id = %self.request_id,
                                    "No drivers within max radius"
                                );
                                return DispatchResult::NoDriversAvailable;
                            }
                        }
                        Err(_) => {
                            // Error during driver acquisition - log and retry after short delay
                            error!(
                                request_id = %self.request_id,
                                "Error during driver acquisition, retrying..."
                            );
                            tokio::time::sleep(Duration::from_millis(500))
                                .await;
                        }
                    }
                }

                DispatchState::AcquiringDriver => {
                    match self.pop_next_candidate().await {
                        Some(driver_id) => {
                            info!(
                                request_id = %self.request_id,
                                driver_id = %driver_id,
                                "Trying to acquire candidate"
                            );
                            // Try to acquire the popped driver
                            match self.acquire_driver(&driver_id).await {
                                Ok(Some(acquisition_result)) => {
                                    info!(
                                        request_id = %self.request_id,
                                        driver_id = %acquisition_result.driver_id,
                                        "Driver acquired successfully, moving to offering state"
                                    );
                                    match self
                                        .dispatch_offer_to_driver(
                                            acquisition_result,
                                        )
                                        .await
                                    {
                                        Ok(resp) => {
                                            info!(
                                                "Ride offer process completed"
                                            );
                                            self.state = resp;
                                        }
                                        Err(e) => {
                                            error!(
                                                "Error during ride offer process: {:?}",
                                                e
                                            );
                                            // Continue trying next candidates
                                            self.state =
                                                DispatchState::AcquiringDriver;
                                        }
                                    }
                                }
                                Ok(None) => {
                                    warn!(
                                        request_id = %self.request_id,
                                        driver_id = %driver_id,
                                        "Failed to acquire driver, trying next candidate"
                                    );
                                    // Stay in AcquiringDriver to pop next
                                }
                                Err(e) => {
                                    error!(
                                        request_id = %self.request_id,
                                        driver_id = %driver_id,
                                        error = ?e,
                                        "Error while trying to acquire driver"
                                    );
                                    // Stay in AcquiringDriver to pop next
                                }
                            }
                        }
                        None => {
                            // Candidate list exhausted — expand radius or give up
                            warn!(
                                request_id = %self.request_id,
                                "No more candidates left to acquire"
                            );
                            if self.current_radius < self.max_radius {
                                self.current_radius += 3000.0; // Expand by 3km
                                info!(
                                    request_id = %self.request_id,
                                    new_radius = %self.current_radius,
                                    "Expanding search radius after exhausting candidates"
                                );
                                self.state = DispatchState::Searching {
                                    radius_km: self.current_radius,
                                };
                            } else {
                                warn!(
                                    request_id = %self.request_id,
                                    "No drivers within max radius after exhausting candidates"
                                );
                                return DispatchResult::NoDriversAvailable;
                            }
                        }
                    }
                }
                DispatchState::OfferingRide {
                    driver_id,
                    deadline,
                    offer_sent_at,
                    distance_km,
                    coordinates,
                    vc,
                } => {
                    let remaining =
                        deadline.saturating_duration_since(Instant::now());

                    warn!(
                        request_id = %self.request_id,
                        driver_id = %driver_id,
                        vehicle_category = ?vc,
                        location = ?coordinates,
                        remaining_ms = remaining.as_millis(),
                        "Waiting for driver response"
                    );

                    let current_driver_id = driver_id.clone();
                    let offer_time = *offer_sent_at;
                    let dist = *distance_km;

                    tokio::select! {
                        biased;

                        event = self.event_receiver.recv() => {
                            match event {
                                Ok(DispatchEvent::CancelRequest) => {
                                    // Rider canceled the request
                                    info!(
                                        request_id = %self.request_id,
                                        driver_id = %current_driver_id,
                                        "Ride request canceled by rider"
                                    );
                                    self.cleanup_current_offer(None).await;

                                    return DispatchResult::CanceledRequest;
                                }
                                Ok(DispatchEvent::DriverResponse(response)) if response.driver_id == current_driver_id => {
                                    let response_time = Instant::now().duration_since(offer_time);

                                    if response.accepted {
                                        // Driver accepted: confirm the assignment (removes from
                                        // inflight without putting driver back in the pool).
                                        if let Err(e) = self
                                            .mgr
                                            .confirm_driver(&current_driver_id)
                                            .await
                                        {
                                            error!(
                                                request_id = %self.request_id,
                                                driver_id = %current_driver_id,
                                                error = ?e,
                                                "Failed to confirm driver assignment"
                                            );
                                        }

                                        return DispatchResult::Success {
                                            driver_id: current_driver_id,
                                            distance_km: dist,
                                        };
                                    } else {
                                        info!(
                                            request_id = %self.request_id,
                                            driver_id = %current_driver_id,
                                            response_time_ms = response_time.as_millis(),
                                            "Driver rejected ride"
                                        );

                                        if let Err(e) = self.mgr.release_driver(
                                            &current_driver_id,
                                            1.0,
                                            5000
                                        ).await {
                                            error!(
                                                request_id = %self.request_id,
                                                driver_id = %current_driver_id,
                                                error = ?e,
                                                "Failed to release driver after rejection"
                                            );
                                        }

                                        self.state = DispatchState::Searching {
                                            radius_km: self.current_radius,
                                        };
                                    }
                                }
                                Ok(DispatchEvent::DriverResponse(response)) => {
                                    // Stale response from different driver - ignore
                                    warn!(
                                        request_id = %self.request_id,
                                        expected_driver = %current_driver_id,
                                        received_driver = %response.driver_id,
                                        "Received stale driver response"
                                    );
                                }
                                Err(broadcast::error::RecvError::Closed) => {
                                    error!(
                                        request_id = %self.request_id,
                                        "Event channel closed"
                                    );
                                    self.cleanup_current_offer(None).await;
                                    return DispatchResult::ChannelClosed;
                                }
                                Err(broadcast::error::RecvError::Lagged(n)) => {
                                    warn!(
                                        request_id = %self.request_id,
                                        skipped = n,
                                        "Event receiver lagged, some events were dropped"
                                    );
                                }
                            }
                        }

                        _ = tokio::time::sleep(remaining) => {
                            warn!(
                                request_id = %self.request_id,
                                driver_id = %current_driver_id,
                                "Offer window expired, entering grace period"
                            );

                            // Grace period: driver's HTTP accept may still be in-flight
                            // (mobile network latency, connection retries).  Loop for the
                            // full window so stale / cancel events don't cut it short.
                            let grace_end = Instant::now() + Duration::from_secs(15);
                            let accepted_in_grace = 'grace: loop {
                                let remaining_grace =
                                    grace_end.saturating_duration_since(Instant::now());
                                if remaining_grace.is_zero() {
                                    break false;
                                }
                                tokio::select! {
                                    biased;
                                    event = self.event_receiver.recv() => {
                                        match event {
                                            Ok(DispatchEvent::DriverResponse(r))
                                                if r.driver_id == current_driver_id
                                                    && r.accepted =>
                                            {
                                                break 'grace true;
                                            }
                                            Ok(DispatchEvent::CancelRequest) => {
                                                // Rider cancelled during grace period
                                                break 'grace false;
                                            }
                                            Ok(_) => {
                                                // Stale event — keep waiting
                                                continue;
                                            }
                                            Err(broadcast::error::RecvError::Closed) => {
                                                // Channel closed
                                                break 'grace false;
                                            }
                                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                                // Missed some messages — keep waiting
                                                continue;
                                            }
                                        }
                                    }
                                    _ = tokio::time::sleep(remaining_grace) => {
                                        break 'grace false;
                                    }
                                }
                            };

                            if accepted_in_grace {
                                warn!(
                                    request_id = %self.request_id,
                                    driver_id = %current_driver_id,
                                    "Driver accepted during grace period"
                                );
                                if let Err(e) = self.mgr.confirm_driver(&current_driver_id).await {
                                    error!(
                                        request_id = %self.request_id,
                                        driver_id = %current_driver_id,
                                        error = ?e,
                                        "Failed to confirm driver during grace period"
                                    );
                                }
                                return DispatchResult::Success {
                                    driver_id: current_driver_id,
                                    distance_km: dist,
                                };
                            }

                            warn!(
                                request_id = %self.request_id,
                                driver_id = %current_driver_id,
                                "Driver response timeout"
                            );

                            if let Err(e) = self.mgr.release_driver(
                                &current_driver_id,
                                1.0,
                                5000
                            ).await {
                                error!(
                                    request_id = %self.request_id,
                                    driver_id = %current_driver_id,
                                    error = ?e,
                                    "Failed to release driver after timeout"
                                );
                            }

                            self.ride_request_timout_hadler(&current_driver_id).await;

                            self.state = DispatchState::AcquiringDriver;
                        }
                    }
                }
            }
        }
    }

    pub async fn start(self) -> DispatchResult {
        self.run().await
    }

    async fn ride_request_timout_hadler(&mut self, driver_id: &str) {
        if let Err(er) = self
            .api_ctx
            .db
            .update_driver_stats_rides_cancelled(driver_id.to_owned(), &1)
            .await
        {
            error!(
                request_id = %self.request_id,
                driver_id = %driver_id,
                error = ?er,
                "Failed to update driver stats for rides cancelled"
            );
        }
    }

    async fn save_ride_request(
        &self,
        m: ride_request::Model,
    ) -> Result<(), AppError> {
        let end_otp = gen_strings::generate_otp(4);
        let from = transform_ride_request_location_data(
            self.ride_request.from.to_owned(),
        );
        let to = transform_ride_request_location_data(
            self.ride_request.to.to_owned(),
        );

        let (from, to) = self
            .api_ctx
            .db
            .create_locations(from, to)
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        self.api_ctx
            .db
            .create_ride_request(m, from, to, end_otp)
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;
        Ok(())
    }
}

async fn get_eta(d: GeoPoint, r: GeoPoint) -> RideResult {
    #[cfg(feature = "reqwest-middleware")]
    {
        use crate::{request::RidesApiClient, simd_json::parse_from_string};

        let base_url = std::env::var("ROUTES_API_URL")
            .expect("ROUTES_API_URL must be set");
        let req_instance = RidesApiClient::new_with_retry(&base_url, None);
        let coords = [(d.lon.0, d.lat.0), (r.lon.0, r.lat.0)];
        let resp = parse_from_string(
            &req_instance
                .get_ride_path_and_distance(
                    &coords,
                    "overview=full&steps=true&geometries=geojson",
                )
                .await?,
        );

        if let Ok(resp) = resp {
            return Ok(Some(resp));
        }
        Ok(None)
    }
}

/// Expands a list of requested vehicle categories into all categories
/// whose drivers are eligible to serve those requests.
/// Bike and Women are exclusive and only match exactly.
pub fn expand_vehicle_categories(
    requested: &[VehicleCategory],
) -> Vec<VehicleCategory> {
    let mut expanded: Vec<VehicleCategory> = requested
        .iter()
        .flat_map(|vc| vc.eligible_serving_categories())
        .collect();
    expanded.sort();
    expanded.dedup();
    expanded
}

pub async fn find_nearest_driver(
    redis: Arc<RedisConnectionPool>,
    coordinate: GeoPoint,
    vehicle_type: &Option<Vec<VehicleCategory>>,
    radius: &Radius,
) -> Result<Vec<DriverCurrentLocationInfo>, AppError> {
    let buckect_key = create_bucket_key(30, TimeStamp(Utc::now()));

    match vehicle_type {
        Some(vehicles) => {
            info!(
                "SEARCHING NEARBY DRIVERS: {:?} {:?} {:?}",
                coordinate, vehicle_type, radius
            );
            let mut response: Vec<DriverCurrentLocationInfo> = Vec::new();
            for vc in vehicles {
                let nearest_drivers = search_for_nearby_drivers_within_radius(
                    &redis,
                    &buckect_key,
                    &4,
                    vc,
                    radius,
                    &coordinate,
                    // region
                )
                .await?;

                let driver_ids = nearest_drivers
                    .iter()
                    .map(|driver| driver.dirver_id.clone())
                    .collect::<Vec<DriverId>>();

                let latest_driver_location =
                    get_all_drivers_last_locations(&redis, &driver_ids).await?;

                let resp = nearest_drivers
                    .iter()
                    .zip(latest_driver_location.iter())
                    .map(|(a, b)| {
                        let timestamp = b.as_ref().map_or_else(
                            || TimeStamp(Utc::now()),
                            |loc| loc.timestamp,
                        );
                        DriverCurrentLocationInfo {
                            coords: a.coords,
                            timestamp,
                            driver_id: a.dirver_id.clone(),
                            distance: a.distance,
                            vehicle_category: *vc,
                        }
                    })
                    .collect::<Vec<DriverCurrentLocationInfo>>();

                response.extend(resp);
            }

            Ok(response)
        }
        None => {
            let mut response: Vec<DriverCurrentLocationInfo> = Vec::new();
            info!(
                "SEARCHING NEARBY DRIVERS WITHOUT VEHICLE TYPE: {:?} {:?}",
                coordinate, radius
            );
            for vc in VehicleCategory::iter() {
                let nearest_drivers = search_for_nearby_drivers_within_radius(
                    &redis,
                    &buckect_key,
                    &4,
                    &vc,
                    radius,
                    &coordinate,
                    // region
                )
                .await?;

                let driver_ids = nearest_drivers
                    .iter()
                    .map(|driver| driver.dirver_id.clone())
                    .collect::<Vec<DriverId>>();

                let latest_driver_location =
                    get_all_drivers_last_locations(&redis, &driver_ids).await?;

                let resp = nearest_drivers
                    .iter()
                    .zip(latest_driver_location.iter())
                    .map(|(a, b)| {
                        let timestamp = b.as_ref().map_or_else(
                            || TimeStamp(Utc::now()),
                            |loc| loc.timestamp,
                        );

                        DriverCurrentLocationInfo {
                            coords: a.coords,
                            timestamp,
                            driver_id: a.dirver_id.clone(),
                            distance: a.distance,
                            vehicle_category: vc,
                        }
                    })
                    .collect::<Vec<DriverCurrentLocationInfo>>();

                response.extend(resp);
            }

            Ok(response)
        }
    }
}

pub fn transform_ride_request_location_data(
    body: RequestRideData,
) -> location::ActiveModel {
    location::ActiveModel {
        id: ActiveValue::Set(ulid_string()),
        lat: ActiveValue::Set(body.geo_point.lat.0),
        lon: ActiveValue::Set(body.geo_point.lon.0),
        street: ActiveValue::Set(body.street),
        city: ActiveValue::Set(body.city),
        road: ActiveValue::Set(body.road),
        country: ActiveValue::Set(body.country),
        building: ActiveValue::Set(body.building),
        floor: ActiveValue::Set(body.floor),
        door: ActiveValue::Set(body.door),
        area_code: ActiveValue::Set(body.area_code),
        ward: ActiveValue::Set(body.ward),
        place_id: ActiveValue::Set(body.place_id),
        instructions: ActiveValue::Set(body.instructions),
        extras: ActiveValue::Set(body.extras),
        ..Default::default()
    }
}

pub fn current_time_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

fn calculate_fatigue(total_rides: i32, threshold: f64) -> f64 {
    let rides = total_rides as f64;

    let fatigue = if rides > threshold {
        // Gentler 1.3 exponent for progressive discouragement
        (rides / threshold).powf(1.3)
    } else {
        rides / threshold
    };

    fatigue.clamp(0.0, 1.0)
}

fn calculate_acceptance_rate(
    total_rides_assigned: i32,
    rides_canceled: i32,
) -> f64 {
    let assigned = total_rides_assigned.max(0);
    let canceled = rides_canceled.max(0);

    if assigned == 0 {
        return 1.0;
    }

    if canceled > assigned {
        #[cfg(debug_assertions)]
        eprintln!(
            "⚠️  Invalid data: {} canceled > {} assigned. Using neutral score.",
            canceled, assigned
        );

        return 0.0;
    }

    ((assigned - canceled) as f64 / assigned as f64).clamp(0.0, 1.0)
}

// Scoring weights - sum to 1.0 for interpretable scores.
// ETA_WEIGHT is reserved for when real ETA data is available; its share has
// been redistributed to distance (most ETA-correlated) and rating (improved
// composite signal from Wilson score).
pub const ETA_WEIGHT: f64 = 0.0; // reserved — not yet active
pub const DISTANCE_WEIGHT: f64 = 0.450; // ~45% - Physical proximity
pub const RATING_WEIGHT: f64 = 0.250; // ~25% - Driver quality (Wilson composite)
pub const IDLE_TIME_WEIGHT: f64 = 0.110; // ~11% - Fairness (waiting time)
pub const ACCEPTANCE_WEIGHT: f64 = 0.090; // ~9%  - Reliability
pub const FATIGUE_WEIGHT: f64 = 0.060; // ~6%  - Driver safety
pub const EARNINGS_WEIGHT: f64 = 0.040; // ~4%  - Load balancing
// Sum: 0.450 + 0.250 + 0.110 + 0.090 + 0.060 + 0.040 = 1.000

// Normalization constants
// const MAX_ETA_SECONDS: f64 = 600.0;          // Cap ETA at 10 minutes
const MAX_EARNINGS_THRESHOLD: f64 = 1000.0; // Earnings threshold for normalization
const DISTANCE_DECAY_FACTOR: f64 = 3.0; // Controls distance penalty curve
const MAX_PICKUP_DISTANCE_KM: f64 = 8.0; // Hard cutoff for pickup distance
const MAX_IDLE_HOURS: f64 = 2.0; // Cap idle time benefit at 2 hours

/// Calculates a cost score for driver assignment (lower score = better match)
///
/// # Scoring Strategy
/// - ETA & Distance dominate pickup experience
/// - Rating & Acceptance protect service quality
/// - Earnings balances load across drivers
/// - Fatigue enforces safety constraints
///
/// # Returns
/// Normalized cost score in range [0.0, 1.0]
pub fn calculate_cost(driver: &crate::types::NearbyDriverWithStat) -> f64 {
    let distance_km = driver.distance.0;
    debug_assert!(distance_km >= 0.0, "Distance must be non-negative");

    // Hard cutoff to avoid absurd assignments
    if distance_km > MAX_PICKUP_DISTANCE_KM {
        return 1.0;
    }

    // ETA normalization (0 = instant, 1 = worst acceptable)
    // let norm_eta = (eta.min(MAX_ETA_SECONDS)) / MAX_ETA_SECONDS;

    // Rating normalization: stat.rating stores the Wilson composite score in
    // [0, 1] (written back by rate_driver_for_ride).  Higher = better driver,
    // so invert to get cost (0 = best, 1 = worst).
    let norm_rating = 1.0 - driver.total_rating_score.clamp(0.0, 1.0);

    // Acceptance normalization (higher acceptance = lower cost)
    let norm_acceptance = 1.0 - driver.acceptance;

    // Earnings normalization (penalize very high earners slightly)
    let norm_earnings = (driver.earnings / MAX_EARNINGS_THRESHOLD).min(1.0);

    // Fatigue normalization (linear penalty up to max fatigue)
    let norm_fatigue = driver.fatigue;

    // Distance cost using exponential decay
    // 0km≈0, 3km≈0.63, 10km≈0.96
    let dist_cost = 1.0 - (-distance_km / DISTANCE_DECAY_FACTOR).exp();

    // Invert so longer idle time gives lower cost (better priority)
    // Cap at MAX_IDLE_HOURS to prevent extreme bias
    // Example: 0 hours = 1.0 (just finished), 2+ hours = 0.0 (waiting long time)
    let normalize_idle_time =
        1.0 - (driver.idle_time / MAX_IDLE_HOURS).min(1.0);

    // Final weighted cost (lower is better)
    let scores = RATING_WEIGHT * norm_rating
        + ACCEPTANCE_WEIGHT * norm_acceptance
        + EARNINGS_WEIGHT * norm_earnings
        + FATIGUE_WEIGHT * norm_fatigue
        + DISTANCE_WEIGHT * dist_cost
        + IDLE_TIME_WEIGHT * normalize_idle_time;
    scores.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use redis_store::r_types::{Latitude, Longitude};

    use crate::types::Meters;

    use super::*;
    //     use crate::dispatch::dispatch::DriverPoolManager;
    //     use redis_store::{RedisConfig, RedisConnectionPool};
    //     use tokio::sync::mpsc;
    //     use tracing_test::traced_test;

    //     //setup test Redis connection
    //     async fn setup() -> DriverPoolManager {
    //         let config = RedisConfig {
    //             host: "".to_string(),
    //             port: 6379,
    //             cluster_enabled: false,
    //             cluster_urls: vec!["".to_string()],
    //             use_legacy_version: false,
    //             pool_size: 50,
    //             reconnect_max_attempts: 10,
    //             reconnect_delay: 5000,
    //             default_ttl: 3600,
    //             default_hash_ttl: 3600,
    //             stream_read_count: 100,
    //             partition: 0,
    //         };
    //         let pool = RedisConnectionPool::new(config).await.unwrap();
    //         let pool = std::sync::Arc::new(pool);
    //         let manager = DriverPoolManager::new(
    //             pool,
    //             "drivers:pool",
    //             "drivers:inflight",
    //             "driver:request",
    //         )
    //         .await
    //         .unwrap();

    //         // clean slate
    //         let _: () = manager.redis.delete_key(&manager.pool_key).await.unwrap();
    //         let _: () =
    //             manager.redis.delete_key(&manager.inflight_key).await.unwrap();

    //         manager
    //     }

    // #[traced_test]
    #[tokio::test]
    async fn test_calculate_cost() {
        let fatigue = calculate_fatigue(8, 50.0);
        let now = Utc::now();
        let idle_since: DateTime<Utc> = "2026-01-22T04:30:13.377923Z"
            .parse()
            .expect("Failed to parse idle_since time");
        let idle_time = ((now - idle_since).num_seconds() as f64) / 3600.0;
        let acceptance = calculate_acceptance_rate(20, 10);
        println!("Cancelation rate: {}", 1.0 - acceptance);
        let driver = crate::types::NearbyDriverWithStat {
            driver_id: DriverId("driver123".to_string()),
            geo_point: GeoPoint {
                lat: Latitude(0.0),
                lon: Longitude(0.0),
            },
            distance: Meters(0.25),
            total_rating_score: 5.0,
            acceptance,
            earnings: 3690.0,
            fatigue,
            idle_time,
        };

        let cost = calculate_cost(&driver);
        println!("Calculated cost: {}", cost);
        assert!(
            (0.0..=1.0).contains(&cost),
            "Cost should be within [0.0, 1.0]"
        );
    }
}
