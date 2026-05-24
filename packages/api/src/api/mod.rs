pub mod bs_plans;
pub mod captain;
pub mod customer;
pub mod docs;
pub mod driver;
pub mod nearby_drivers;
pub mod preferences;
pub mod profile;
pub mod provider;
pub mod rating;
pub mod ride_request;
pub mod rides;
pub mod two_factor;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, post, put},
};
use nearby_drivers::get_nearby_drivers;
use profile::{create_profile, update_device_info, update_personal_details};
use provider::create_driver;
use rides::{
    accept_ride_request, cancel_ride, create_ride, end_ride,
    ride_fair_estimation, send_ride_request, update_location_coordinates,
};

use crate::api::driver::payment;

pub fn handlers() -> Router {
    Router::new()
        .route("/create-driver", post(create_driver))
        .route("/logout-driver", put(captain::logout_driver))
        .route(
            "/update-location-coordinates",
            post(update_location_coordinates),
        )
        .route("/go-online", post(driver::go_online))
        .route("/go-offline", post(driver::go_offline))
        .route("/create-ride/{is_ride_on_free_trial}", post(create_ride))
        // .route("/get-ride-details", get(get_ride_details))
        // .route("/get-driver-details", get(get_driver_details))
        .route("/end-ride/{ride_id}/end", delete(end_ride))
        .route(
            "/search-for-nearby-drivers/{region}",
            get(get_nearby_drivers),
        )
        .route("/ride-fair-estimation", post(ride_fair_estimation))
        .route("/send-ride-request", post(send_ride_request))
        .route("/api/cancel-ride/{ride_id}", post(cancel_ride))
        .route(
            "/api/rider-cancel-ride-request/{ride_id}",
            post(rides::rider_cancel_ride_request),
        )
        .route(
            "/accept-ride-request/{ride_id}/{vc}/accept",
            post(accept_ride_request),
        )
        .route(
            "/get-accepted-ride-by-driver/{ride_id}/{driver_id}/ride-details",
            get(rides::get_accepted_ride_by_driver),
        )
        .route(
            "/driver/arrived-at-pickup-location",
            post(rides::driver_arrived_at_pickup_location),
        )
        .route(
            "/create-subscriptions-plan/{plan_id}",
            post(bs_plans::create_bs_plan),
        )
        .route("/get-bs-subscription", get(bs_plans::get_bs_subscription))
        .route(
            "/rate-driver/{driver_id}/{ride_id}",
            post(rating::rate_driver),
        )
        .route("/rate-rider/{ride_id}", post(rating::rate_rider))
        // Driver documents routes
        .route(
            "/driver/upload-documents",
            post(docs::handle_docs_upload)
                .options(options)
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route(
            "/get-driver-documents/{path_id}",
            get(docs::get_driver_document),
        )
        .route(
            "/save-driver-identity-documents/{id_type}",
            post(docs::save_driver_identity_documents),
        )
        .route(
            "/save-driver-documents/{document_type}",
            post(docs::save_driver_documents),
        )
        .route(
            "/save-vehicle-info/v-details",
            post(docs::save_vehicle_info),
        )
        .route("/set-driver-photo", post(captain::set_driver_photo))
        .route(
            "/get-driver-simple-profile",
            get(captain::get_driver_simple_profile),
        )
        .route(
            "/driver/subscription-status",
            get(captain::get_driver_subscription_status),
        )
        .route(
            "/set-driver-has-completed-onboarding",
            post(captain::set_driver_has_completed_onboarding),
        )
        .route(
            "/driver/categories",
            get(captain::get_driver_categories)
                .put(captain::set_driver_categories),
        )
        .route("/test-job", post(bs_plans::test_get_bs_plans)) // TODO:: to be romoved in production
        // .route("/track-driver-on-ride-location/:ride_id", get(track_driver_on_ride_location))
        .route(
            "/payment/charge-phone-number/{sub_id}",
            post(payment::charge_phone_number),
        )
        .route(
            "/payment/confirm-payment/{chekout_req_id}",
            get(payment::confirm_payment),
        )
        .route(
            "/confirm-collected-cash/{ride_id}/{is_discounted}/{discount}",
            post(driver::confirm_collected_cash),
        )
        .route(
            "/api/get-driver-daily-earnings/{date}",
            get(driver::get_driver_daily_earnings),
        )
        .route(
            "/api/get-driver-weekly-earnings-report/{date}",
            get(driver::get_driver_weekly_earnings),
        )
        .route(
            "/api/get-driver-vehicle-categories",
            get(captain::get_driver_vehicle_and_categories),
        )
        .route(
            "/api/driver/{driver_id}/preferences",
            get(preferences::get_preferences)
                .put(preferences::update_preferences),
        )
        .route(
            "/api/profile/bio",
            axum::routing::put(preferences::save_bio),
        )
        .route(
            "/api/profile/personal-details",
            axum::routing::put(update_personal_details),
        )
        .route(
            "/api/profile/device-info",
            axum::routing::put(update_device_info),
        )
        .route(
            "/api/v1/driver/{driver_id}/rating-summary",
            get(rating::get_rating_summary),
        )
        .route(
            "/api/customer/profile",
            get(customer::get_customer_profile)
                .patch(customer::patch_customer_profile),
        )
        .route(
            "/api/customer/profile/address",
            axum::routing::put(customer::upsert_customer_address),
        )
        .route(
            "/api/customer/ride-history",
            get(customer::get_ride_history),
        )
        .route(
            "/api/customer/rides/{ride_id}",
            get(customer::get_ride_detail),
        )
        .route(
            "/api/customer/link-google",
            axum::routing::put(customer::link_google),
        )
}

pub fn auth_handlers() -> Router {
    Router::new()
        .route("/create-profile/{type}", post(create_profile))
        .route("/login-customer", post(customer::login_customer))
        .route("/login-driver", post(captain::login_driver))
        .route("/api/2fa/send", post(two_factor::send_otp))
        .route("/api/2fa/verify", post(two_factor::verify_otp))
}

/// Empty handler for OPTIONS routes
async fn options() {}
