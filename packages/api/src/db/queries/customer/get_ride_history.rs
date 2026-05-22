use chrono::Datelike;
use db_store::Database;
use redis_store::r_types::AppError;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{DbBackend, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct RideHistoryGroup {
    /// Human-readable month label, e.g. "January 2026".
    pub label: String,
    /// Sum of fares for **completed** rides in this month.
    /// `None` when no completed rides exist in the month.
    pub total_spent: Option<Decimal>,
    pub rides: Vec<RideHistorySummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RideHistorySummary {
    pub ride_id: String,
    pub driver_id: String,
    /// Best available drop-off place name (ward → city → street → area_code).
    pub destination_name: String,
    pub status: String,
    /// `None` for rides with no recorded fare (e.g. pure cancellations).
    pub amount: Option<Decimal>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Whether the rider has submitted a rating for the driver on this ride.
    pub has_rated_driver: bool,
}

// ── Trait ────────────────────────────────────────────────────────────────────

pub trait GetRiderRideHistory {
    fn get_rider_ride_history(
        &self,
        rider_id: &str,
        page: u32,
        page_size: u32,
    ) -> impl std::future::Future<
        Output = Result<Vec<RideHistoryGroup>, AppError>,
    > + Send;
}

// ── Implementation ────────────────────────────────────────────────────────────

/// Internal row returned by the raw SQL query.
#[derive(Debug, FromQueryResult)]
struct RideRow {
    ride_id: String,
    driver_id: String,
    status: String,
    amount: Option<Decimal>,
    started_at: DateTimeWithTimeZone,
    destination_name: String,
    has_rated_driver: bool,
}

impl GetRiderRideHistory for Database {
    async fn get_rider_ride_history(
        &self,
        rider_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<RideHistoryGroup>, AppError> {
        let offset = (page * page_size) as i64;
        let limit = page_size as i64;
        let rider_id = rider_id.to_string();

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
            SELECT
                r.id                                                 AS ride_id,
                r.driver_id,
                r.status,
                r.fare                                               AS amount,
                r.trip_start_time                                    AS started_at,
                COALESCE(l.ward, l.city, l.street, l.area_code, '') AS destination_name,
                (dr.ride_id IS NOT NULL)                             AS has_rated_driver
            FROM ride r
            INNER JOIN ride_requests rr ON rr.id  = r.id
            LEFT  JOIN location      l  ON l.id   = rr.to_location_id
            LEFT  JOIN driver_rating dr ON dr.ride_id = r.id
                                       AND dr.customer_id = r.customer_id
            WHERE r.customer_id = $1
            ORDER BY r.trip_start_time DESC
            LIMIT $2 OFFSET $3
            "#,
            [rider_id.into(), limit.into(), offset.into()],
        );

        let rows = RideRow::find_by_statement(stmt)
            .all(self.conn())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(group_by_month(rows))
    }
}

// ── Grouping logic ────────────────────────────────────────────────────────────

fn group_by_month(rows: Vec<RideRow>) -> Vec<RideHistoryGroup> {
    // BTreeMap key (year, month) keeps groups in natural order;
    // we reverse at the end so the newest month comes first.
    // Within each group the ride order is preserved from the DESC-sorted query.
    let mut map: BTreeMap<(i32, u32), Vec<RideRow>> = BTreeMap::new();

    for row in rows {
        let key = (row.started_at.year(), row.started_at.month());
        map.entry(key).or_default().push(row);
    }

    map.into_iter()
        .rev()
        .map(|((year, month), group_rows)| {
            let label = format!("{} {}", month_name(month), year);

            let total_spent = {
                let has_completed = group_rows
                    .iter()
                    .any(|r| r.status.eq_ignore_ascii_case("Completed"));

                if has_completed {
                    let sum: Decimal = group_rows
                        .iter()
                        .filter(|r| r.status.eq_ignore_ascii_case("Completed"))
                        .filter_map(|r| r.amount)
                        .sum();
                    Some(sum)
                } else {
                    None
                }
            };

            let rides = group_rows
                .into_iter()
                .map(|r| RideHistorySummary {
                    ride_id: r.ride_id,
                    driver_id: r.driver_id,
                    destination_name: r.destination_name,
                    status: r.status,
                    amount: r.amount,
                    started_at: r.started_at.with_timezone(&chrono::Utc),
                    has_rated_driver: r.has_rated_driver,
                })
                .collect();

            RideHistoryGroup {
                label,
                total_spent,
                rides,
            }
        })
        .collect()
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, TimeZone};
    use rust_decimal::prelude::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn utc_dt(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
    ) -> DateTimeWithTimeZone {
        FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(year, month, day, hour, min, 0)
            .unwrap()
    }

    fn row(
        ride_id: &str,
        status: &str,
        amount: Option<Decimal>,
        started_at: DateTimeWithTimeZone,
        destination: &str,
    ) -> RideRow {
        RideRow {
            ride_id: ride_id.to_string(),
            driver_id: "driver-1".to_string(),
            status: status.to_string(),
            amount,
            started_at,
            destination_name: destination.to_string(),
            has_rated_driver: false,
        }
    }

    #[test]
    fn empty_input_returns_empty_groups() {
        assert!(group_by_month(vec![]).is_empty());
    }

    #[test]
    fn all_cancelled_month_has_no_total() {
        let rows = vec![
            row(
                "r1",
                "Canceled",
                None,
                utc_dt(2026, 1, 27, 10, 46),
                "Kew Palace",
            ),
            row(
                "r2",
                "Canceled",
                None,
                utc_dt(2026, 1, 27, 10, 37),
                "Kew Palace",
            ),
        ];

        let groups = group_by_month(rows);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].label, "January 2026");
        assert_eq!(groups[0].total_spent, None);
        assert_eq!(groups[0].rides.len(), 2);
    }

    #[test]
    fn completed_rides_are_summed_in_total() {
        let rows = vec![
            row(
                "r1",
                "Completed",
                Some(d("12.50")),
                utc_dt(2025, 11, 18, 9, 58),
                "Rainham Station",
            ),
            row(
                "r2",
                "Completed",
                Some(d("7.00")),
                utc_dt(2025, 11, 10, 14, 0),
                "Victoria",
            ),
            row(
                "r3",
                "Canceled",
                None,
                utc_dt(2025, 11, 5, 8, 0),
                "Heathrow",
            ),
        ];

        let groups = group_by_month(rows);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].total_spent, Some(d("19.50")));
    }

    #[test]
    fn cancelled_fare_excluded_from_total() {
        let rows = vec![
            row(
                "r1",
                "Completed",
                Some(d("10.00")),
                utc_dt(2025, 10, 5, 9, 0),
                "A",
            ),
            row(
                "r2",
                "Canceled",
                Some(d("5.00")),
                utc_dt(2025, 10, 3, 9, 0),
                "B",
            ),
        ];

        let groups = group_by_month(rows);
        assert_eq!(groups[0].total_spent, Some(d("10.00")));
    }

    #[test]
    fn multiple_months_sorted_newest_first() {
        let rows = vec![
            row("r1", "Canceled", None, utc_dt(2026, 1, 27, 10, 0), "Dest A"),
            row(
                "r2",
                "Completed",
                Some(d("0.00")),
                utc_dt(2025, 11, 18, 9, 58),
                "Rainham Station",
            ),
            row(
                "r3",
                "Canceled",
                None,
                utc_dt(2025, 10, 24, 9, 24),
                "TW9 3AG",
            ),
        ];

        let groups = group_by_month(rows);

        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0].label, "January 2026");
        assert_eq!(groups[1].label, "November 2025");
        assert_eq!(groups[2].label, "October 2025");

        assert_eq!(groups[0].total_spent, None);
        assert_eq!(groups[1].total_spent, Some(d("0.00")));
        assert_eq!(groups[2].total_spent, None);
    }

    #[test]
    fn rides_within_month_preserve_desc_order() {
        let rows = vec![
            row("r1", "Canceled", None, utc_dt(2026, 1, 27, 10, 46), "A"),
            row("r2", "Canceled", None, utc_dt(2026, 1, 27, 10, 37), "B"),
            row("r3", "Canceled", None, utc_dt(2026, 1, 27, 9, 5), "C"),
        ];

        let groups = group_by_month(rows);
        let ids: Vec<&str> =
            groups[0].rides.iter().map(|r| r.ride_id.as_str()).collect();
        assert_eq!(ids, ["r1", "r2", "r3"]);
    }

    #[test]
    fn destination_name_is_passed_through() {
        let rows = vec![row(
            "r1",
            "Canceled",
            None,
            utc_dt(2025, 10, 1, 8, 0),
            "TW9 3AG",
        )];

        let groups = group_by_month(rows);
        assert_eq!(groups[0].rides[0].destination_name, "TW9 3AG");
    }
}
