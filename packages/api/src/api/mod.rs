pub mod admin;
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
pub mod referral;
pub mod ride_fare;
pub mod ride_request;
pub mod rides;
pub mod two_factor;
pub mod wallet;

use std::sync::Arc;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware as axum_mw,
    routing::{delete, get, post, put},
};
use nearby_drivers::get_nearby_drivers;
use profile::{create_profile, update_device_info, update_personal_details};
use provider::create_driver;
use redis_store::rate_limit::Policy;
use rides::{
    accept_ride_request, cancel_ride, create_ride, end_ride,
    ride_fair_estimation, send_ride_request, update_location_coordinates,
};

use crate::{
    APIContext,
    api::driver::payment,
    middleware::rate_limit::{RateLimitState, rate_limit},
};

pub fn handlers(ctx: Arc<APIContext>) -> Router {
    // Parse the trusted-proxy list once; reuse across every rate-limit state.
    let proxies = ctx.config.parsed_trusted_proxies();

    // Pre-build the rate-limit states once so the same `Arc<APIContext>` and
    // `Policy` are reused across requests rather than reconstructed per call.
    let rl_user = RateLimitState::new(ctx.clone(), Policy::user_default())
        .with_trusted_proxies(proxies.clone());
    let rl_expensive = RateLimitState::new(ctx.clone(), Policy::expensive())
        .with_trusted_proxies(proxies.clone());
    let rl_high_freq =
        RateLimitState::new(ctx.clone(), Policy::high_frequency())
            .with_trusted_proxies(proxies);

    // Wrap each state in an axum middleware layer once so the per-route
    // `.layer()` calls below stay short.
    let user_layer = axum_mw::from_fn_with_state(rl_user.clone(), rate_limit);
    let expensive_layer =
        axum_mw::from_fn_with_state(rl_expensive.clone(), rate_limit);
    let high_freq_layer =
        axum_mw::from_fn_with_state(rl_high_freq.clone(), rate_limit);

    Router::new()
        .route(
            "/create-driver",
            post(create_driver).layer(user_layer.clone()),
        )
        .route(
            "/logout-driver",
            put(captain::logout_driver).layer(user_layer.clone()),
        )
        .route(
            "/update-location-coordinates",
            post(update_location_coordinates).layer(high_freq_layer.clone()),
        )
        .route(
            "/go-online",
            post(driver::go_online).layer(user_layer.clone()),
        )
        .route(
            "/go-offline",
            post(driver::go_offline).layer(user_layer.clone()),
        )
        .route(
            "/create-ride/{is_ride_on_free_trial}",
            post(create_ride).layer(user_layer.clone()),
        )
        // .route("/get-ride-details", get(get_ride_details))
        // .route("/get-driver-details", get(get_driver_details))
        .route(
            "/end-ride/{ride_id}/end",
            delete(end_ride).layer(user_layer.clone()),
        )
        .route(
            "/search-for-nearby-drivers/{region}",
            get(get_nearby_drivers).layer(expensive_layer.clone()),
        )
        .route(
            "/ride-fair-estimation",
            post(ride_fair_estimation).layer(expensive_layer.clone()),
        )
        .route(
            "/send-ride-request",
            post(send_ride_request).layer(user_layer.clone()),
        )
        .route(
            "/api/cancel-ride/{ride_id}",
            post(cancel_ride).layer(user_layer.clone()),
        )
        .route(
            "/api/rider-cancel-ride-request/{ride_id}",
            post(rides::rider_cancel_ride_request).layer(user_layer.clone()),
        )
        .route(
            "/accept-ride-request/{ride_id}/{vc}/accept",
            post(accept_ride_request).layer(user_layer.clone()),
        )
        .route(
            "/get-accepted-ride-by-driver/{ride_id}/{driver_id}/ride-details",
            get(rides::get_accepted_ride_by_driver).layer(user_layer.clone()),
        )
        .route(
            "/driver/arrived-at-pickup-location",
            post(rides::driver_arrived_at_pickup_location)
                .layer(user_layer.clone()),
        )
        .route(
            "/create-subscriptions-plan/{plan_id}",
            post(bs_plans::create_bs_plan).layer(user_layer.clone()),
        )
        .route(
            "/get-bs-subscription",
            get(bs_plans::get_bs_subscription).layer(user_layer.clone()),
        )
        .route(
            "/rate-driver/{driver_id}/{ride_id}",
            post(rating::rate_driver).layer(user_layer.clone()),
        )
        .route(
            "/rate-rider/{ride_id}",
            post(rating::rate_rider).layer(user_layer.clone()),
        )
        // Driver documents routes — large body uploads, treat as expensive.
        // DefaultBodyLimit must remain on the route definition (route-level
        // request extension), so it stacks with the rate-limit layer.
        .route(
            "/driver/upload-documents",
            post(docs::handle_docs_upload)
                .options(options)
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
                .layer(expensive_layer.clone()),
        )
        .route(
            "/get-driver-documents/{path_id}",
            get(docs::get_driver_document).layer(user_layer.clone()),
        )
        .route(
            "/save-driver-identity-documents/{id_type}",
            post(docs::save_driver_identity_documents)
                .layer(expensive_layer.clone()),
        )
        .route(
            "/save-driver-documents/{document_type}",
            post(docs::save_driver_documents).layer(expensive_layer.clone()),
        )
        .route(
            "/save-vehicle-info/v-details",
            post(docs::save_vehicle_info).layer(user_layer.clone()),
        )
        .route(
            "/set-driver-photo",
            post(captain::set_driver_photo).layer(expensive_layer.clone()),
        )
        .route(
            "/get-driver-simple-profile",
            get(captain::get_driver_simple_profile).layer(user_layer.clone()),
        )
        .route(
            "/driver/subscription-status",
            get(captain::get_driver_subscription_status)
                .layer(user_layer.clone()),
        )
        .route(
            "/set-driver-has-completed-onboarding",
            post(captain::set_driver_has_completed_onboarding)
                .layer(user_layer.clone()),
        )
        .route(
            "/driver/categories",
            get(captain::get_driver_categories)
                .put(captain::set_driver_categories)
                .layer(user_layer.clone()),
        )
        .route("/test-job", post(bs_plans::test_get_bs_plans)) // TODO:: to be romoved in production
        // .route("/track-driver-on-ride-location/:ride_id", get(track_driver_on_ride_location))
        // Payment endpoints — touch external M-Pesa; treat as expensive to
        // bound both our outbound load AND user-side abuse of expensive paths.
        .route(
            "/payment/charge-phone-number/{sub_id}",
            post(payment::charge_phone_number).layer(expensive_layer.clone()),
        )
        .route(
            "/payment/confirm-payment/{chekout_req_id}",
            get(payment::confirm_payment).layer(user_layer.clone()),
        )
        .route(
            "/confirm-collected-cash/{ride_id}/{is_discounted}/{discount}",
            post(driver::confirm_collected_cash).layer(user_layer.clone()),
        )
        .route(
            "/api/get-driver-daily-earnings/{date}",
            get(driver::get_driver_daily_earnings).layer(user_layer.clone()),
        )
        .route(
            "/api/get-driver-weekly-earnings-report/{date}",
            get(driver::get_driver_weekly_earnings).layer(user_layer.clone()),
        )
        .route(
            "/api/get-driver-vehicle-categories",
            get(captain::get_driver_vehicle_and_categories)
                .layer(user_layer.clone()),
        )
        .route(
            "/api/driver/{driver_id}/preferences",
            get(preferences::get_preferences)
                .put(preferences::update_preferences)
                .layer(user_layer.clone()),
        )
        .route(
            "/api/profile/bio",
            axum::routing::put(preferences::save_bio).layer(user_layer.clone()),
        )
        .route(
            "/api/profile/personal-details",
            axum::routing::put(update_personal_details)
                .layer(user_layer.clone()),
        )
        .route(
            "/api/profile/device-info",
            axum::routing::put(update_device_info).layer(user_layer.clone()),
        )
        .route(
            "/api/v1/driver/{driver_id}/rating-summary",
            get(rating::get_rating_summary).layer(user_layer.clone()),
        )
        .route(
            "/api/customer/profile",
            get(customer::get_customer_profile)
                .patch(customer::patch_customer_profile)
                .layer(user_layer.clone()),
        )
        .route(
            "/api/customer/profile/address",
            axum::routing::put(customer::upsert_customer_address)
                .layer(user_layer.clone()),
        )
        .route(
            "/api/customer/ride-history",
            get(customer::get_ride_history).layer(user_layer.clone()),
        )
        .route(
            "/api/customer/rides/{ride_id}",
            get(customer::get_ride_detail).layer(user_layer.clone()),
        )
        .route(
            "/api/customer/link-google",
            axum::routing::put(customer::link_google).layer(user_layer.clone()),
        )
        .route(
            "/api/rides/{ride_id}/fare",
            get(ride_fare::get_current_fare).layer(user_layer.clone()),
        )
        .route(
            "/api/rides/{ride_id}/fare/history",
            get(ride_fare::get_fare_history).layer(user_layer.clone()),
        )
        .route(
            "/api/rides/{ride_id}/fare/components/{key}",
            get(ride_fare::get_fare_component).layer(user_layer.clone()),
        )
        // Driver referral programme + the wallet cash rewards land in.
        .route(
            "/driver/referral/code",
            get(referral::get_referral_code).layer(user_layer.clone()),
        )
        .route(
            "/driver/referral/stats",
            get(referral::get_referral_stats).layer(user_layer.clone()),
        )
        .route(
            "/driver/referral/history",
            get(referral::get_referral_history).layer(user_layer.clone()),
        )
        .route(
            "/driver/wallet",
            get(wallet::get_wallet).layer(user_layer.clone()),
        )
        .route(
            "/driver/wallet/transactions",
            get(wallet::get_wallet_transactions).layer(user_layer),
        )
}

pub fn auth_handlers(ctx: Arc<APIContext>) -> Router {
    // Auth endpoints share one strict per-IP, fail-CLOSED policy. A Redis
    // outage must not let brute-force attempts through these routes.
    let proxies = ctx.config.parsed_trusted_proxies();
    let rl_auth = RateLimitState::new(ctx, Policy::auth_strict())
        .with_trusted_proxies(proxies);
    let auth_layer = axum_mw::from_fn_with_state(rl_auth, rate_limit);

    Router::new()
        .route(
            "/create-profile/{type}",
            post(create_profile).layer(auth_layer.clone()),
        )
        .route(
            "/login-customer",
            post(customer::login_customer).layer(auth_layer.clone()),
        )
        .route(
            "/login-driver",
            post(captain::login_driver).layer(auth_layer.clone()),
        )
        .route(
            "/api/2fa/send",
            post(two_factor::send_otp).layer(auth_layer.clone()),
        )
        .route(
            "/api/2fa/verify",
            post(two_factor::verify_otp).layer(auth_layer),
        )
}

/// Empty handler for OPTIONS routes
async fn options() {}
