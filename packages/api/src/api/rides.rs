use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::{StatusCode, header::HeaderMap},
};
use chrono::Utc;
use futures::future::join_all;
use geo::{Distance, Haversine, Point};
use num_traits::ToPrimitive;
use redis_store::{
    events::{
        DriverArrivedEvent, DriverArrivedEventPayload, DriverArrivedPayload,
        EventPayload, GeoLocation, RIDE_EVENTS_STREAM, RideCancelPayload,
        RideCanceledEvent, RideEndEvent, RideEndEventPayload, RideEndPayload,
        RideStartEvent, RideStartEventPayload, RideStartPayload,
    },
    r_types::GeoPoint,
};
use serde::{Deserialize, Serialize};
use std::{pin::Pin, sync::Arc};
use time::OffsetDateTime;
// use tokio::time::Duration;
use tracing::{error_span, info, info_span, warn_span};
use utils::{
    Result, convert_seconds,
    gen_strings::{self, ulid_string},
    hashing_algo::{
        DecryptingRecord, decrypt_data, deserialize_data_from_slice,
        extract_contact_info,
    },
    meters_to_km, seconds_to_minutes,
};

use crate::{
    APIContext,
    api::ride_request::{self, RequestDriver, RequestRideData, RiderDataInfo},
    api_responses::responces::Response,
    cache::{
        keys::{
            driver_location_info_key, ride_path_key_id,
            saving_driver_location_record_key,
        },
        read_writer::{
            get_and_delete_on_going_ride_loactions, get_driver_location_info,
            get_llen, get_on_going_ride_coordinates, get_ride_info,
            ride_clean_up, set_driver_location_info,
            set_on_going_ride_coordinates, set_ride_info, with_redis_lock,
        },
    },
    dispatch::state_machine::{
        DispatchEvent, DriverResponse, RideSearchResult, find_nearest_driver,
    },
    queries::{
        customer::get_customer_profile::GetRiderProfile,
        driver_stats::DriverStatsQueries,
        drivers::DriverQueries,
        get_fair_estimates::{FareEstimate, GetFairEstimates},
        ride::RideQueries,
        ride_fare::RideFareQueries,
    },
    request::RidesApiClient,
    schemas::{location, ride_request::RideRequestStatus},
    simd_json::parse_from_string,
    tracking::route_change::Polyline,
    types::*,
};

#[derive(Debug, Serialize)]
pub struct LocationUpdateResponse {
    pub is_success: i32,
}

