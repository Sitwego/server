use db_store::Database;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;
use tracing::info_span;
use utils::Result;

use crate::{schemas::vehicle_categories, types::VehicleCategory};

/// Minimum approach distance (km) before a pickup fare applies. Short
/// approaches are absorbed (free) — the driver is close enough that charging
/// for the leg isn't warranted.
pub const PICKUP_FARE_MIN_DISTANCE_KM: f64 = 1.0;

/// Prices the pickup (approach) leg from the matched driver's estimated
/// distance/duration to the rider:
///
///   pickup_fare = (distance_km × distance_rate) + (duration_min × time_rate)
///
/// Distance arrives in metres and duration in seconds (OSRM units persisted by
/// dispatch on the ride request); rates are the category's per-km short-tier
/// rate and per-minute rate, matching how the ride fare itself is priced.
/// Approaches under [`PICKUP_FARE_MIN_DISTANCE_KM`] are free. Returns whole
/// KES, floored at 0.
pub fn compute_pickup_fare(
    pricing: &vehicle_categories::Model,
    distance_m: Option<f64>,
    duration_s: Option<i32>,
) -> i32 {
    let distance_km = distance_m.unwrap_or(0.0) / 1000.0;
    if distance_km < PICKUP_FARE_MIN_DISTANCE_KM {
        return 0;
    }
    let duration_min = duration_s.unwrap_or(0) as f64 / 60.0;
    let fare = distance_km * pricing.short_distance_kes_per_km as f64
        + duration_min * pricing.per_min_rate as f64;
    fare.round().max(0.0) as i32
}

pub trait GetFairEstimates {
    fn get_fair_estimate(
        &self,
        vhcle_ctgry: &[VehicleCategory],
        distance_km: f32,
        waiting_minutes: i32,
        is_return_trip: bool,
        duration: i32,
    ) -> impl std::future::Future<Output = Result<Vec<FareEstimate>>> + Send;

    /// Fetches the pricing row for a single vehicle category. Used to price
    /// the pickup leg (and any other server-side fare component) at ride start.
    fn get_category_pricing(
        &self,
        category: &VehicleCategory,
    ) -> impl std::future::Future<
        Output = Result<Option<vehicle_categories::Model>>,
    > + Send;

    /// Resolves the server-authoritative pickup fare for a ride: loads the
    /// category's pricing and prices the approach leg via [`compute_pickup_fare`].
    /// A pricing lookup miss/error must not block the caller, so it degrades to
    /// 0 (the pickup leg simply isn't charged).
    fn resolve_pickup_fare(
        &self,
        category: &VehicleCategory,
        distance_m: Option<f64>,
        duration_s: Option<i32>,
    ) -> impl std::future::Future<Output = i32> + Send;
}

impl GetFairEstimates for Database {
    async fn get_fair_estimate(
        &self,
        v_c: &[VehicleCategory],
        distance_km: f32,
        waiting_minutes: i32,
        is_return_trip: bool,
        duration: i32,
    ) -> Result<Vec<FareEstimate>> {
        let v_c = v_c.to_vec();
        self.transaction(move |tx| {
            let value: Vec<VehicleCategory> = v_c.clone();
            async move {
                let categories = value.to_vec();

                let pricing_val = vehicle_categories::Entity::find()
                    .filter(
                        vehicle_categories::Column::Category.is_in(categories),
                    )
                    .all(&*tx)
                    .await?;
                let fair_estimate = pricing_val
                    .iter()
                    .map(|m| {
                        calculate_fare_for_category(
                            m,
                            distance_km,
                            waiting_minutes,
                            is_return_trip,
                            duration,
                        )
                    })
                    .collect::<Vec<FareEstimate>>();

                // Log the fare estimates for debugging
                info_span!("fare_estimate", ?fair_estimate).in_scope(|| {
                    tracing::info!(
                        "Calculated fare estimates: {:?}",
                        fair_estimate
                    );
                });

                Ok(fair_estimate)
            }
        })
        .await
    }

    async fn get_category_pricing(
        &self,
        category: &VehicleCategory,
    ) -> Result<Option<vehicle_categories::Model>> {
        let pricing = vehicle_categories::Entity::find()
            .filter(vehicle_categories::Column::Category.eq(category.clone()))
            .one(self.conn())
            .await?;
        Ok(pricing)
    }

    async fn resolve_pickup_fare(
        &self,
        category: &VehicleCategory,
        distance_m: Option<f64>,
        duration_s: Option<i32>,
    ) -> i32 {
        match self.get_category_pricing(category).await {
            Ok(Some(pricing)) => {
                compute_pickup_fare(&pricing, distance_m, duration_s)
            }
            Ok(None) => {
                tracing::warn!(
                    tag = "pickup_fare",
                    "No pricing row for category {:?}; skipping pickup fare",
                    category
                );
                0
            }
            Err(err) => {
                tracing::error!(
                    tag = "pickup_fare",
                    "Failed to load category pricing: {:?}; skipping pickup fare",
                    err
                );
                0
            }
        }
    }
}

// Struct for fare calculation results
#[derive(Debug, Serialize)]
pub struct FareEstimate {
    category: String,
    base_fare: i32,
    distance_cost: i32,
    waiting_cost: i32,
    total_before_discount: i32,
    discount: f32,
    final_fare: f32,
}
fn calculate_fare_for_category(
    pricing: &vehicle_categories::Model,
    distance_km: f32,
    waiting_minutes: i32,
    is_return_trip: bool,
    duration: i32,
) -> FareEstimate {
    // Split distance into short and long
    let short_distance = distance_km.min(10.0);
    let long_distance = (distance_km - 10.0).max(0.0);

    let short_distance_cost =
        (short_distance * pricing.short_distance_kes_per_km as f32) as i32;
    let long_distance_cost =
        (long_distance * pricing.long_distance_kes_per_km as f32) as i32;

    // Total distance cost
    let distance_cost = short_distance_cost + long_distance_cost;

    // Waiting cost
    let waiting_cost = waiting_minutes * pricing.waiting_per_minute_kes;

    //Total per minute
    let total_per_min = duration * pricing.per_min_rate;

    // Total before discount
    let total_before_discount =
        pricing.base_fare_kes + distance_cost + waiting_cost + total_per_min;

    // Parse return trip discount safely
    let discount_percentage = pricing
        .return_trip_discount
        .trim_end_matches('%')
        .parse::<f32>()
        .unwrap_or(0.0)
        / 100.0;

    // Apply discount if it's a return trip
    let discount = if is_return_trip {
        total_before_discount as f32 * discount_percentage
    } else {
        0.0
    };

    // Final fare after discount
    let mut final_fare = total_before_discount as f32 - discount;

    // Apply minimum fare
    if final_fare < pricing.min_fare as f32 {
        final_fare = pricing.min_fare as f32;
    }

    //TODO:: we can apply surges if applicable

    FareEstimate {
        category: pricing.category.to_string(),
        base_fare: pricing.base_fare_kes,
        distance_cost,
        waiting_cost,
        total_before_discount,
        discount,
        final_fare,
    }
}
