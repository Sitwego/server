pub mod dispatch;
pub mod state_machine;

// use crate::cache::keys::ride_request_key;
// use crate::dispatch::dispatch::DriverPoolManager;
// use crate::queries::driver_stats::DriverStatsQueries;
// use crate::queries::drivers::DriverQueries;
// use crate::queries::ride::RideQueries;
// use crate::schemas::ride_request::RideRequestStatus;
// use anyhow::Result;
// use chrono::Utc;
// use fred::prelude::LuaInterface;
// use sea_orm::{ActiveValue, Iterable, entity::prelude::*};
// use serde::{Deserialize, Serialize};
// use smallvec::SmallVec;
// use std::collections::HashMap;
// use std::sync::Arc;
// use time::OffsetDateTime;
// use tracing::{info, warn};
// use utils::gen_strings::{self, cal_hash, ulid_string};
// // // use pathfinding::kuhn_munkres::kuhn_munkres_min;
// // // use pathfinding::matrix::Matrix;
// use crate::APIContext;
// use crate::api::nearby_drivers::DriverCurrentLocationInfo;
// use crate::cache::read_writer::{
//     get_all_drivers_last_locations, search_for_nearby_drivers_within_radius,
// };
// use crate::helper::{create_bucket_key, ttl_to_datetime};
// use crate::schemas::{driver_stats, location, ride_request};
// use crate::types::{
//     DriverId, Meters, NearbyDriverWithStat, RideData, RideId, RideInfo,
//     TimeStamp, VehicleCategory,
// };
// use redis_store::r_types::{AppError, GeoPoint, Radius};
// use smallvec::smallvec;

// type RideResultOptions = Option<(Vec<(f64, f64)>, f64, u64)>;
// type RideResult = utils::Result<RideResultOptions>;
// pub type RideSearchResult =
//     Option<(Vec<(f64, f64)>, f64, u64, GeoPoint, GeoPoint)>;

// pub const LIMIT_PARALLEL_REQUESTS_LUA: &str = r#"
// -- KEYS[1] = ZSET key
// -- ARGV[1] = current_time (unix seconds)
// -- ARGV[2] = max_parallel
// -- ARGV[3] = request_id
// -- ARGV[4] = expiry_time (unix seconds)

// -- 1. Remove expired requests
// redis.call(
//   'ZREMRANGEBYSCORE',
//   KEYS[1],
//   '-inf',
//   ARGV[1]
// )

// -- 2. Count active requests
// local count = redis.call('ZCARD', KEYS[1])

// -- 3. If below limit, add request
// if count < tonumber(ARGV[2]) then
//   redis.call(
//     'ZADD',
//     KEYS[1],
//     ARGV[4],
//     ARGV[3]
//   )
//   return 1
// end

// return 0
// "#;

// pub mod eta_utils {
//     use crate::request::RidesApiClient;
//     use crate::simd_json::parse_from_string;
//     use redis_store::r_types::GeoPoint;

//     use super::RideResult;

//     // Weights for cost function
//     pub const ETA_WEIGHT: f64 = 0.4;
//     pub const RATING_WEIGHT: f64 = 0.2;
//     pub const ACCEPTANCE_WEIGHT: f64 = 0.15;
//     pub const EARNINGS_WEIGHT: f64 = 0.15;
//     pub const FATIGUE_WEIGHT: f64 = 0.1;

//     pub fn calculate_cost(
//         eta: f64,
//         driver: &crate::types::NearbyDriverWithStat,
//     ) -> f64 {
//         let norm_eta = (eta.min(600.0)) / 600.0;

//         let norm_rating =
//             1.0 - ((driver.total_rating_score - 1.0) / 4.0).clamp(0.0, 1.0);

//         let norm_acceptance = 1.0 - driver.acceptance.clamp(0.0, 1.0);

//         let norm_earnings = (driver.earnings / 1000.0).min(1.0);

//         let norm_fatigue = (driver.fatigue / 12.0).min(1.0);

