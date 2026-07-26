//! Bridges into existing Sitwego services — READ-ONLY.
//!
//! The adapter translates protocol to product; it never mutates dispatch state
//! (Inviolable Rule #3). Step 4 adds discovery: route + availability + fare
//! quote for an inbound `search`, reusing the exact code paths the rider app
//! uses (`RidesApiClient` → OSRM, `find_nearest_driver` → Redis/H3,
//! `GetFairEstimates` → Postgres pricing).

use std::sync::Arc;

use api::db::queries::get_fair_estimates::GetFairEstimates;
use api::dispatch::state_machine::find_nearest_driver;
use api::request::RidesApiClient;
use api::simd_json::parse_from_string;
use api::types::VehicleCategory;
use async_trait::async_trait;
use db_store::Database;
use redis_store::RedisConnectionPool;
use redis_store::r_types::{GeoPoint, Latitude, Longitude, Radius};

/// How far around the pickup we look for drivers, in metres. Mirrors the rider
/// app's discovery radius (`rides.rs`).
const DISCOVERY_RADIUS_M: f64 = 4000.0;

/// A pickup/drop pair extracted from the search intent (decimal degrees).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

/// One bookable option in the quote — becomes one item + fulfillment in the
/// `on_search` catalog.
#[derive(Debug, Clone)]
pub struct QuoteOption {
    /// Sitwego service-tier code, e.g. `SWIFT` — used as the item id,
    /// fulfillment id and item `descriptor.code` (the way nammayatri uses its
    /// service-tier codes). Step 5's `select` echoes this back.
    pub tier_code: String,
    /// Human name, e.g. `Swift`.
    pub tier_name: String,
    /// Beckn `vehicle.category` network code: CAB / AUTO_RICKSHAW / TWO_WHEELER.
    pub vehicle_category: &'static str,
    /// Estimated total fare in whole currency units (KES).
    pub fare: f64,
    /// Fare components (whole KES) for the Step 5 quote breakup, straight from
    /// `FareEstimate`. `time_fare` is the per-minute (duration) component.
    /// They can sum to less than `fare` when the category minimum fare kicks in.
    pub base_fare: i32,
    pub distance_fare: i32,
    pub time_fare: i32,
    pub waiting_fare: i32,
}

/// The discovery result for a pickup/drop pair.
#[derive(Debug, Clone, Default)]
pub struct Quote {
    pub options: Vec<QuoteOption>,
    pub distance_km: f64,
    pub duration_s: u64,
}

/// Produces a quote for a search intent. Trait so handlers/tests don't need
/// OSRM + Redis + Postgres running.
#[async_trait]
pub trait DiscoveryAdapter: Send + Sync {
    async fn quote(
        &self,
        pickup: LatLon,
        dropoff: LatLon,
    ) -> anyhow::Result<Quote>;
}

/// Map a Sitwego vehicle category to the Beckn `vehicle.category` code.
/// Codes per nammayatri's `castVariant`: bikes are TWO_WHEELER, autos are
/// AUTO_RICKSHAW, every cab tier is CAB.
pub fn beckn_vehicle_category(category: &VehicleCategory) -> &'static str {
    match category {
        VehicleCategory::Bike => "TWO_WHEELER",
        VehicleCategory::Auto => "AUTO_RICKSHAW",
        _ => "CAB",
    }
}

/// The real discovery pipeline, identical to the rider app's estimate flow.
pub struct SitwegoDiscovery {
    db: Arc<Database>,
    redis: Arc<RedisConnectionPool>,
    routes_api_url: String,
}

impl SitwegoDiscovery {
    pub fn new(
        db: Arc<Database>,
        redis: Arc<RedisConnectionPool>,
        routes_api_url: String,
    ) -> Self {
        Self {
            db,
            redis,
            routes_api_url,
        }
    }
}