#[derive(Debug, Deserialize)]
pub struct PathParams {
    pub ride_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverLocationUpdate {
    pub lat_lng: GeoPoint,
    pub speed: Option<VelocityInMetersPerSec>,
    pub accuracy: Option<AccuracyThreshold>,
    pub distance: Option<Meters>,
    pub timestamp: TimeStamp,
}

pub async fn update_location_coordinates(
    headers: HeaderMap,
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Query(PathParams { ride_id: _ }): Query<PathParams>,
    Json(mut driver_new_locations): Json<Vec<DriverLocationUpdate>>,
) -> Result<Json<LocationUpdateResponse>, AppError> {
    let driver_id = DriverId(driver_id);

    // Categories are sent by the app as a comma-separated string in the "vc" header (e.g., "sedan,SUV")
    let categories: Vec<VehicleCategory> = headers
        .get("vc")
        .and_then(|v| v.to_str().ok())
        .map(|s| {
            s.split(',')
                .filter_map(|part| part.trim().parse::<VehicleCategory>().ok())
                .collect()
        })
        .unwrap_or_default();

    if categories.is_empty() {
        return Err(AppError::NotFound(
            "Driver has no vehicle categories set".to_string(),
        ));
    }

    driver_new_locations.sort_by(|a, b| {
        let TimeStamp(a_time) = a.timestamp;
        let TimeStamp(b_time) = b.timestamp;
        a_time.cmp(&b_time)
    });

    driver_new_locations.dedup_by(|a, b| {
        (a.lat_lng.lat - b.lat_lng.lat).0.abs() < 0.000001
            && (a.lat_lng.lon - b.lat_lng.lon).0.abs() < 0.000001
    });

    let driver_location_info =
        get_driver_location_info(&ctx.redis.clone(), driver_id.clone()).await;

    let location_input = filter_loctions(
        driver_new_locations,
        driver_location_info.as_ref(),
        |item, ref_item| {
            let TimeStamp(item_time) = item.timestamp;
            let TimeStamp(ref_item_time) = ref_item.position_info.timestamp;
            item_time > ref_item_time
        },
    );

    let last_knonw_location = if let Some(last_location) = location_input.last()
    {
        last_location.clone()
    } else {
        warn_span!("Empty Location Update", driver_id = %driver_id.0)
                .in_scope(|| {
                    tracing::warn!(
                        "Driver location update: Location coordinates can't be empty for driver_id: {}",
                        driver_id.0
                    );
                });
        // Return early if there are no new locations to process
        return Ok(Json(LocationUpdateResponse { is_success: 0 }));
    };

    let exp = OffsetDateTime::now_utc().hour().to_string();

    info!(
        tag = "Locations Update",
        "Received location update {:?} for driver {:?}, categories {:?}",
        &location_input,
        &driver_id,
        &categories
    );

    with_redis_lock(
        ctx.redis.clone(),
        (
            ctx.clone(),
            driver_id.clone(),
            last_knonw_location,
            location_input,
            driver_location_info,
            categories,
        ),
        process_ride_coordinates,
        &saving_driver_location_record_key(exp, driver_id),
        60, // 60 seconds lock
    )
    .await
    .map_err(|err| {
        error_span!("Error updating driver location", error = %err);
        AppError::InternalError(format!(
            "Failed to update driver location: {:?}",
            err
        ))
    })?;

    Ok(Json(LocationUpdateResponse { is_success: 1 }))
}

#[derive(Debug, Deserialize)]
pub struct CreateRideParms {
    pub ride_id: RideId,
}

#[derive(Debug, Serialize)]
pub struct CreateRideResponse {
    pub ride_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FareComponent {
    pub waiting_charge: i32,
    pub toll: i32,
    pub extra_dx: i32,
}
#[derive(Debug, Deserialize)]
pub struct CreateRideRequest {
    pub start_otp: String,
    pub a: GeoPoint,
    pub b: GeoPoint,
    pub fare_component: Option<FareComponent>,
}

pub async fn create_ride(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(client_id): Extension<String>,
    Path(is_ride_on_free_trial): Path<bool>,
    Json(body): Json<CreateRideRequest>,
) -> Result<StatusCode, AppError> {
    let ride_info =
        get_ride_info(&ctx.redis, DriverId(client_id.to_owned())).await;
    if ride_info.is_none() {
        return Err(AppError::InternalError("Ride info not found".to_string()));
    }
    let ride_info = ride_info.unwrap();

    let ride_request = ctx
        .db
        .get_ride_request_by_id_with_location(&ride_info.ride_id)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

    if let Some((ride_request, _location_info)) = ride_request {
        if let Some(otp) = ride_request.otp
            && body.start_otp != otp
        {
            return Err(AppError::InternalError(
                "Invalid OTP provided".to_string(),
            ));
        }

        let estimated_fare = ride_request.fare.to_f64().unwrap_or(0.0);
        let ride_start_event = RideStartEvent {
            event_id: ulid_string(),
            timestamp: Utc::now().timestamp_millis(),
            event_type: "RideStartEvent".to_string(),
            ride_id: ride_request.id,
            driver_id: ride_request.driver_id,
            rider_id: ride_request.customer_id,
            priority: 1,
            ack_required: true,
            event_payload: RideStartEventPayload {
                ride_start: RideStartPayload {
                    start_location: GeoLocation {
                        latitude: body.a.lat.0,
                        longitude: body.a.lon.0,
                        address: "null".to_string(),
                        place_id: "null".to_string(),
                    },
                    destination: GeoLocation {
                        latitude: body.b.lat.0,
                        longitude: body.b.lon.0,
                        address: "null".to_string(),
                        place_id: "null".to_string(),
                    },
                    estimated_fare,
                    vehicle_type: "null".to_string(),
                    vehicle_number: "null".to_string(),
                    driver_info: None,
                    estimated_duration: ride_request
                        .estimated_duration
                        .unwrap_or(0),
                },
            },
        };
        set_ride_info(
            &ctx.redis,
            &DriverId(client_id.to_owned()),
            &RideInfo {
                ride_status: RideRequestStatus::Inprogress,
                ..ride_info.to_owned()
            },
            &ctx.config.exp_ttl,
        )
        .await?;

        ctx.db
            .update_ride_request_status(
                ride_info.ride_id.to_owned(),
                RideRequestStatus::Inprogress,
            )
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        ctx.db
            .start_ride(
                &DriverId(client_id),
                &ride_info.ride_id,
                body.start_otp,
                "KES".into(),
                body.a, // current location
                body.b,
                is_ride_on_free_trial,
                &ride_info.vehicle_category.to_string(), // Pass vehicle category to start_ride query
            )
            .await
            .map_err(|err| {
                AppError::InternalError(format!(
                    "Failed to create ride: {:?}",
                    err
                ))
            })?;

        let (fare_components_json, fare_total) =
            if let Some(fc) = body.fare_component {
                let estimated_fare = ride_request.fare;
                let extra_total = rust_decimal::Decimal::from(
                    fc.waiting_charge + fc.toll + fc.extra_dx,
                );
                let total = estimated_fare + extra_total;
                let json = serde_json::json!({
                    "estimated_fare": estimated_fare,
                    "waiting_charge": fc.waiting_charge,
                    "toll": fc.toll,
                    "extra_dx": fc.extra_dx,
                });
                (json, total)
            } else {
                let estimated = ride_request.fare;
                (
                    serde_json::json!({ "estimated_fare": estimated }),
                    estimated,
                )
            };

        ctx.db
            .insert_ride_fare(
                &ride_info.ride_id.0,
                fare_components_json,
                fare_total,
                "estimated",
                None,
            )
            .await
            .map_err(|err| {
                AppError::InternalError(format!(
                    "Failed to record fare: {:?}",
                    err
                ))
            })?;

        ctx.redis
            .xadd_event(RIDE_EVENTS_STREAM, &ride_start_event)
            .await
            .map_err(|err| {
                AppError::InternalError(format!(
                    "Failed to publish ride start event: {:?}",
                    err
                ))
            })?;
        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound("Ride request not found".to_string()))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RideReq {
    from: RequestRideData,
    to: RequestRideData,
}
#[derive(Debug, Serialize)]
pub struct FairEstimateResponse {
    line_str: Vec<(f64, f64)>,
    distance: f64,
    search_req_id: String,
    duration: (u64, &'static str),
    estimates: Vec<FareEstimate>,
}

pub async fn ride_fair_estimation(
    Extension(_client_id): Extension<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Json(body): Json<RideReq>,
) -> Result<Json<FairEstimateResponse>, AppError> {
    let RideReq { from, to } = body.clone();

    #[cfg(feature = "reqwest-middleware")]
    let (line_str, dx, dr) = {
        let base_url = std::env::var("ROUTES_API_URL")
            .expect("ROUTES_API_URL must be set");
        let route_query = RidesApiClient::new_with_retry(&base_url, None);
        let coords = [
            (from.geo_point.lon.0, from.geo_point.lat.0),
            (to.geo_point.lon.0, to.geo_point.lat.0),
        ];
        let route = route_query
            .get_ride_path_and_distance(
                &coords,
                "overview=full&steps=true&geometries=geojson",
            )
            .await
            .expect("Failed to get route path");
        let route_path_data = parse_from_string(&route);
        match route_path_data {
            Ok((line_str, distance, duration)) => {
                Ok((line_str, meters_to_km(distance), duration))
            }
            Err(err) => Err(AppError::InternalError(err.to_string())),
        }
    }?;

    let vc = find_nearest_driver(
        ctx.redis.clone(),
        from.geo_point,
        &None,
        &Radius(4000.0),
    )
    .await
    .map_err(|err| {
        AppError::InternalError(format!(
            "Failed to find nearest driver: {:?}",
            err
        ))
    })?
    .into_iter()
    .map(|driver| driver.vehicle_category)
    .collect::<Vec<VehicleCategory>>();

    let search_req_id = gen_strings::ulid_string();
    ctx.redis
        .set_key(
            &ride_path_key_id(&search_req_id),
            (line_str.clone(), dx, dr, from.geo_point, to.geo_point),
            ctx.config.default_ttl,
        ) // 36000
        .await
        .map_err(|err| {
            AppError::InternalError(format!(
                "Failed to set ride path: {:?}",
                err
            ))
        })?;

    // log the vehicle categories found
    info_span!("vehicle_categories", ?vc).in_scope(|| {
        tracing::info!("Vehicle categories found: {:?}", vc);
    });

    let fair_estimate = ctx
        .db
        .get_fair_estimate(
            &vc,
            dx as f32,
            0,
            false,
            seconds_to_minutes(dr) as i32,
        )
        .await
        .map_err(|err| {
            AppError::InternalError(format!(
                "Failed to get fare estimate: {:?}",
                err
            ))
        })?;

    Ok(Json(FairEstimateResponse {
        line_str,
        distance: dx,
        search_req_id,
        duration: convert_seconds(dr),
        estimates: fair_estimate,
    }))
}

#[derive(Debug, Deserialize, Clone)]
pub struct SendRideReqQr {
    pub q: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SendRideRequestData {
    from: RequestRideData,
    to: RequestRideData,
    fare: i32,
    dx: f64,
    duration: i32,
    vehicle_type: Option<Vec<VehicleCategory>>,
    radius: Radius,
}

#[derive(Debug, Clone, Serialize)]
pub struct RideRequsetResponse {
    pub resp: Option<(String, String)>,
}
pub async fn send_ride_request(
    Extension(client_id): Extension<String>,
    Extension(ctx): Extension<Arc<APIContext>>,
    Query(q): Query<SendRideReqQr>,
    Json(data): Json<SendRideRequestData>,
) -> Result<Json<RideRequsetResponse>, AppError> {
    // Get Rider profile data
    let rider_profile = ctx
        .db
        .get_rider_small_profile(&client_id)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;
    let rider_data_info = if let Some(rider_profile) = rider_profile {
        let (email, phone_number) = extract_contact_info(
            &rider_profile.contact_data,
            &rider_profile.nonce,
            &rider_profile.encrypted_key,
        )
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

        RiderDataInfo {
            id: rider_profile.id,
            first_name: rider_profile.first_name,
            last_name: rider_profile.last_name,
            rating: rider_profile.rating,
            total_rating_score: rider_profile.total_rating_score,
            email,
            phone_number,
            mobile_country_code: rider_profile.mobile_country_code,
        }
    } else {
        return Err(AppError::NotFound("Rider profile not found".to_string()));
    };

    let request = RequestDriver {
        from: data.from,
        to: data.to,
        fare: data.fare,
        dx: data.dx,
        duration: data.duration,
        vehicle_type: data.vehicle_type,
        radius: data.radius,
        rider_profile: rider_data_info,
    };

    //Register ride request and get back response channel
    let ride_req_id = q.q.clone();
    let response_rx = ctx
        .dispatch_api_manager
        .register_ride_request(ride_req_id.to_string(), client_id.to_string())
        .map_err(|err| {
            tracing::warn!(
                tag = "send_ride_request",
                ride_req_id = %ride_req_id,
                error = %err,
                "Duplicate ride request registration rejected"
            );
            AppError::LockContention
        })?;

    let ride: RideSearchResult = ctx
        .redis
        .get_key(&ride_path_key_id(&q.q))
        .await
        .map_err(|er| AppError::InternalError(er.to_string()))?;

    // Create a dispatch Job
    let dispatch_job = ride_request::DispatchJob {
        request_id: ride_req_id,
        rider_id: client_id.to_string(),
        response_rx,
        request,
        ride_search_result: ride,
    };

    // Enqueue the dispatch job for processing
    ctx.dispatcher_queue.enqueue_dispatch(dispatch_job).await.map_err(
        |err| {
            AppError::InternalError(format!(
                "Failed to enqueue dispatch job: {:?}",
                err
            ))
        },
    )?;

    Ok(Json(RideRequsetResponse {
        resp: Some((q.q, " ".to_string())),
    }))
}

pub async fn rider_cancel_ride_request(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(client_id): Extension<String>,
    Path(ride_id): Path<RideId>,
) -> Result<StatusCode, AppError> {
    tracing::warn!(
        tag = "rider_cancel_ride_request",
        rider_id = %client_id,
        ride_id = %ride_id.0,
        "Rider is cancelling dispatch request"
    );
    match ctx
        .dispatch_api_manager
        .send_driver_response(&ride_id.0, DispatchEvent::CancelRequest)
    {
        Ok(_) => {
            tracing::info!(
                tag = "rider_cancel_ride_request",
                rider_id = %client_id,
                ride_id = %ride_id.0,
                "Cancel signal delivered to dispatcher"
            );
            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::warn!(
                tag = "rider_cancel_ride_request",
                rider_id = %client_id,
                ride_id = %ride_id.0,
                error = %e,
                "Cancel signal failed — dispatch may have already completed"
            );
            // Dispatch already finished (accepted, timed out, or no drivers) — treat as OK
            Ok(StatusCode::OK)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancellRideBody {
    pub ride_path_id: String,
    pub reason: String,
    pub note: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelRideParams {
    pub account_type: AccountType,
}
pub async fn cancel_ride(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(id): Extension<String>,
    Path(ride_id): Path<RideId>,
    Query(by): Query<CancelRideParams>,
    Json(body): Json<CancellRideBody>,
) -> Result<StatusCode, AppError> {
    let ride_request = ctx
        .db
        .get_ride_request_by_id(&ride_id)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

    //Create ride cancel event
    let mut ride_cancel_event = RideCanceledEvent {
        event_id: ulid_string(),
        correlation_id: Some(ulid_string()),
        timestamp: Utc::now().timestamp_millis(),
        event_type: "RideCancelEvent".to_string(),
        ride_id: ride_id.0.to_string(),
        priority: 1,
        ack_required: false,
        event_payload: EventPayload {
            ride_cancel: RideCancelPayload {
                reason: body.reason,
                canceled_by: match by.account_type {
                    AccountType::Customer => 1,
                    AccountType::Driver => 0,
                    AccountType::Admin => 2,
                },
                refund_amount: 0.00,
                cancellation_fee: "0".to_string(),
                note: body.note,
            },
        },
        ..Default::default()
    };
    let events_manager =
        redis_store::events::EventsManger::new(RIDE_EVENTS_STREAM);

    if let Some(ride_request) = ride_request {
        if ride_request.request_status == RideRequestStatus::Canceled
            || ride_request.request_status == RideRequestStatus::Completed
        {
            return Err(AppError::InternalError(
                "Ride already canceled or completed".to_string(),
            ));
        }
        // If account_type is rider don't cancel just clean up so that
        // driver can continue to get location updates
        // delete ride request from db
        // and fire an event to notify driver that ride has been cancelled
        if by.account_type == AccountType::Customer {
            let mut event = ride_cancel_event;
            event.driver_id = ride_request.driver_id.clone();
            event.rider_id = ride_request.customer_id.clone();
            ctx.db
                .delete_ride_request_by_id(&ride_id)
                .await
                .map_err(|err| AppError::InternalError(err.to_string()))?;
            ride_clean_up(
                &ctx.redis,
                &DriverId(ride_request.driver_id),
                &ride_id,
                &body.ride_path_id,
            )
            .await?;
            events_manager
                .publish_event(Some(&event), &ctx.redis)
                .await
                .map_err(|err| {
                    AppError::InternalError(format!(
                        "Failed to publish ride canceled event: {:?}",
                        err
                    ))
                })?;
            return Ok(StatusCode::OK);
        }
        ctx.db
            .update_ride_request_status(
                ride_id.to_owned(),
                RideRequestStatus::Canceled,
            )
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        ctx.db
            .update_driver_stats_rides_cancelled(id.to_owned(), &1)
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        ride_clean_up(
            &ctx.redis,
            &DriverId(id.to_string()),
            &ride_id,
            &body.ride_path_id,
        )
        .await?;
        // publish ride canceled event
        ride_cancel_event.driver_id = id;
        ride_cancel_event.rider_id = ride_request.customer_id;
        events_manager
            .publish_event(Some(&ride_cancel_event), &ctx.redis)
            .await
            .map_err(|err| {
                AppError::InternalError(format!(
                    "Failed to publish ride canceled event: {:?}",
                    err
                ))
            })?;

        Ok(StatusCode::OK)
    } else {
        Err(AppError::NotFound("Ride request not found".to_string()))
    }
}

#[derive(Debug, Serialize)]
pub struct AcceptedRideResp {
    pub pickup_location: Point,
    pub polyline: Option<Vec<(f64, f64)>>,
}
// #[axum_macros::debug_handler]
pub async fn accept_ride_request(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Path((ride_id, vc)): Path<(RideId, VehicleCategory)>,
) -> Result<Json<AcceptedRideResp>, AppError> {
    let response = DriverResponse {
        driver_id: driver_id.to_owned(),
        accepted: true,
        // Calculate actual response time
        response_time_ms: (Utc::now().timestamp_millis()).max(0) as u64,
    };

    match ctx.dispatch_api_manager.send_driver_response(
        &ride_id.0,
        DispatchEvent::DriverResponse(response),
    ) {
        Ok(_) => {
            info!(
                tag = "Ride Accepted",
                "✅✅Driver {} accepted ride request {}", driver_id, ride_id.0
            );
            // get requestedride from db
            let requested_ride =
                ctx.db.get_ride_request_by_id(&ride_id).await.map_err(
                    |err| {
                        tracing::error!(
                            tag = "accept_ride_request",
                            driver_id = %driver_id,
                            ride_id = %ride_id.0,
                            error = %err,
                            "DB error fetching ride request"
                        );
                        AppError::InternalError(err.to_string())
                    },
                )?;

            // check if ride request exist
            if requested_ride.is_none() {
                tracing::error!(
                    tag = "accept_ride_request",
                    driver_id = %driver_id,
                    ride_id = %ride_id.0,
                    "Ride request not found in DB"
                );
                return Err(AppError::NotFound(
                    "Ride request not found".to_string(),
                ));
            }

            let requested_ride = requested_ride.unwrap();

            let ride_search_result: RideSearchResult = ctx
                .redis
                .get_key(&ride_path_key_id(&ride_id.0))
                .await
                .map_err(|er| {
                    tracing::error!(
                        tag = "accept_ride_request",
                        driver_id = %driver_id,
                        ride_id = %ride_id.0,
                        error = %er,
                        "Redis error fetching ride search result"
                    );
                    AppError::InternalError(er.to_string())
                })?;

            // Extract (from -> to), polyline data from ride_search_result
            let (p, _dt, _dr, from, _) = match ride_search_result {
                Some(result) => result,
                None => {
                    tracing::error!(
                        tag = "accept_ride_request",
                        driver_id = %driver_id,
                        ride_id = %ride_id.0,
                        "Ride search result not found in Redis (key may have expired)"
                    );
                    return Err(AppError::InternalError(
                        "Ride search result not found in cache".to_string(),
                    ));
                }
            };

            let ride_info = RideInfo {
                vehicle_category: vc,
                ride_id: ride_id.to_owned(),
                ride_status: RideRequestStatus::Accepted,
                ride_data: Some(RideData::Taxi {
                    pickup_location: (from.lat.0, from.lon.0).into(),
                    polyline: Some(p),
                    polyline_2: None,
                }),
                estimated_pickup_time: Some(
                    requested_ride.estimated_duration_to_pickup.unwrap_or(0)
                        as f64,
                ),
                estimated_pickup_distance: Some(Meters(
                    requested_ride.estimated_distance_to_pickup.unwrap_or(0.0),
                )),
                created_at: TimeStamp(Utc::now()),
            };
            set_ride_info(
                &ctx.redis,
                &DriverId(driver_id.to_owned()),
                &ride_info,
                &ctx.config.exp_ttl,
            )
            .await
            .map_err(|err| {
                tracing::error!(
                    tag = "accept_ride_request",
                    driver_id = %driver_id,
                    ride_id = %ride_id.0,
                    error = ?err,
                    "Failed to set ride info in Redis"
                );
                err
            })?;

            let (pickup_location, polyline, _polyline_2) = ride_info
                .ride_data
                .as_ref()
                .and_then(|ride_data| match ride_data {
                    RideData::Taxi {
                        pickup_location,
                        polyline,
                        polyline_2,
                    } => Some((
                        pickup_location.to_owned(),
                        polyline.clone(),
                        polyline_2.clone(),
                    )),
                    _ => None,
                })
                .ok_or_else(|| {
                    tracing::error!(
                        tag = "accept_ride_request",
                        driver_id = %driver_id,
                        ride_id = %ride_id.0,
                        "pickup_location or polyline missing from ride_data"
                    );
                    AppError::InternalError(
                        "pickup_location and polyline should not be empty"
                            .to_string(),
                    )
                })?;
            //TODO:: notify customer driver on the way

            ctx.db
                .update_ride_request_status(
                    ride_id.to_owned(),
                    RideRequestStatus::Accepted,
                )
                .await
                .map_err(|err| {
                    tracing::error!(
                        tag = "accept_ride_request",
                        driver_id = %driver_id,
                        ride_id = %ride_id.0,
                        error = %err,
                        "DB error updating ride request status to Accepted"
                    );
                    AppError::InternalError(err.to_string())
                })?;

            Ok(Json(AcceptedRideResp {
                pickup_location,
                polyline,
            }))
        }
        Err(err) => {
            // Channel gone means the SM already finished (cleanup_request was called).
            // Check whether THIS driver's accept was already processed (idempotent
            // retry due to mobile network drop) vs a stale offer from a prior dispatch.
            let existing =
                ctx.db.get_ride_request_by_id(&ride_id).await.ok().flatten();

            if let Some(ride) = existing
                && ride.driver_id == driver_id
                && ride.request_status == RideRequestStatus::Accepted
            {
                tracing::info!(
                    tag = "accept_ride_request",
                    driver_id = %driver_id,
                    ride_id = %ride_id.0,
                    "Idempotent accept: ride already accepted by this driver"
                );
                let ride_info =
                    get_ride_info(&ctx.redis, DriverId(driver_id.to_owned()))
                        .await;
                let (pickup_location, polyline) = ride_info
                    .and_then(|info| info.ride_data)
                    .and_then(|rd| match rd {
                        RideData::Taxi {
                            pickup_location,
                            polyline,
                            ..
                        } => Some((pickup_location, polyline)),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        AppError::InternalError(
                            "Ride info not in cache for idempotent accept"
                                .to_string(),
                        )
                    })?;
                return Ok(Json(AcceptedRideResp {
                    pickup_location,
                    polyline,
                }));
            }

            tracing::warn!(
                tag = "accept_ride_request",
                driver_id = %driver_id,
                ride_id = %ride_id.0,
                error_message = ?err,
                "Ride offer no longer active (timed out, cancelled, or already accepted)"
            );
            Err(AppError::Gone(
                "Ride offer is no longer available".to_string(),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AcceptedRideData {
    pub ride_id: RideId,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GetAcceptedRideByDriverResp {
    pub id: String,
    pub driver_id: DriverId,
    pub customer_id: String,
    pub fare: sea_orm::prelude::Decimal,
    pub request_status: RideRequestStatus,
    pub estimated_distance_to_pickup: Option<f64>,
    pub estimated_duration_to_pickup: Option<i32>,
    pub estimated_distance: Option<f64>,
    pub estimated_duration: Option<i32>,
    pub search_request_valid_till: Option<chrono::DateTime<Utc>>,
    pub start_time: Option<OffsetDateTime>,
    pub created_at: chrono::DateTime<Utc>,
    pub message: Option<String>,
    pub otp: Option<String>,
    pub from: Option<location::Model>,
    pub to: Option<location::Model>,
    // From profile
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub face_image_id: Option<String>,
    pub email: String,
    pub phone: String,
    pub verified: Option<bool>,
    pub is_new: Option<bool>,
    // From vehicle
    pub plate_number: Option<String>,
    pub vehicle_type: Option<String>,
    pub color: Option<String>,
    pub rating: Option<f64>,
    pub total_ratings: Option<i32>,
    pub p1: Option<Vec<(f64, f64)>>,
    pub p2: Option<Vec<(f64, f64)>>,
}
pub async fn get_accepted_ride_by_driver(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(_): Extension<String>,
    Path((ride_id, _driver_id)): Path<(RideId, DriverId)>,
) -> Result<Response<GetAcceptedRideByDriverResp>, AppError> {
    let ride_data_resp =
        ctx.db.get_current_ride_info(ride_id).await.map_err(|err| {
            AppError::InternalError(format!(
                "Error geting Ride info data: {:?}",
                err
            ))
        })?;
    // If ride_data is Some, get driver info
    if let Some(ride_data) = ride_data_resp? {
        let driver_id = DriverId(ride_data.0.driver_id.to_owned());
        let driver_info = ctx
            .db
            .get_driver_profile_and_rating(driver_id.to_owned())
            .await
            .map_err(|err| {
                AppError::InternalError(format!(
                    "Error geting Driver info data: {:?}",
                    err
                ))
            })?;

        info!(
            tag = "Accepted Ride Data",
            "Ride data: {:?}, Driver info: {:?}", ride_data, driver_info
        );

        //get ride_info and driver info from redis
        let ride_info_redis =
            get_ride_info(&ctx.redis, driver_id).await.ok_or_else(|| {
                AppError::InternalError(
                    "Failed to get ride info from redis".to_string(),
                )
            })?;

        let (ride_model, from, to, driver_bundle) =
            match (Some(ride_data), driver_info) {
                (Some((ride_model, from, to)), Some(driver_bundle)) => {
                    (ride_model, from, to, driver_bundle)
                }
                _ => {
                    return Err(AppError::NotFound(
                        "Ride or driver data not found".to_string(),
                    ));
                }
            };

        // Use the per-record encrypted_key blob (stored at registration time)
        // so kms:Decrypt recovers the correct data key after any restart.
        let buffer = decrypt_data(&DecryptingRecord {
            ciphertext: &driver_bundle.contact_data,
            nonce: &driver_bundle.nonce,
            key_id: String::new(),
            encrypted_key: &driver_bundle.encrypted_key,
        })
        .await
        .map_err(|error| AppError::InternalError(error.to_string()))?;

        let contact = deserialize_data_from_slice(&buffer)
            .map_err(|error| AppError::InternalError(error.to_string()))?;

        let res = GetAcceptedRideByDriverResp {
            id: ride_model.id,
            driver_id: DriverId(ride_model.driver_id),
            customer_id: ride_model.customer_id,
            fare: ride_model.fare,
            request_status: ride_model.request_status,
            estimated_distance_to_pickup: ride_model
                .estimated_distance_to_pickup,
            estimated_duration_to_pickup: ride_model
                .estimated_duration_to_pickup,
            estimated_distance: ride_model.estimated_distance,
            estimated_duration: ride_model.estimated_duration,
            search_request_valid_till: ride_model.search_request_valid_till,
            start_time: ride_model.start_time,
            created_at: ride_model.created_at,
            message: ride_model.message,
            otp: ride_model.otp,
            from,
            to,
            first_name: driver_bundle.first_name,
            last_name: driver_bundle.last_name,
            face_image_id: driver_bundle.face_image_id,
            verified: driver_bundle.verified,
            is_new: driver_bundle.is_new,
            email: contact.email,
            phone: contact.phone,
            plate_number: driver_bundle.plate_number,
            vehicle_type: driver_bundle.vehicle_type,
            color: driver_bundle.color,
            rating: driver_bundle.rating,
            total_ratings: driver_bundle.total_ratings,
            p1: ride_info_redis.ride_data.as_ref().and_then(|ride_data| {
                match ride_data {
                    RideData::Taxi { polyline, .. } => polyline.clone(),
                    _ => None,
                }
            }),
            p2: ride_info_redis.ride_data.as_ref().and_then(|ride_data| {
                match ride_data {
                    RideData::Taxi { polyline_2, .. } => polyline_2.clone(),
                    _ => None,
                }
            }),
        };
        Ok(Response::OK(res))
    } else {
        Err(AppError::NotFound("Ride not found".to_string()))
    }
}

#[derive(Debug, Deserialize)]
pub struct DriverArrived {
    pub msg: Option<String>,
    pub ride_id: RideId,
    pub at_location: GeoPoint,
    pub arrived_at: i64,
}

//When driver arrives at pickup location
//Notify Customer driver arrived.
pub async fn driver_arrived_at_pickup_location(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Json(body): Json<DriverArrived>,
) -> Result<Response<serde_json::Value>, (StatusCode, AppError)> {
    let now = Utc::now().timestamp_millis();
    let ride =
        ctx.db.get_ride_request_by_id(&body.ride_id).await.map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppError::InternalError(err.to_string()),
            )
        })?;
    if let Some(ride) = ride {
        if body.arrived_at > now + 1000 {
            return Err((
                StatusCode::BAD_REQUEST,
                AppError::InternalError(
                    "Invalid arrived_at: future timestamp".to_string(),
                ),
            ));
        }

        let ride_arrived_event = DriverArrivedEvent {
            event_id: ulid_string(),
            timestamp: now,
            event_type: "DriverArrivedEvent".to_string(),
            ride_id: ride.id.to_string(),
            driver_id: driver_id.to_owned(),
            rider_id: ride.customer_id.to_owned(),
            priority: 1,
            ack_required: false,
            event_payload: DriverArrivedEventPayload {
                driver_arrived: DriverArrivedPayload {
                    arrival_location: GeoLocation {
                        latitude: body.at_location.lat.0,
                        longitude: body.at_location.lon.0,
                        address: "null".to_string(),
                        place_id: "null".to_string(),
                    },
                    actual_arrival_time: body.arrived_at,
                    wait_time_seconds: 0,
                },
            },
        };
        let events_manager =
            redis_store::events::EventsManger::new(RIDE_EVENTS_STREAM);
        events_manager
            .publish_event(Some(&ride_arrived_event), &ctx.redis)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppError::InternalError(format!(
                        "Failed to publish driver arrived event: {:?}",
                        err
                    )),
                )
            })?;

        let driver_display_name =
            ctx.db.get_driver_display_name(&driver_id).await;

        // Notify the rider that their driver has arrived
        let customer_id = ride.customer_id.clone();
        crate::notif::spawn_notify(
            ctx.db.clone(),
            ctx.notif.clone(),
            customer_id.clone(),
            move |b| {
                b.title("Your driver has arrived! 🚗")
                    .message(format!(
                        "{} is waiting at your pickup location.",
                        driver_display_name
                    ))
                    .android_channel("on-driver-arrival")
                    .android_color("#ed1380")
                    .android_tag(format!("driver-arrival-{}", customer_id))
                    .click_action("OPEN_RIDE_TRACKING")
                    .high_priority()
                    .content_available()
            },
        );

        Ok(Response::OK(
            serde_json::json!({ "arrived_at": body.arrived_at }),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            AppError::NotFound("Ride not found".to_string()),
        ))
    }
}

#[derive(Debug, Deserialize)]
pub struct EndRideBody {
    pub ride_path_id: String,
    pub geo_point: GeoPoint,
}

pub async fn end_ride(
    Extension(ctx): Extension<Arc<APIContext>>,
    Extension(driver_id): Extension<String>,
    Path(ride_id): Path<RideId>,
    Json(body): Json<EndRideBody>,
) -> Result<StatusCode, AppError> {
    // get ride data from database
    let ride = ctx
        .db
        .ride_exist(&ride_id)
        .await
        .map_err(|err| AppError::InternalError(err.to_string()))?;

    if let Some(ride) = ride {
        ctx.db
            .update_ride_request_status(
                ride_id.to_owned(),
                RideRequestStatus::Completed,
            )
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;
        // increment the stats.total_rides
        ctx.db
            .update_driver_stats_total_rides(driver_id.to_owned(), &1)
            .await
            .map_err(|err| AppError::InternalError(err.to_string()))?;
        let mut location_points_for_rides = get_on_going_ride_coordinates(
            &ctx.redis,
            &DriverId(driver_id.to_owned()),
            &ride_id,
            &20,
        )
        .await
        .map_err(|err| {
            AppError::InternalError(format!(
                "Failed to get ride coordinates: {:?}",
                err
            ))
        })?;

        ride_clean_up(
            &ctx.redis,
            &DriverId(driver_id.to_owned()),
            &ride_id,
            &body.ride_path_id,
        )
        .await?;

        // Re-add driver to the dispatch pool so they can receive new ride requests.
        // Without this, the driver remains absent from drivers:pool after confirm_driver
        // removed them at ride acceptance, making acquisition fail despite geo-search finding them.
        if let Err(e) =
            ctx.driver_pool_manager.add_driver(&driver_id, 1.0).await
        {
            tracing::error!(driver_id = %driver_id, error = ?e, "Failed to re-add driver to pool after ride completion");
        }

        location_points_for_rides.push(body.geo_point);
        ctx.r_tx
        .send((
            ride_id.to_owned(),
            DriverId(driver_id),
            location_points_for_rides,
        ))
        .await
        .map_err(|err| {
            AppError::InternalError(format!(
                "Failed to send ride coordinates to process_on_going_ride_coordinates: {:?}",
                err
            ))
        })?;
        let ride_end_event = RideEndEvent {
            event_id: ulid_string(),
            timestamp: Utc::now().timestamp_millis(),
            event_type: "RideEndEvent".to_string(),
            ride_id: ride.id,
            driver_id: ride.driver_id,
            rider_id: ride.customer_id,
            priority: 1,
            ack_required: true,
            event_payload: RideEndEventPayload {
                ride_end: RideEndPayload {
                    end_location: GeoLocation {
                        latitude: body.geo_point.lat.0,
                        longitude: body.geo_point.lon.0,
                        address: "null".to_string(),
                        place_id: "null".to_string(),
                    },
                    distance_km: ride.traveled_distance,
                    duration_seconds: 0, //TODO: calculate duration
                    final_fare: ride
                        .fare
                        .map(|f| f.to_f64().unwrap_or(0.0))
                        .unwrap_or(0.0),
                    rider_rating: None,
                    driver_rating: None,
                },
            },
        };
        let events_manager =
            redis_store::events::EventsManger::new(RIDE_EVENTS_STREAM);
        events_manager
            .publish_event(Some(&ride_end_event), &ctx.redis)
            .await
            .map_err(|err| {
            AppError::InternalError(format!(
                "Failed to publish ride end event: {:?}",
                err
            ))
        })?;
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

type RideParams = (
    Arc<APIContext>,
    DriverId,
    DriverLocationUpdate,
    Vec<DriverLocationUpdate>,
    Option<DriverLocationIfo>,
    Vec<VehicleCategory>,
);

fn next_notification_state(
    ride_status: Option<&RideRequestStatus>,
    current: Option<RideNotificationState>,
    distance_to_pickup: Option<Meters>,
) -> (Option<f64>, Option<RideNotificationState>) {
    let Some(RideRequestStatus::Accepted) = ride_status else {
        return (None, None);
    };
    let Some(current) = current else {
        return (None, None);
    };
    let Some(distance) = distance_to_pickup else {
        return (None, Some(current));
    };

    let target = match distance.0 {
        d if d <= 40.0 => RideNotificationState::DriverArrived,
        d if d <= 100.0 => RideNotificationState::DriverArriving,
        _ => RideNotificationState::DriverOnTheWay,
    };

    // Notification state only moves forward.
    (Some(distance.0), Some(target.max(current)))
}

/// Processes the ride coordinates and updates the driver location info in Redis.
async fn process_ride_coordinates(params: RideParams) -> Result<(), AppError> {
    let (
        ctx,
        driver_id,
        current_driver_location,
        driver_new_locations,
        driver_location_info,
        categories,
    ) = params;
    let vehicle_category = *categories.first().ok_or_else(|| {
        AppError::NotFound("Driver has no vehicle categories set".to_string())
    })?;
    // get ride info
    let ride_info = get_ride_info(&ctx.redis, driver_id.clone()).await;

    let ride_id = ride_info.as_ref().map(|info| info.ride_id.clone());

    let ride_status = ride_info.as_ref().map(|info| info.ride_status);

    let ride_data =
        ride_info.as_ref().map(|info| info.ride_data.clone()).and_then(|r| r);

    let distance_notification_status = driver_location_info
        .as_ref()
        .map(|info| info.ride_notification_state)
        .and_then(|r| r);

    let ride_polyline =
        ride_data.as_ref().and_then(|ride_data| match ride_data {
            RideData::Taxi { polyline, .. } => polyline.clone(),
            _ => None,
        });

    let driver_last_recorded_location =
        driver_location_info.as_ref().map(|info| info.position_info.clone());

    let location =
        if let Some(RideRequestStatus::Inprogress) = ride_status.as_ref() {
            filter_locations_based_on_accuracy(
                LocationFilterConfig {
                    min_accuracy: AccuracyThreshold(50.0),
                    min_distance: 10.0,
                },
                driver_new_locations,
                driver_last_recorded_location.as_ref(),
            )
        } else {
            driver_new_locations
        };

    let position_info = DriverLocation {
        timestamp: current_driver_location.timestamp,
        location: current_driver_location.lat_lng,
        driver_id: driver_id.clone(),
        vehicle_category,
        distance: current_driver_location.distance,
    };

    let (dst, r_status) = next_notification_state(
        ride_status.as_ref(),
        distance_notification_status,
        current_driver_location.distance,
    );

    #[allow(clippy::type_complexity)]
    let mut job_tasks: Vec<
        Pin<Box<dyn Future<Output = Result<(), AppError>> + Send>>,
    > = Vec::new();

    // If there's no ongoing ride or ride_status === RequestStatus::New, just update the location and return
    if ride_id.is_none() || ride_status == Some(RideRequestStatus::New) {
        // Fan-out location to any additional categories the driver is registered for.
        for &cat in categories.iter() {
            ctx.tx
                .send(DriverLocationEvent {
                    location_profile: LocationProfile {
                        vehicle_category: cat,
                        created_at: TimeStamp(Utc::now()),
                    },
                    latitude: current_driver_location.lat_lng.lat,
                    longitude: current_driver_location.lat_lng.lon,
                    timestamp: current_driver_location.timestamp,
                    driver_id: driver_id.clone(),
                })
                .await
                .map_err(|err| {
                    AppError::InternalError(format!(
                        "Sending category location to drainer failed: {:?}",
                        err
                    ))
                })?;
        }
        let async_job = async {
            set_driver_location_info(
                &ctx.redis,
                driver_location_info_key(driver_id.to_owned()),
                &position_info,
                &None,
                &None,
                &None,
                &ctx.config.exp_ttl,
            )
            .await?;
            Ok::<(), AppError>(())
        };
        job_tasks.push(Box::pin(async_job));

        join_all(job_tasks)
            .await
            .into_iter()
            .try_for_each(Result::from)
            .map_err(|err| {
                error_span!("Error processing job tasks", error = %err);
                AppError::InternalError(format!(
                    "Failed to process job tasks: {:?}",
                    err
                ))
            })?;

        return Ok(());
    }

    tracing::info_span!(
        "INPROGESS RIDE STATUS",
        location = ?location,
        driver_id = %driver_id.0,
        ride_id = ?ride_id,
        ride_status = ?ride_status,
        distance_notification_state = ?distance_notification_status,
        vehicle_category = ?vehicle_category,
    )
    .in_scope(|| {
        tracing::info!("RIDE STATUS: {:?}", ride_status);
    });

    let update_driver_location_info = async {
        set_driver_location_info(
            &ctx.redis,
            driver_location_info_key(driver_id.to_owned()),
            &position_info,
            &ride_status.clone(),
            &r_status,
            &Some(Meters(dst.unwrap_or_default())),
            &ctx.config.exp_ttl,
        )
        .await?;
        Ok::<(), AppError>(())
    };

    job_tasks.push(Box::pin(update_driver_location_info));

    if let (Some(ride_id), Some(RideRequestStatus::Inprogress)) =
        (ride_id.as_ref(), ride_status.as_ref())
    {
        let geopoints = location
            .iter()
            .map(|data| GeoPoint {
                lat: data.lat_lng.lat,
                lon: data.lat_lng.lon,
            })
            .collect::<Vec<GeoPoint>>();
        let llen = get_llen(&ctx.redis, &driver_id, ride_id).await?;
        if llen + geopoints.len() as i64 > 10 {
            let mut on_ride_data = get_and_delete_on_going_ride_loactions(
                &ctx.redis, &driver_id, ride_id, &llen,
            )
            .await?;

            on_ride_data.extend(geopoints.clone());

            info!(
                tag = "Draining Location",
                "draining location to database, {}, geo_point {:?}",
                geopoints.len() as i64 + llen,
                on_ride_data
            );

            let driver_id_clone = driver_id.clone();
            let ctx_clone = ctx.clone();
            let sync_ride_coordinates_ftr = async move {
                ctx_clone
                    .r_tx
                    .send((ride_id.clone(), driver_id_clone, on_ride_data))
                    .await
                    .map_err(|err| {
                        AppError::InternalError(format!(
                            "Sendig Jobs Task to process_on_going_ride_coordinates failed: {:?}",
                            err
                        ))
                    })?;
                Ok::<(), AppError>(())
            };
            job_tasks.push(Box::pin(sync_ride_coordinates_ftr));
        } else {
            let save_ongoing_ride_coords = async {
                set_on_going_ride_coordinates(
                    &ctx.redis,
                    &ctx.config.exp_ttl,
                    &driver_id,
                    ride_id,
                    geopoints,
                )
                .await?;
                Ok::<(), AppError>(())
            };

            job_tasks.push(Box::pin(save_ongoing_ride_coords));
        }
    }

    if let Some(polyline) = ride_polyline {
        let polyline = polyline
            .iter()
            .map(|(lon, lat)| GeoPoint {
                lat: Latitude(*lat),
                lon: Longitude(*lon),
            })
            .collect::<Vec<GeoPoint>>();

        let po_lyn = Polyline::new(polyline);
        let closest_point =
            po_lyn.find_closest_point(current_driver_location.lat_lng);
        let dx = Haversine::distance(
            Point::new(
                current_driver_location.lat_lng.lat.0,
                current_driver_location.lat_lng.lon.0,
            ),
            Point::new(closest_point.lat.0, closest_point.lon.0),
        );
        if dx > 100.0 {
            error_span!(
                "Route Change Detected!",
                driver_id = %driver_id.0,
                ride_id = ?ride_id,
                vehicle_category = ?vehicle_category,
            )
            .in_scope(|| {
                tracing::warn!(
                    "Route change detected! for Driver ID: {}, Ride ID: {:?}, Vehicle Category: {:?}, at Distance of: {:?}m, Location: {:?}, Closest  Point: {:?}",
                    driver_id.0,
                    ride_id,
                    vehicle_category,
                    dx,
                    current_driver_location.lat_lng,
                    closest_point
                );
            });
        }
    }

    join_all(job_tasks).await.into_iter().try_for_each(Result::from).map_err(
        |err| {
            error_span!("Error processing job tasks", error = %err);
            AppError::InternalError(format!(
                "Failed to process job tasks: {:?}",
                err
            ))
        },
    )?;

    Ok(())
}

fn filter_loctions<T, R, F>(
    input: Vec<T>,
    reference: Option<&R>,
    is_newer: F,
) -> Vec<T>
where
    F: Fn(&T, &R) -> bool,
{
    input
        .into_iter()
        .filter(|item| {
            reference.map(|ref_item| is_newer(item, ref_item)).unwrap_or(true)
        })
        .collect()
}

fn calculate_distance(
    last: &DriverLocation,
    current: &DriverLocationUpdate,
) -> f64 {
    let Latitude(last_lat) = last.location.lat;
    let Longitude(last_lon) = last.location.lon;
    let Latitude(current_lat) = current.lat_lng.lat;
    let Longitude(current_lon) = current.lat_lng.lon;
    let point_a = Point::new(last_lat, last_lon);
    let point_b = Point::new(current_lat, current_lon);
    Haversine::distance(point_a, point_b)
}

struct LocationFilterConfig {
    min_accuracy: AccuracyThreshold,
    min_distance: f64,
}

/// Filters driver location updates based on accuracy and distance from the last recorded location.
fn filter_locations_based_on_accuracy(
    config: LocationFilterConfig,
    mut driver_new_locations: Vec<DriverLocationUpdate>,
    driver_last_recorded_location: Option<&DriverLocation>,
) -> Vec<DriverLocationUpdate> {
    driver_new_locations.dedup_by(|a, b| {
        a.lat_lng.lat == b.lat_lng.lat && a.lat_lng.lon == b.lat_lng.lon
    });
    driver_new_locations
        .into_iter()
        .filter(|location| {
            let accuracy = location.accuracy.unwrap_or(AccuracyThreshold(0.0));
            if accuracy.0 < 0.0 {
                return false;
            }
            let is_within_accuracy = accuracy.0 <= config.min_accuracy.0;
            let is_far_enough = driver_last_recorded_location
                .map(|last| {
                    let distance = calculate_distance(last, location);
                    #[cfg(debug_assertions)]
                    info!(
                        tag = "Locations Distance Between",
                        "Distance between is: {:?}m", distance
                    );
                    distance > config.min_distance
                })
                .unwrap_or(true);
            is_within_accuracy && is_far_enough
        })
        .collect()
}