//         ETA_WEIGHT * norm_eta
//             + RATING_WEIGHT * norm_rating
//             + ACCEPTANCE_WEIGHT * norm_acceptance
//             + EARNINGS_WEIGHT * norm_earnings
//             + FATIGUE_WEIGHT * norm_fatigue
//     }

//     pub trait EtaCalculator {
//         fn get_eta(
//             &self,
//             driver_point: GeoPoint,
//             rider_point: GeoPoint,
//         ) -> impl Future<Output = RideResult> + Send;
//     }

//     pub struct ApiEtaCalculator;

//     impl EtaCalculator for ApiEtaCalculator {
//         async fn get_eta(&self, d: GeoPoint, r: GeoPoint) -> RideResult {
//             #[cfg(feature = "reqwest-middleware")]
//             {
//                 let base_url = "http://127.0.0.1:5000";
//                 let req_instance =
//                     RidesApiClient::new_with_retry(base_url, None);
//                 let coords = [(d.lon.0, d.lat.0), (r.lon.0, r.lat.0)];
//                 let resp = parse_from_string(
//                     &req_instance
//                         .get_ride_path_and_distance(
//                             &coords,
//                             "overview=full&steps=true&geometries=geojson",
//                         )
//                         .await
//                         .expect("fetch eta failed"),
//                 );

//                 if let Ok(resp) = resp {
//                     return Ok(Some(resp));
//                 }
//                 Ok(None)
//             }
//         }
//     }
// }
// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct RequestRideData {
//     pub geo_point: GeoPoint,
//     pub street: Option<String>,
//     pub city: Option<String>,
//     pub road: Option<String>,
//     pub country: Option<String>,
//     pub building: Option<String>,
//     pub floor: Option<String>,
//     pub door: Option<String>,
//     pub area_code: Option<String>,
//     pub ward: Option<String>,
//     pub place_id: Option<String>,
//     pub instructions: Option<String>,
//     pub extras: Option<sea_orm::entity::prelude::Json>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RideRequestNotification {
//     id: String,
//     from: RequestRideData,
//     to: RequestRideData,
//     fare: i32,
//     distance: f64,
//     vc: Option<VehicleCategory>,
//     duration: i32,
//     distance_to_pickup: f64,
//     duration_to_pickup: f64,
//     driver_to_pickup_line_str: Option<Vec<(f64, f64)>>,
//     ride_line_str: Option<Vec<(f64, f64)>>,
//     msg: Option<String>,
//     search_request_id: String,
//     search_request_valid_till: OffsetDateTime,
//     estimated_start_time: Option<OffsetDateTime>,
// }

// pub struct DriverAssignment<T: eta_utils::EtaCalculator> {
//     ctx: Arc<APIContext>,
//     request: RequestDriver,
//     drivers: Vec<DriverCurrentLocationInfo>,
//     client_id: String,
//     search_req_id: String,
//     request_id: String,
//     stats: HashMap<String, driver_stats::Model>,
//     drivers_with_stats: Option<Vec<NearbyDriverWithStat>>,
//     assigned_driver: Option<String>,
//     assigned_driver_eta: RideResultOptions,
//     notification: Option<RideRequestNotification>,
//     rq_model: Option<ride_request::Model>,
//     eta_calculator: T,
//     ride: RideSearchResult,
//     driver_pool_manager: DriverPoolManager,
// }

// impl<T: eta_utils::EtaCalculator> DriverAssignment<T> {
//     pub async fn new(
//         ctx: Arc<APIContext>,
//         request: RequestDriver,
//         client_id: String,
//         search_req_id: String,
//         ride: RideSearchResult,
//         eta_calculator: T,
//     ) -> Self {
//         let pool_ky = format!("driver::pool::{}::key", search_req_id);
//         let inflight_ky = format!("driver::inflight::{}::key", search_req_id);
//         let request_map_key = format!("driver:request::{}::key", search_req_id);
//         let driver_pool_manager = DriverPoolManager::new(
//             ctx.redis.clone(),
//             &pool_ky,
//             &inflight_ky,
//             &request_map_key,
//         )
//         .await
//         .expect("Failed to create DriverPoolManager");
//         Self {
//             ctx,
//             request,
//             request_id: search_req_id.to_owned(),
//             drivers: Vec::new(),
//             client_id,
//             search_req_id,
//             stats: HashMap::new(),
//             drivers_with_stats: None,
//             assigned_driver: None,
//             assigned_driver_eta: None,
//             notification: None,
//             ride,
//             rq_model: None,
//             eta_calculator,
//             driver_pool_manager,
//         }
//     }

