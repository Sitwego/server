use db_store::Database;
use redis_store::r_types::AppError;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
    pub city: Option<String>,
    pub ward: Option<String>,
    pub state: Option<String>,
    pub place_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RideDetail {
    pub ride_id: String,
    pub driver_id: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub ride_category: Option<String>,
    pub destination_name: String,
    pub total_fare: Option<Decimal>,
    pub status: String,
    pub from: Option<LatLng>,
    pub to: Option<LatLng>,
    pub driver: DriverDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverDetail {
    pub name: String,
    pub rating: Option<f64>,
    pub photo_id: Option<String>,
    pub vehicle: VehicleDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VehicleDetail {
    pub make: Option<String>,
    pub model: Option<String>,
    pub color: String,
    pub plate_number: String,
}

// ── Trait ─────────────────────────────────────────────────────────────────────

pub trait GetRideDetail {
    fn get_ride_detail(
        &self,
        ride_id: &str,
        customer_id: &str,
    ) -> impl std::future::Future<Output = Result<Option<RideDetail>, AppError>> + Send;
}

// ── Internal row ──────────────────────────────────────────────────────────────

#[derive(Debug, FromQueryResult)]
struct RideDetailRow {
    ride_id: String,
    driver_id: String,
    status: String,
    total_fare: Option<Decimal>,
    started_at: DateTimeWithTimeZone,
    ride_category: Option<String>,
    destination_name: String,
    // from/to coordinates
    from_lat: Option<f64>,
    from_lon: Option<f64>,
    from_city: Option<String>,
    from_ward: Option<String>,
    from_state: Option<String>,
    from_place_id: Option<String>,
    to_lat: Option<f64>,
    to_lon: Option<f64>,
    to_city: Option<String>,
    to_ward: Option<String>,
    to_state: Option<String>,
    to_place_id: Option<String>,
    // driver profile
    first_name: Option<String>,
    last_name: Option<String>,
    photo_id: Option<String>,
    // driver stats
    rating: Option<f64>,
    // vehicle
    make: Option<String>,
    model: Option<String>,
    color: Option<String>,
    plate_number: Option<String>,
}

// ── Implementation ─────────────────────────────────────────────────────────────

impl GetRideDetail for Database {
    async fn get_ride_detail(
        &self,
        ride_id: &str,
        customer_id: &str,
    ) -> Result<Option<RideDetail>, AppError> {
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            SELECT
                r.id                                                 AS ride_id,
                r.driver_id,
                r.status,
                r.fare                                               AS total_fare,
                r.trip_start_time                                    AS started_at,
                r.trip_category                                      AS ride_category,
                COALESCE(tl.ward, tl.city, tl.street, tl.area_code, '') AS destination_name,
                fl.lat                                               AS from_lat,
                fl.lon                                               AS from_lon,
                fl.city                                              AS from_city,
                fl.ward                                              AS from_ward,
                fl.state                                             AS from_state,
                fl.place_id                                          AS from_place_id,
                tl.lat                                               AS to_lat,
                tl.lon                                               AS to_lon,
                tl.city                                              AS to_city,
                tl.ward                                              AS to_ward,
                tl.state                                             AS to_state,
                tl.place_id                                          AS to_place_id,
                p.first_name,
                p.last_name,
                d.photo_id,
                ds.total_rating_score                                 AS rating,
                v.make,
                v.model,
                v.color,
                v.plate_number
            FROM ride r
            INNER JOIN ride_requests rr ON rr.id        = r.id
            LEFT  JOIN location      fl ON fl.id        = rr.from_location_id
            LEFT  JOIN location      tl ON tl.id        = rr.to_location_id
            LEFT  JOIN profile       p  ON p.id         = r.driver_id
            LEFT  JOIN driver        d  ON d.id         = r.driver_id
            LEFT  JOIN driver_stats  ds ON ds.driver_id = r.driver_id
            LEFT  JOIN vehicle       v  ON v.driver_id  = r.driver_id
            WHERE r.id = $1
              AND r.customer_id = $2
            "#,
            [ride_id.into(), customer_id.into()],
        );

        let row = RideDetailRow::find_by_statement(stmt)
            .one(self.conn())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| {
            let last_initial = r
                .last_name
                .as_deref()
                .and_then(|s| s.chars().next())
                .map(|c| format!(" {}.", c))
                .unwrap_or_default();
            let driver_name = format!(
                "{}{}",
                r.first_name.as_deref().unwrap_or("Unknown"),
                last_initial
            );

            RideDetail {
                ride_id: r.ride_id,
                driver_id: r.driver_id,
                date: r.started_at.with_timezone(&chrono::Utc),
                ride_category: r.ride_category,
                destination_name: r.destination_name,
                total_fare: r.total_fare,
                status: r.status,
                from: match (r.from_lat, r.from_lon) {
                    (Some(lat), Some(lng)) => Some(LatLng {
                        lat,
                        lng,
                        city: r.from_city,
                        ward: r.from_ward,
                        state: r.from_state,
                        place_id: r.from_place_id,
                    }),
                    _ => None,
                },
                to: match (r.to_lat, r.to_lon) {
                    (Some(lat), Some(lng)) => Some(LatLng {
                        lat,
                        lng,
                        city: r.to_city,
                        ward: r.to_ward,
                        state: r.to_state,
                        place_id: r.to_place_id,
                    }),
                    _ => None,
                },
                driver: DriverDetail {
                    name: driver_name,
                    rating: r.rating,
                    photo_id: r.photo_id,
                    vehicle: VehicleDetail {
                        make: r.make,
                        model: r.model,
                        color: r.color.unwrap_or_default(),
                        plate_number: r.plate_number.unwrap_or_default(),
                    },
                },
            }
        }))
    }
}
