use chrono::{Datelike, Duration, NaiveDate, Weekday};
use db_store::Database;
use redis_store::r_types::AppError;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, FromQueryResult,
    IntoActiveModel, JoinType, QueryFilter, QuerySelect, RelationTrait,
    Statement,
};
use serde::Serialize;
use std::collections::HashMap;
use tracing::{error, info};
use utils::Result;

use crate::schemas::driver_earning;

#[derive(Debug, FromQueryResult, Serialize, PartialEq, Eq)]
pub struct DailyEarningsSummary {
    pub total_earnings: Option<Decimal>,
    pub total_discount: Option<Decimal>,
    pub total_rides: Option<i64>,
}

#[derive(Debug, FromQueryResult, Serialize, PartialEq)]
pub struct RideHistoryItem {
    pub ride_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub created_at: sea_orm::prelude::DateTimeWithTimeZone,
    pub estimated_distance: Option<f64>,
    pub estimated_duration: Option<i32>,
    pub from_area_code: Option<String>,
    pub from_ward: Option<String>,
    pub from_city: Option<String>,
    pub to_area_code: Option<String>,
    pub to_ward: Option<String>,
    pub to_city: Option<String>,
    pub has_rated_customer: bool,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct DailyEarningsReport {
    pub summary: DailyEarningsSummary,
    pub rides: Vec<RideHistoryItem>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct BarChartEntry {
    pub value: Decimal,
    pub label: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct WeeklyEarningsReport {
    pub total_rides: i64,
    pub total_earnings: Decimal,
    pub chart_data: Vec<BarChartEntry>,
}

pub trait DriverEarnings {
    fn insert_driver_earning_for_ride(
        &self,
        dr_earnings_model: &driver_earning::Model,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    fn get_daily_earnings_for_driver(
        &self,
        driver_id: &str,
        date: &str,
    ) -> impl std::future::Future<Output = Result<DailyEarningsReport, AppError>>
    + Send;

    fn get_weekly_earnings_for_driver(
        &self,
        driver_id: &str,
        date: &str,
    ) -> impl std::future::Future<
        Output = Result<WeeklyEarningsReport, AppError>,
    > + Send;
}

impl DriverEarnings for Database {
    async fn insert_driver_earning_for_ride(
        &self,
        dr_earnings_model: &driver_earning::Model,
    ) -> Result<(), AppError> {
        let _ = self
            .transaction(move |tx| {
                let driver_earning_model = dr_earnings_model.clone();
                async move {
                    driver_earning_model
                        .into_active_model()
                        .insert(&*tx)
                        .await?;
                    Ok(())
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(())
    }

    async fn get_daily_earnings_for_driver(
        &self,
        driver_id: &str,
        date: &str,
    ) -> Result<DailyEarningsReport, AppError> {
        // Parse the date string (expected format: YYYY-MM-DD)
        let parsed_date =
            NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|err| {
                AppError::ValidationError(format!(
                    "Invalid date format: {}",
                    err
                ))
            })?;

        // Create date range for the entire day in Nairobi timezone
        let start_of_day =
            parsed_date.and_hms_opt(0, 0, 0).ok_or_else(|| {
                AppError::ValidationError("Invalid date".to_string())
            })?;
        // .and_local_timezone(Nairobi)
        // .single()
        // .ok_or_else(|| {
        //     AppError::ValidationError("Invalid date".to_string())
        // })?
        // .to_utc();

        let end_of_day = parsed_date
            .succ_opt()
            .ok_or_else(|| {
                AppError::ValidationError("Invalid date".to_string())
            })?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| {
                AppError::ValidationError("Invalid date".to_string())
            })?;
        // .and_local_timezone(Nairobi)
        // .single()
        // .ok_or_else(|| {
        //     AppError::ValidationError("Invalid date".to_string())
        // })?
        // .to_utc();

        info!(
            "Fetching daily earnings for driver_id: {}, date: {}, start_of_day: {}, end_of_day: {}",
            driver_id, date, start_of_day, end_of_day
        );

        let report = self
            .transaction(move |tx| {
                let driver_id = driver_id.to_string();
                async move {
                    // Aggregate summary: total earnings, discount, and ride count
                    let summary = driver_earning::Entity::find()
                        .join(
                            JoinType::InnerJoin,
                            driver_earning::Relation::Ride.def(),
                        )
                        .filter(driver_earning::Column::DriverId.eq(&driver_id))
                        .filter(
                            driver_earning::Column::CreatedAt.gte(start_of_day),
                        )
                        .filter(
                            driver_earning::Column::CreatedAt.lt(end_of_day),
                        )
                        .select_only()
                        .column_as(
                            driver_earning::Column::Amount.sum(),
                            "total_earnings",
                        )
                        .column_as(
                            driver_earning::Column::Discount.sum(),
                            "total_discount",
                        )
                        .column_as(
                            driver_earning::Column::Id.count(),
                            "total_rides",
                        )
                        .into_model::<DailyEarningsSummary>()
                        .one(&*tx)
                        .await
                        .map_err(|err: sea_orm::DbErr| {
                            error!("Database error: {:?}", err);
                            AppError::DatabaseError(err.to_string())
                        })?
                        .unwrap_or(DailyEarningsSummary {
                            total_earnings: Some(Decimal::ZERO),
                            total_discount: Some(Decimal::ZERO),
                            total_rides: Some(0),
                        });

                    // Individual rides with location info for the history list
                    let rides = RideHistoryItem::find_by_statement(
                        Statement::from_sql_and_values(
                            DbBackend::Postgres,
                            r#"
                            SELECT
                                de.ride_id,
                                de.amount,
                                de.currency,
                                de.created_at,
                                rr.estimated_distance,
                                rr.estimated_duration,
                                fl.area_code AS from_area_code,
                                fl.ward      AS from_ward,
                                fl.city      AS from_city,
                                tl.area_code AS to_area_code,
                                tl.ward      AS to_ward,
                                tl.city      AS to_city,
                                (cr.id IS NOT NULL) AS has_rated_customer
                            FROM driver_earning de
                            INNER JOIN ride_requests rr ON rr.id = de.ride_id
                            LEFT  JOIN location fl         ON fl.id = rr.from_location_id
                            LEFT  JOIN location tl         ON tl.id = rr.to_location_id
                            LEFT  JOIN customer_rating cr  ON cr.ride_id = de.ride_id
                                                         AND cr.driver_id = de.driver_id
                            WHERE de.driver_id  = $1
                              AND de.created_at >= $2
                              AND de.created_at  < $3
                            ORDER BY de.created_at DESC
                            "#,
                            [
                                driver_id.clone().into(),
                                start_of_day.into(),
                                end_of_day.into(),
                            ],
                        ),
                    )
                    .all(&*tx)
                    .await
                    .map_err(|err| {
                        AppError::DatabaseError(err.to_string())
                    })?;

                    info!("Rides count: {}", rides.len());

                    Ok(DailyEarningsReport { summary, rides })
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        Ok(report)
    }

    async fn get_weekly_earnings_for_driver(
        &self,
        driver_id: &str,
        date: &str,
    ) -> Result<WeeklyEarningsReport, AppError> {
        let today =
            NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|err| {
                AppError::ValidationError(format!(
                    "Invalid date format: {}",
                    err
                ))
            })?;

        // Anchor to Monday of the week containing `today`.
        // Days after `today` have no records so they resolve to 0 naturally.
        let days_from_monday = today.weekday().num_days_from_monday() as i64;
        let week_start = today - Duration::days(days_from_monday); // Monday
        let week_end = week_start + Duration::days(6); // Sunday

        let start_dt = week_start.and_hms_opt(0, 0, 0).ok_or_else(|| {
            AppError::ValidationError("Invalid date".to_string())
        })?;

        // Query up to end-of-Sunday (exclusive = Monday 00:00 next week)
        let end_dt = week_end
            .succ_opt()
            .ok_or_else(|| {
                AppError::ValidationError("Invalid date".to_string())
            })?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| {
                AppError::ValidationError("Invalid date".to_string())
            })?;

        info!(
            "Fetching weekly earnings for driver_id: {}, week: {} to {} (today: {})",
            driver_id, week_start, week_end, today
        );

        let earnings = self
            .transaction(move |tx| {
                let driver_id = driver_id.to_string();
                async move {
                    let rows = driver_earning::Entity::find()
                        .filter(driver_earning::Column::DriverId.eq(&driver_id))
                        .filter(driver_earning::Column::CreatedAt.gte(start_dt))
                        .filter(driver_earning::Column::CreatedAt.lt(end_dt))
                        .all(&*tx)
                        .await?; // DbErr -> utils::Error via From impl
                    Ok(rows)
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        info!("Earnings count for week: {}", earnings.len());

        let mut daily_totals: HashMap<NaiveDate, Decimal> = HashMap::new();
        let mut total_earnings = Decimal::ZERO;
        let total_rides = earnings.len() as i64;

        for earning in &earnings {
            let day = earning.created_at.date_naive();
            *daily_totals.entry(day).or_insert(Decimal::ZERO) += earning.amount;
            total_earnings += earning.amount;
        }

        let chart_data = (0..7)
            .map(|i| {
                let day = week_start + Duration::days(i);
                let value =
                    daily_totals.get(&day).copied().unwrap_or(Decimal::ZERO);
                let label = match day.weekday() {
                    Weekday::Mon => "M",
                    Weekday::Tue => "T",
                    Weekday::Wed => "W",
                    Weekday::Thu => "T",
                    Weekday::Fri => "F",
                    Weekday::Sat => "S",
                    Weekday::Sun => "S",
                }
                .to_string();
                BarChartEntry { value, label }
            })
            .collect();

        Ok(WeeklyEarningsReport {
            total_rides,
            total_earnings,
            chart_data,
        })
    }
}