//     pub async fn prepare_notification(mut self) -> Result<Self, AppError> {
//         let (ride_path, _, _, _, _) =
//             self.ride.to_owned().expect("This can not be null");

//         let notification = RideRequestNotification {
//             id: ulid_string(),
//             from: self.request.from.to_owned(),
//             to: self.request.to.to_owned(),
//             fare: self.request.fare,
//             duration: self.request.duration,
//             distance: self.request.dx,
//             distance_to_pickup: 0.0,
//             duration_to_pickup: 0.0,
//             vc: None,
//             msg: None,
//             driver_to_pickup_line_str: None,
//             ride_line_str: Some(ride_path),
//             search_request_id: self.search_req_id.to_owned(),
//             search_request_valid_till: OffsetDateTime::now_utc(),
//             estimated_start_time: None,
//         };
//         self.notification = Some(notification);
//         Ok(self)
//     }

//     pub async fn find_drivers(mut self) -> Result<Self, AppError> {
//         let drivers = find_nearest_driver(
//             self.ctx.clone(),
//             self.request.from.to_owned().geo_point,
//             &self.request.vehicle_type,
//             &self.request.radius,
//         )
//         .await?;

//         if drivers.is_empty() {
//             warn!(
//                 "No drivers found within the radius: {:?} for request: {:?}",
//                 self.request.radius, self.request
//             );
//             return Err(AppError::NotFound(
//                 "No drivers found within the specified radius".to_string(),
//             ));
//         }
//         self.drivers = drivers;
//         Ok(self)
//     }

//     pub async fn driver_stats(mut self) -> Result<Self, AppError> {
//         let stats = self
//             .ctx
//             .clone()
//             .db
//             .get_drivers_stats(
//                 self.drivers
//                     .iter()
//                     .map(|d| d.to_owned().driver_id.0)
//                     .collect::<Vec<String>>(),
//             )
//             .await
//             .map_err(|err| AppError::InternalError(err.to_string()))?;

//         if stats.is_empty() {
//             warn!(
//                 "No driver stats found for the given drivers: {:?}",
//                 self.drivers
//             );
//             return Err(AppError::NotFound(
//                 "No driver stats found for the given drivers".to_string(),
//             ));
//         }
//         self.stats = stats;
//         Ok(self)
//     }

//     fn drivers_with_stats(
//         &self,
//         drivers: &[DriverCurrentLocationInfo],
//     ) -> Result<Vec<NearbyDriverWithStat>> {
//         let mut drivers_with_stats: Vec<NearbyDriverWithStat> = Vec::new();
//         let now = Utc::now();

//         for driver in drivers {
//             let (total_rating_score, acceptance, earnings, fatigue, idle_time) =
//                 match self.stats.get(&driver.driver_id.0) {
//                     Some(stat) => {
//                         let acceptance = match (
//                             stat.rides_cancelled,
//                             stat.total_rides_assigned,
//                         ) {
//                             (Some(cancelled), Some(assigned))
//                                 if assigned > 0 =>
//                             {
//                                 if cancelled > assigned {
//                                     0.0
//                                 } else {
//                                     cancelled as f64 / assigned as f64
//                                 }
//                             }

//                             _ => 0.8,
//                         };

//                         let fatigue = calculate_fatigue(stat.total_rides, 50.0);
//                         let idle_time = ((now - stat.idle_since.to_utc())
//                             .num_seconds()
//                             as f64)
//                             / 3600.0;

//                         let total_rating_score =
//                             stat.total_rating_score.unwrap_or(0.0);

//                         (
//                             total_rating_score,
//                             acceptance,
//                             stat.total_earnings,
//                             fatigue,
//                             idle_time,
//                         )
//                     }
//                     None => (0.0, 0.8, 0.0, 0.0, 0.0),
//                 };

