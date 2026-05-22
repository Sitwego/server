use crate::{DriverId, RideId, VehicleCategory};

/// Driver Location Record key
/// # Arguments
/// * `driver_id` driver ID
/// # Returns
///  a String
///
pub fn driver_location_info_key(DriverId(id): DriverId) -> String {
    format!("dr::lt::r-{id}")
}

pub fn saving_driver_location_record_key(
    key: String,
    DriverId(id): DriverId,
) -> String {
    format!("sd::lt::r-{key}-{id}")
}

pub fn ride_info_key(DriverId(id): DriverId) -> String {
    format!("ride::lt::info::{id}")
}

pub fn on_driver_location_key(
    buckect: &i64,
    vehicle_category: &VehicleCategory,
) -> String {
    format!("on::dr::lt::{vehicle_category}-{buckect}")
}

pub fn on_going_ride_coordinates_key(
    DriverId(driver_id): &DriverId,
    RideId(ride_id): &RideId,
) -> String {
    format!("ongoing::ri::coord::{driver_id}-{ride_id}")
}

pub fn ride_path_key_id(path_id: &str) -> String {
    format!("ride::path::id::{path_id}")
}

pub fn mpesa_c_key(checkout_req_id: &str, merchant_req_id: &str) -> String {
    format!("mpesa::key{}-{}", merchant_req_id, checkout_req_id)
}

pub fn ride_request_key(request_id: &str) -> String {
    format!("driver::ride::request::{request_id}")
}

pub fn notification_key(driver_id: &str, shard: u64) -> String {
    format!("notif::driver-id::{shard}&{driver_id}")
}

pub fn request_candidate_list_key(request_id: &str) -> String {
    format!("rd::request::candidate::list::{request_id}")
}

pub fn driver_opted_categories_key(DriverId(id): &DriverId) -> String {
    format!("dr::opted::cats::{id}")
}

pub fn driver_max_radius_key(DriverId(id): &DriverId) -> String {
    format!("dr::mxr::{id}")
}