#[async_trait]
impl DiscoveryAdapter for SitwegoDiscovery {
    async fn quote(
        &self,
        pickup: LatLon,
        dropoff: LatLon,
    ) -> anyhow::Result<Quote> {
        // 1. Route the trip (OSRM) for distance + duration.
        let client = RidesApiClient::new_with_retry(&self.routes_api_url, None);
        let raw = client
            .get_ride_path_and_distance(
                &[(pickup.lon, pickup.lat), (dropoff.lon, dropoff.lat)],
                "overview=full&steps=true&geometries=geojson",
            )
            .await
            .map_err(|e| anyhow::anyhow!("routing failed: {e:?}"))?;
        let (_line, distance_m, duration_s) = parse_from_string(&raw)
            .map_err(|e| anyhow::anyhow!("route parse failed: {e:?}"))?;
        let distance_km = utils::meters_to_km(distance_m);

        // 2. Which vehicle categories actually have drivers near the pickup
        //    (read-only Redis/H3 lookup through the dispatch module).
        let pickup_point = GeoPoint {
            lat: Latitude(pickup.lat),
            lon: Longitude(pickup.lon),
        };
        let mut categories: Vec<VehicleCategory> = find_nearest_driver(
            self.redis.clone(),
            pickup_point,
            &None,
            &Radius(DISCOVERY_RADIUS_M),
        )
        .await
        .map_err(|e| anyhow::anyhow!("driver lookup failed: {e:?}"))?
        .into_iter()
        .map(|driver| driver.vehicle_category)
        .collect();
        categories.sort_by_cached_key(|c| c.to_string());
        categories.dedup_by_key(|c| c.to_string());

        if categories.is_empty() {
            // No drivers around: an honest empty quote, not an error.
            return Ok(Quote {
                options: vec![],
                distance_km,
                duration_s,
            });
        }

        // 3. Price each available category with the app's own fare engine.
        let estimates = self
            .db
            .get_fair_estimate(
                &categories,
                distance_km as f32,
                0,
                false,
                utils::seconds_to_minutes(duration_s) as i32,
            )
            .await
            .map_err(|e| anyhow::anyhow!("fare estimate failed: {e:?}"))?;

        let category_code = |name: &str| {
            categories
                .iter()
                .find(|c| c.to_string() == name)
                .map(beckn_vehicle_category)
        };
        let options = estimates
            .into_iter()
            .filter_map(|est| {
                let vehicle_category = category_code(&est.category)?;
                // The duration component isn't a named field on FareEstimate;
                // it's what remains of the pre-discount total.
                let time_fare = est.total_before_discount
                    - est.base_fare
                    - est.distance_cost
                    - est.waiting_cost;
                Some(QuoteOption {
                    tier_code: est.category.to_uppercase(),
                    tier_name: est.category.clone(),
                    vehicle_category,
                    fare: est.final_fare as f64,
                    base_fare: est.base_fare,
                    distance_fare: est.distance_cost,
                    time_fare,
                    waiting_fare: est.waiting_cost,
                })
            })
            .collect();

        Ok(Quote {
            options,
            distance_km,
            duration_s,
        })
    }
}

/// Boot-time placeholder for dev environments without OSRM/Redis/Postgres:
/// `search` still ACKs, but the async pipeline logs the failure and no
/// `on_search` goes out.
pub struct UnconfiguredDiscovery;

#[async_trait]
impl DiscoveryAdapter for UnconfiguredDiscovery {
    async fn quote(
        &self,
        _pickup: LatLon,
        _dropoff: LatLon,
    ) -> anyhow::Result<Quote> {
        anyhow::bail!(
            "discovery not configured (DATABASE_URL / ROUTES_API_URL / redis missing)"
        )
    }
}

/// Resolve one of our catalog tier codes (e.g. `SWIFT`) back to the Sitwego
/// vehicle category it was minted from (`QuoteOption::tier_code` is the
/// category's display name upper-cased).
pub fn vehicle_category_from_tier(tier_code: &str) -> Option<VehicleCategory> {
    use sea_orm::Iterable;
    VehicleCategory::iter()
        .find(|category| category.to_string().to_uppercase() == tier_code)
}

/// A confirmed network booking, ready to enter dispatch (Step 6).
#[derive(Debug, Clone, PartialEq)]
pub struct RideDispatch {
    /// Adapter-minted ride id (ULID) — becomes the dispatch request id and
    /// the idempotency anchor across the process boundary.
    pub request_id: String,
    /// Real customer identity from the Beckn order, shown to the driver.
    pub customer_name: String,
    pub customer_phone: String,
    pub pickup: LatLon,
    pub dropoff: LatLon,
    /// The fare agreed at init (whole KES) — never re-priced at confirm.
    pub fare: i32,
    /// Tier code the customer booked (e.g. `SWIFT`).
    pub tier_code: String,
}

/// Outcome of cancelling a booking on the api side (Step 7). Mirrors the
/// internal endpoint's `BecknCancelResponse.outcome` codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelOutcome {
    /// Dispatch was still hunting for a driver and has been signalled.
    DispatchCancelled,
    /// An accepted (not yet started) booking was torn down.
    Cancelled,
    /// The booking was already cancelled or completed.
    AlreadyClosed,
    /// The ride is in progress — v1 policy refuses mid-ride network cancels.
    ActiveRide,
    /// Dispatch has no trace of the request (e.g. it never got in).
    NotFound,
}

/// Hands a confirmed booking to the dispatch core. Trait so handler tests can
/// record the handoff instead of needing the api service running.
#[async_trait]
pub trait DispatchAdapter: Send + Sync {
    async fn request_ride(&self, dispatch: RideDispatch) -> anyhow::Result<()>;

    /// Cancel a network booking (customer side) — dispatch in flight or an
    /// accepted, not-yet-started ride.
    async fn cancel_ride(
        &self,
        request_id: &str,
    ) -> anyhow::Result<CancelOutcome>;
}