//             drivers_with_stats.push(NearbyDriverWithStat {
//                 driver_id: driver.driver_id.clone(),
//                 geo_point: driver.coords,
//                 total_rating_score,
//                 acceptance: acceptance.clamp(0.0, 1.0),
//                 earnings,
//                 fatigue: fatigue.clamp(0.0, 1.0),
//                 idle_time: idle_time.max(0.0),
//             });
//         }

//         Ok(drivers_with_stats)
//     }

//     // Limit parallel ride requests to drivers
//     async fn can_get_ride_request(
//         &self,
//         driver_id: &str,
//     ) -> Result<bool, AppError> {
//         let now = Utc::now().timestamp();
//         let exp = now + 30; // 15 -> 30 seconds expiry
//         // key is request_id
//         let key = format!("ride_request::driver_pool::{}", driver_id);
//         let res: i64 = self
//             .ctx
//             .redis
//             .pool
//             .eval(
//                 LIMIT_PARALLEL_REQUESTS_LUA,
//                 vec![key],
//                 vec![
//                     now.to_string(),         // Current time
//                     "2".to_string(), // Max parallel requests driver will only get 2 requests at a time
//                     self.request_id.clone(), // Request ID
//                     exp.to_string(), // Expiry time
//                 ],
//             )
//             .await
//             .map_err(|er| {
//                 AppError::InternalError(format!(
//                     "Failed to create driver pool: {:?}",
//                     er
//                 ))
//             })?;
//         Ok(res == 1)
//     }

//     async fn dispatcher(
//         &self,
//         driver_stats: &Vec<NearbyDriverWithStat>,
//     ) -> Result<()> {
//         let key = format!("ride-drivers-list::{}", self.request_id);
//         let resp = self
//             .ctx
//             .redis
//             .zrange::<String>(&key, 0, 10, None, false, None, false)
//             .await
//             .map_err(|er| {
//                 AppError::InternalError(format!(
//                     "Failed to fetch drivers from redis: {:?}",
//                     er
//                 ))
//             })?;
//         info!(
//             "Top drivers from redis for request {}: {:?}",
//             self.request_id, resp
//         );
//         for driver in driver_stats {
//             if let Some((_, _, eta)) = self
//                 .eta_calculator
//                 .get_eta(driver.clone().geo_point, self.request.from.geo_point)
//                 .await?
//             {
//                 // let cost = eta_utils::calculate_cost(eta as f64, &driver);
//                 let can_get =
//                     self.can_get_ride_request(&driver.driver_id.0).await?;
//                 if can_get {
//                     warn!(
//                         "Driver ID: {:?}, ETA: {:?}, Can Get Ride Request: {:?}",
//                         driver.driver_id.0, eta, can_get
//                     );
//                 } else {
//                     warn!(
//                         "Driver ID: {:?} has reached max parallel ride requests. ETA: {:?}",
//                         driver.driver_id.0, eta
//                     );
//                 }
//             }
//         }
//         Ok(())
//     }

//     async fn assign_driver(
//         &self,
//         nearby_driver_with_stat: &Vec<NearbyDriverWithStat>,
//     ) -> Result<(Option<String>, RideResultOptions)> {
//         let mut best_driver: Option<(String, f64)> = None;
//         let mut best_eta: RideResultOptions = None;

//         let request_id = ride_request_key(&self.request_id);

//         for driver_with_stat in nearby_driver_with_stat {
//             if let Some((line_str, dx, eta)) = self
//                 .eta_calculator
//                 .get_eta(
//                     driver_with_stat.to_owned().geo_point,
//                     self.request.from.geo_point,
//                 )
//                 .await?
//             {
//                 let cost =
//                     eta_utils::calculate_cost(eta as f64, driver_with_stat);
//                 info!(
//                     "Cost is {:?} for driver id {:?}",
//                     cost, driver_with_stat.driver_id.0
//                 );

//                 if !self
//                     .driver_pool_manager
//                     .is_inflight(&driver_with_stat.driver_id.0)
//                     .await?
//                 {
//                     // add to inflight
//                     self.driver_pool_manager
//                         .add_driver(&driver_with_stat.driver_id.0, cost)
//                         .await?;

