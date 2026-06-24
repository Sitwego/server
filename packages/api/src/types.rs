use chrono::{DateTime, Utc};
use dashmap::DashMap;
use fred::types::geo::GeoValue;
use geo::Point;
pub use redis_store::r_types::*;
use serde::{Deserialize, Serialize};
use std::ops::Not;
use strum_macros::{Display, EnumIter, EnumString};

use crate::schemas::ride_request::RideRequestStatus;
pub use crate::schemas::vehicle_categories::VehicleCategory;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct MarchantId(pub String);
#[shared_macro::impl_getter]
#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct RideId(pub String);
#[derive(Deserialize, Serialize, Clone, Debug, Eq, Hash, PartialEq)]
#[shared_macro::impl_getter]
pub struct DriverId(pub String);
#[derive(Deserialize, Serialize, Clone, Debug, Eq, Hash, PartialEq)]
#[shared_macro::impl_getter]
pub struct ProfileId(pub String);
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
#[shared_macro::impl_getter]
pub struct TownName(pub String);

#[derive(Deserialize, Serialize, Clone, Debug, Eq, Hash, PartialEq)]
#[shared_macro::impl_getter]
pub struct CustomerId(pub String);

#[derive(
    Deserialize, Serialize, Clone, Copy, Debug, Eq, PartialEq, PartialOrd,
)]
#[shared_macro::impl_getter]
pub struct TimeStamp(pub DateTime<Utc>);
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct AccuracyThreshold(pub f64);
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct VelocityInMetersPerSec(pub f64);
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Copy)]
pub struct Meters(pub f64);
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Copy)]
pub struct Minutes(pub f64);

pub type DriverLocationMap = DashMap<String, Vec<GeoValue>>;

#[derive(
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    EnumIter,
    EnumString,
    Display,
)]
pub enum RideNotificationState {
    #[strum(serialize = "IDLE")]
    #[serde(rename = "IDLE")]
    Idle = 0,
    #[strum(serialize = "DRIVER_ON_THE_WAY")]
    #[serde(rename = "DRIVER_ON_THE_WAY")]
    DriverOnTheWay = 1,
    #[strum(serialize = "DRIVER_ARRIVING")]
    #[serde(rename = "DRIVER_ARRIVING")]
    DriverArriving = 2,
    #[strum(serialize = "DRIVER_ARRIVED")]
    #[serde(rename = "DRIVER_ARRIVED")]
    DriverArrived = 3,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationProfile {
    pub vehicle_category: VehicleCategory,
    pub created_at: TimeStamp,
}

#[derive(
    Debug,
    Clone,
    EnumString,
    Display,
    Serialize,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
)]
pub enum DriverMode {
    Online,
    Offline,
    Idle,
}

#[derive(
    Debug,
    Clone,
    EnumString,
    Display,
    Serialize,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AccountType {
    Driver = 0,
    Customer = 1,
    Admin = 2,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    pub driver_id: DriverId,
    pub driver_mode: DriverMode,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Customer {
    pub customer_id: CustomerId,
    pub coordinates: GeoPoint,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DriverLocationIfo {
    pub ride_status: Option<RideRequestStatus>,
    pub position_info: DriverLocation,
    pub ride_notification_state: Option<RideNotificationState>,
    pub pickup_location_distance: Option<Meters>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NearbyDriverWithStat {
    pub driver_id: DriverId,
    pub geo_point: GeoPoint,
    pub distance: Meters, // distance from customer pickup location
    pub total_rating_score: f64,
    pub acceptance: f64,
    pub earnings: f64,
    pub fatigue: f64,
    pub idle_time: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriverLocation {
    pub timestamp: TimeStamp,
    pub location: GeoPoint,
    pub distance: Option<Meters>,
    pub driver_id: DriverId,
    pub vehicle_category: VehicleCategory,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriverLocationCoordinates {
    pub coords: GeoPoint,
    pub dirver_id: DriverId,
    pub distance: f64,
}

#[derive(Debug, Clone)]
pub struct DriverLocationEvent {
    pub location_profile: LocationProfile,
    pub latitude: Latitude,
    pub longitude: Longitude,
    pub timestamp: TimeStamp,
    pub driver_id: DriverId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RideInfo {
    pub ride_id: RideId,
    pub ride_status: RideRequestStatus,
    pub ride_data: Option<RideData>,
    pub estimated_pickup_time: Option<f64>,
    pub estimated_pickup_distance: Option<Meters>,
    pub created_at: TimeStamp,
    pub vehicle_category: VehicleCategory,
    /// Server-computed pickup fare (KES), locked at driver-accept time. Read
    /// back at ride start. `#[serde(default)]` keeps ride-infos cached before
    /// this field existed deserializable.
    #[serde(default)]
    pub pickup_fare: Option<i32>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Display, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RideData {
    #[serde(rename_all = "camelCase")]
    Taxi {
        pickup_location: Point,
        polyline: Option<Vec<(f64, f64)>>,
        polyline_2: Option<Vec<(f64, f64)>>,
    },
    #[serde(rename_all = "camelCase")]
    Ridepooling {
        route_code: String,
        destination: Point,
        pickup_location: Point,
        driver_name: Option<String>,
        route_long_name: Option<String>,
        vehicle_number: String,
    },
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactData {
    pub email: String,
    pub phone_number: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct APISuccess {
    pub res: String,
}

impl Default for APISuccess {
    fn default() -> Self {
        Self {
            res: "Success".to_string(),
        }
    }
}

impl Not for RideId {
    type Output = bool;
    fn not(self) -> Self::Output {
        self.0.is_empty()
    }
}