/// Calls the api service's private internal plane
/// (`POST {base}/internal/beckn/ride-request`, `X-Internal-Token` auth).
/// Dispatch state (offer channels, driver responses) lives in the api
/// process, so this HTTP hop is the only way in — the dispatch core itself is
/// called, never modified (Rule #3).
pub struct HttpDispatchAdapter {
    client: reqwest::Client,
    base_url: String,
    token: String,
    /// Pre-provisioned network-rider profile id (DB FK anchor).
    network_rider_id: String,
}

impl HttpDispatchAdapter {
    pub fn new(
        base_url: String,
        token: String,
        network_rider_id: String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            token,
            network_rider_id,
        }
    }
}

#[async_trait]
impl DispatchAdapter for HttpDispatchAdapter {
    async fn request_ride(&self, dispatch: RideDispatch) -> anyhow::Result<()> {
        use api::api::beckn_internal::BecknRideRequest;
        use api::api::ride_request::{RequestRideData, RiderDataInfo};

        let vehicle_category = vehicle_category_from_tier(&dispatch.tier_code)
            .ok_or_else(|| {
                anyhow::anyhow!("unknown tier code `{}`", dispatch.tier_code)
            })?;

        let (first_name, last_name) =
            match dispatch.customer_name.split_once(' ') {
                Some((first, last)) => (first.to_string(), last.to_string()),
                None => (dispatch.customer_name.clone(), String::new()),
            };
        let ride_data = |point: LatLon| RequestRideData {
            geo_point: GeoPoint {
                lat: Latitude(point.lat),
                lon: Longitude(point.lon),
            },
            street: None,
            city: None,
            road: None,
            country: None,
            building: None,
            floor: None,
            door: None,
            area_code: None,
            ward: None,
            place_id: None,
            instructions: None,
            extras: None,
        };

        let body = BecknRideRequest {
            request_id: dispatch.request_id.clone(),
            rider: RiderDataInfo {
                id: self.network_rider_id.clone(),
                first_name,
                last_name,
                rating: None,
                total_rating_score: None,
                email: String::new(),
                phone_number: dispatch.customer_phone.clone(),
                mobile_country_code: None,
            },
            from: ride_data(dispatch.pickup),
            to: ride_data(dispatch.dropoff),
            fare: dispatch.fare,
            vehicle_category,
            radius_m: DISCOVERY_RADIUS_M,
        };

        let url = format!(
            "{}/internal/beckn/ride-request",
            self.base_url.trim_end_matches('/')
        );
        let resp = self
            .client
            .post(&url)
            .header("x-internal-token", &self.token)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("dispatch handoff failed: {e}"))?;
        if !resp.status().is_success() {
            anyhow::bail!(
                "dispatch handoff rejected: HTTP {}",
                resp.status().as_u16()
            );
        }
        Ok(())
    }

    async fn cancel_ride(
        &self,
        request_id: &str,
    ) -> anyhow::Result<CancelOutcome> {
        let url = format!(
            "{}/internal/beckn/ride-request/{}/cancel",
            self.base_url.trim_end_matches('/'),
            request_id
        );
        let resp = self
            .client
            .post(&url)
            .header("x-internal-token", &self.token)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("cancel call failed: {e}"))?;
        if !resp.status().is_success() {
            anyhow::bail!("cancel rejected: HTTP {}", resp.status().as_u16());
        }
        let body: api::api::beckn_internal::BecknCancelResponse = resp
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("cancel response unreadable: {e}"))?;
        Ok(match body.outcome.as_str() {
            "dispatch_cancelled" => CancelOutcome::DispatchCancelled,
            "cancelled" => CancelOutcome::Cancelled,
            "already_closed" => CancelOutcome::AlreadyClosed,
            "active_ride" => CancelOutcome::ActiveRide,
            _ => CancelOutcome::NotFound,
        })
    }
}

/// Boot-time placeholder when the internal plane isn't configured: `confirm`
/// still ACKs, but the async pipeline reports the failure to the BAP.
pub struct UnconfiguredDispatch;

#[async_trait]
impl DispatchAdapter for UnconfiguredDispatch {
    async fn request_ride(
        &self,
        _dispatch: RideDispatch,
    ) -> anyhow::Result<()> {
        anyhow::bail!(
            "dispatch not configured (SITWEGO_INTERNAL_API_URL / \
             BECKN_INTERNAL_TOKEN / BECKN_NETWORK_RIDER_ID missing)"
        )
    }

    async fn cancel_ride(
        &self,
        _request_id: &str,
    ) -> anyhow::Result<CancelOutcome> {
        anyhow::bail!(
            "dispatch not configured (SITWEGO_INTERNAL_API_URL / \
             BECKN_INTERNAL_TOKEN / BECKN_NETWORK_RIDER_ID missing)"
        )
    }
}