//                     if let Some(acquired_driver) = self
//                         .driver_pool_manager
//                         .acquire_driver(&request_id, 60_000, 20)
//                         .await?
//                     {
//                         info!(
//                             "Acquired driver {:?} for request {:?}",
//                             acquired_driver, request_id
//                         );
//                         break;
//                     }
//                 }

//                 match best_driver {
//                     None => {
//                         best_driver = Some((
//                             driver_with_stat.driver_id.0.to_string(),
//                             cost,
//                         ));
//                         best_eta = Some((line_str, dx, eta));
//                     }
//                     Some((_, best_cost)) if cost < best_cost => {
//                         best_driver = Some((
//                             driver_with_stat.driver_id.0.to_string(),
//                             cost,
//                         ));
//                         best_eta = Some((line_str, dx, eta));
//                     }
//                     _ => {}
//                 }
//                 // best_driver = match best_driver {
//                 //     None => Some((driver_with_stat.driver_id.0, cost)),
//                 //     Some((_, best_cost)) if cost < best_cost => Some((driver_with_stat.driver_id.0, cost)),
//                 //     Some(best) => Some(best),
//                 // };
//             }
//         }

//         Ok((best_driver.map(|(id, _)| id), best_eta))
//     }

//     pub fn with_stats(mut self) -> Result<Self, AppError> {
//         let drivers_with_stats = self
//             .drivers_with_stats(&self.drivers)
//             .map_err(|er| AppError::InternalError(er.to_string()))?;
//         self.drivers_with_stats = Some(drivers_with_stats);
//         Ok(self)
//     }

//     pub async fn assign(mut self) -> Result<Self, AppError> {
//         let drivers_with_stats = self
//             .drivers_with_stats
//             .as_ref()
//             .ok_or_else(|| {
//                 anyhow::anyhow!("Must call with_stats() before assign()")
//             })
//             .map_err(|er| AppError::InternalError(er.to_string()))?;

//         let _ = self.dispatcher(drivers_with_stats).await;

//         let (assigned_driver, assigned_driver_eta) = self
//             .assign_driver(drivers_with_stats)
//             .await
//             .map_err(|er| AppError::InternalError(er.to_string()))?;
//         match assigned_driver {
//             Some(ref driver_id) => {
//                 self.ctx
//                     .db
//                     .update_driver_stats_total_rides_assigned(
//                         driver_id.to_string(),
//                         &1,
//                     )
//                     .await
//                     .map_err(|err| AppError::InternalError(err.to_string()))?;
//             }
//             None => {}
//         }

//         if assigned_driver.is_none() {
//             warn!(
//                 "No suitable driver found for the ride request: {:?}",
//                 self.request
//             );
//             return Err(AppError::NotFound(
//                 "No suitable driver found for the ride request".to_string(),
//             ));
//         }
//         self.assigned_driver = assigned_driver;
//         self.assigned_driver_eta = assigned_driver_eta;
//         Ok(self)
//     }

//     pub async fn notify_driver(
//         mut self,
//         custom_msg: Option<String>,
//     ) -> Result<Self, AppError> {
//         if let (Some(driver_id), Some(mut notification), Some(eta)) = (
//             &self.assigned_driver,
//             self.notification.take(),
//             &self.assigned_driver_eta,
//         ) {
//             let assigned_driver_info = self
//                 .drivers
//                 .iter()
//                 .find(|d| d.driver_id.0 == *driver_id)
//                 .ok_or_else(|| {
//                     anyhow::anyhow!("Assigned driver not found in drivers list")
//                 })
//                 .map_err(|er| AppError::InternalError(er.to_string()))?;

//             notification.distance_to_pickup = assigned_driver_info.distance;
//             notification.duration_to_pickup = eta.2 as f64 / 60.0;
//             notification.vc = Some(assigned_driver_info.vehicle_category);

//             let estimated_start_time = OffsetDateTime::now_utc()
//                 + tokio::time::Duration::from_secs(eta.2);
//             notification.estimated_start_time = Some(estimated_start_time);

//             if let Some(msg) = custom_msg {
//                 notification.msg = Some(msg);
//             }

//             let shard = (cal_hash(driver_id) % 128) as u64;
//             let ttl = (Utc::now() + tokio::time::Duration::from_secs(45))
//                 .to_rfc3339();
//             let request_id = self.request_id.clone();
//             notification.driver_to_pickup_line_str = Some(eta.0.clone()); //TODO: can be saved to a redis key and fetched later when needed.

//             let stream_data: SmallVec<[(String, String); 8]> = smallvec![
//                 ("type".to_string(), "RideRequest".to_string()),
//                 ("category".to_string(), "NEW_RIDE_AVAILABLE".to_string()),
//                 ("ttl".to_string(), ttl.to_owned()),
//                 ("id".to_string(), request_id.to_owned()),
//                 (
//                     "data".to_string(),
//                     serde_json::to_string(&notification).unwrap() // Serialize notification data
//                 )
//             ];

//             let (dx, dt) = match self.ride.as_ref() {
//                 Some((_, dx, dt, _, _)) => (*dx, *dt),
//                 None => (0.0, 0),
//             };

//             self.rq_model = Some(ride_request::Model {
//                 id: request_id,
//                 driver_id: driver_id.to_owned(),
//                 customer_id: self.client_id.to_owned(),
//                 fare: Decimal::new(self.request.fare as i64, 0),
//                 estimated_distance_to_pickup: Some(
//                     notification.distance_to_pickup,
//                 ),
//                 estimated_duration_to_pickup: Some(
//                     notification.duration_to_pickup as i32,
//                 ),
//                 estimated_distance: Some(dx),
//                 estimated_duration: Some(dt as i32),
//                 search_request_valid_till: Some(ttl_to_datetime(&ttl).unwrap()),
//                 start_time: Some(estimated_start_time),
//                 ..Default::default()
//             });

//             self.ctx
//                 .redis
//                 .xadd(
//                     &format!("notif::driver-id::{shard}&{driver_id}"),
//                     stream_data.to_vec(),
//                     1000,
//                 )
//                 .await
//                 .map_err(|er| AppError::InternalError(er.to_string()))?;
//         }

//         Ok(self)
//     }

//     pub async fn create_ride_request(self) -> Result<Self, AppError> {
//         if self.rq_model.is_none() {
//             return Err(AppError::InternalError(
//                 "Ride request model must be set before creating ride request"
//                     .to_string(),
//             ));
//         }
//         let start_otp = gen_strings::generate_otp(4);
//         let end_otp = gen_strings::generate_otp(4);
//         let from =
//             transform_ride_request_location_data(self.request.from.to_owned());
//         let to =
//             transform_ride_request_location_data(self.request.to.to_owned());
//         let (from, to) =
//             self.ctx
//                 .clone()
//                 .db
//                 .create_locations(from, to)
//                 .await
//                 .map_err(|err| AppError::InternalError(err.to_string()))?;

//         let ride_request_model =
//             self.rq_model.to_owned().expect("Ride request model can't be null");
//         let _ = self
//             .ctx
//             .clone()
//             .db
//             .create_ride_request(
//                 ride_request_model,
//                 from,
//                 to,
//                 start_otp,
//                 end_otp,
//             )
//             .await
//             .map_err(|err| AppError::InternalError(err.to_string()))?;

//         Ok(self)
//     }
//     pub async fn result<F, Fut>(
//         self,
//         f: F,
//     ) -> Result<Option<(String, String)>, AppError>
//     where
//         F: FnOnce(Option<(RideInfo, String)>) -> Fut,
//         Fut: Future<Output = Result<(), AppError>>,
//     {
//         let estimated_pickup_time = self
//             .rq_model
//             .as_ref()
//             .map(|n| n.estimated_duration_to_pickup.unwrap_or(0) as f64)
//             .unwrap_or(0.0);
//         let estimated_pickup_distance = self
//             .rq_model
//             .as_ref()
//             .map(|n| Meters(n.estimated_distance_to_pickup.unwrap_or(0.0)));

//         let (polyline, _, _) = match self.ride {
//             Some((c, v, n, _, _)) => (c, v, n),
//             None => (vec![], 0.0, 0),
//         };

//         let ride_id = RideId(self.request_id);

//         let ride_info = RideInfo {
//             ride_id: ride_id.to_owned(),
//             ride_status: RideRequestStatus::New,
//             ride_data: Some(RideData::Taxi {
//                 polyline: Some(polyline),
//                 pickup_location: (
//                     self.request.from.geo_point.lat.0,
//                     self.request.from.geo_point.lon.0,
//                 )
//                     .into(),
//             }),
//             estimated_pickup_time: Some(estimated_pickup_time),
//             estimated_pickup_distance,
//             created_at: TimeStamp(Utc::now()),
//         };

//         f(self
//             .assigned_driver
//             .as_ref()
//             .map(|driver| (ride_info, driver.clone())))
//         .await?;
//         Ok(self.assigned_driver.map(|driver| (driver, ride_id.0)))
//     }
// }

// pub async fn find_nearest_driver(
//     ctx: Arc<APIContext>,
//     coordinate: GeoPoint,
//     vehicle_type: &Option<Vec<VehicleCategory>>,
//     radius: &Radius,
// ) -> Result<Vec<DriverCurrentLocationInfo>, AppError> {
//     let buckect_key = create_bucket_key(30, TimeStamp(Utc::now()));

//     match vehicle_type {
//         Some(vehicles) => {
//             info!(
//                 "SEARCHING NEARBY DRIVERS: {:?} {:?} {:?}",
//                 coordinate, vehicle_type, radius
//             );
//             let mut response: Vec<DriverCurrentLocationInfo> = Vec::new();
//             for vc in vehicles {
//                 let nearest_drivers = search_for_nearby_drivers_within_radius(
//                     &ctx.redis,
//                     &buckect_key,
//                     &4,
//                     vc,
//                     radius,
//                     &coordinate,
//                     // region
//                 )
//                 .await?;

//                 let driver_ids = nearest_drivers
//                     .iter()
//                     .map(|driver| driver.dirver_id.clone())
//                     .collect::<Vec<DriverId>>();

//                 let latest_driver_location =
//                     get_all_drivers_last_locations(&ctx.redis, &driver_ids)
//                         .await?;

//                 let resp = nearest_drivers
//                     .iter()
//                     .zip(latest_driver_location.iter())
//                     .map(|(a, b)| {
//                         let timestamp = b.as_ref().map_or_else(
//                             || TimeStamp(Utc::now()),
//                             |loc| loc.timestamp,
//                         );

//                         let vehicle_category = b.as_ref().map_or_else(
//                             || VehicleCategory::Swift,
//                             |d_info| d_info.vehicle_category,
//                         );
//                         DriverCurrentLocationInfo {
//                             coords: a.coords,
//                             timestamp,
//                             driver_id: a.dirver_id.clone(),
//                             distance: a.distance,
//                             vehicle_category,
//                         }
//                     })
//                     .collect::<Vec<DriverCurrentLocationInfo>>();

//                 response.extend(resp);
//             }

//             Ok(response)
//         }
//         None => {
//             let mut response: Vec<DriverCurrentLocationInfo> = Vec::new();
//             for vc in VehicleCategory::iter() {
//                 let nearest_drivers = search_for_nearby_drivers_within_radius(
//                     &ctx.redis,
//                     &buckect_key,
//                     &4,
//                     &vc,
//                     radius,
//                     &coordinate,
//                     // region
//                 )
//                 .await?;

//                 let driver_ids = nearest_drivers
//                     .iter()
//                     .map(|driver| driver.dirver_id.clone())
//                     .collect::<Vec<DriverId>>();

//                 let latest_driver_location =
//                     get_all_drivers_last_locations(&ctx.redis, &driver_ids)
//                         .await?;

//                 let resp = nearest_drivers
//                     .iter()
//                     .zip(latest_driver_location.iter())
//                     .map(|(a, b)| {
//                         let timestamp = b.as_ref().map_or_else(
//                             || TimeStamp(Utc::now()),
//                             |loc| loc.timestamp,
//                         );

//                         let vehicle_category = b.as_ref().map_or_else(
//                             || VehicleCategory::Swift,
//                             |d_info| d_info.vehicle_category,
//                         );
//                         DriverCurrentLocationInfo {
//                             coords: a.coords,
//                             timestamp,
//                             driver_id: a.dirver_id.clone(),
//                             distance: a.distance,
//                             vehicle_category,
//                         }
//                     })
//                     .collect::<Vec<DriverCurrentLocationInfo>>();

//                 response.extend(resp);
//             }

//             Ok(response)
//         }
//     }
// }

// fn calculate_fatigue(total_rides: i32, threshold: f64) -> f64 {
//     let fatigue = if total_rides > 75 {
//         (total_rides as f64 / threshold).powf(1.5)
//     } else {
//         total_rides as f64 / threshold
//     };

//     fatigue.max(0.0).min(0.5)
// }

// pub fn transform_ride_request_location_data(
//     body: RequestRideData,
// ) -> location::ActiveModel {
//     location::ActiveModel {
//         id: ActiveValue::Set(ulid_string()),
//         lat: ActiveValue::Set(body.geo_point.lat.0),
//         lon: ActiveValue::Set(body.geo_point.lat.0),
//         street: ActiveValue::Set(body.street),
//         city: ActiveValue::Set(body.city),
//         road: ActiveValue::Set(body.road),
//         country: ActiveValue::Set(body.country),
//         building: ActiveValue::Set(body.building),
//         floor: ActiveValue::Set(body.floor),
//         door: ActiveValue::Set(body.door),
//         area_code: ActiveValue::Set(body.area_code),
//         ward: ActiveValue::Set(body.ward),
//         place_id: ActiveValue::Set(body.place_id),
//         instructions: ActiveValue::Set(body.instructions),
//         extras: ActiveValue::Set(body.extras),
//         ..Default::default()
//     }
// }

// // Multiple riders: Bipartite matching
// // async fn assign_drivers(riders: Vec<Rider>) -> Result<Vec<(String, String)>> {
// //     let mut all_drivers = Vec::new();
// //     for rider in &riders {
// //         let drivers = get_nearby_drivers(redis, db, rider.lat, rider.lon).await?;
// //         all_drivers.extend(drivers);
// //     }
// //     all_drivers.sort_by(|a, b| a.id.cmp(&b.id));
// //     all_drivers.dedup_by(|a, b| a.id == b.id);

// //     if all_drivers.is_empty() || riders.is_empty() {
// //         return Ok(vec![]);
// //     }

// //     let mut cost_rows = Vec::new();
// //     for rider in &riders {
// //         let eta_futures: Vec<_> = all_drivers.iter().map(|driver| {
// //             let driver_clone = driver.clone();
// //             async move {
// //                 let eta = get_eta(client, &config.osrm_url, driver_clone.lat, driver_clone.lon, rider.lat, rider.lon).await;
// //                 (driver_clone, eta)
// //             }
// //         }).collect();

// //         let results = join_all(eta_futures).await;
// //         let mut row = Vec::new();
// //         for (driver, eta_opt) in results {
// //             let cost = eta_opt.map_or(1000.0, |eta| calculate_cost(eta, &driver));
// //             row.push(cost);
// //         }
// //         cost_rows.push(row);
// //     }

// //     let driver_count = all_drivers.len();
// //     for row in &mut cost_rows {
// //         while row.len() < driver_count {
// //             row.push(1000.0);
// //         }
// //     }

// //     let cost_matrix = Matrix::from_rows(cost_rows)?;
// //     let (_, assignments) = kuhn_munkres_min(&cost_matrix);

// //     let mut result = Vec::new();
// //     for (rider_idx, driver_idx) in assignments.iter().enumerate() {
// //         if rider_idx < riders.len() && *driver_idx < all_drivers.len() {
// //             result.push((riders[rider_idx].id.clone(), all_drivers[*driver_idx].id.clone()));
// //         }
// //     }

// //     Ok(result)
// // }
