use crate::{
    schemas::{customer_rating, driver, driver_rating, driver_stats},
    types::{CustomerId, DriverId, RideId},
};
use db_store::Database;
use redis_store::r_types::AppError;
use sea_orm::prelude::Decimal;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DbBackend,
    EntityTrait, FromQueryResult, JoinType, QueryFilter, QuerySelect,
    RelationTrait, Statement, prelude::Expr, sea_query::SimpleExpr,
};
use serde::{Deserialize, Serialize};
use utils::{gen_strings::ulid_string, *};

pub trait DriverRating {
    fn rate_driver_for_ride(
        &self,
        create_rate_data: CreateRateData,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
    fn get_driver_recommendation_score(
        &self,
        driver_id: DriverId,
    ) -> impl std::future::Future<
        Output = Result<Option<DriverRatingResponse>, AppError>,
    > + Send;
}

#[derive(Debug, Clone)]
pub struct CreateRateData {
    pub ride_id: RideId,
    pub driver_id: DriverId,
    pub customer_id: CustomerId,
    pub feedback_details: Option<String>,
    pub was_offered_assistance: Option<bool>,
    pub attachment_id: Option<String>,
    pub rating_value: i32, // To be removed once granular sub-scores are fully supported on the frontend
    pub punctuality: Option<i32>,
    pub driving_behavior: Option<i32>,
    pub safety_compliance: Option<i32>,
    pub vehicle_cleanliness: Option<i32>,
}

/// Intermediate aggregate computed from all driver_rating rows for a driver.
#[derive(Debug, FromQueryResult)]
struct DriverCompositeStats {
    total_ratings: i64,
    avg_composite_score: f64, // AVG of per-row composites, clamped to [1.0, 5.0]
    positive_count: i64,
    assistance_count: i64,
}

// A struct to hold the query results
#[derive(Debug, Serialize, Deserialize, FromQueryResult)]
pub struct DriverRatingResponse {
    pub id: String,
    pub total_ratings: i64,
    pub total_rating: i64,
    pub avg_rating: Option<f64>, // Option to handle NULL for drivers with no ratings
    pub recommendation_score: f64,
}

const AVG_RATING_EXPR: &str = "AVG(driver_rating.rating_value)::FLOAT8";
// Bayesian score: CASE WHEN COUNT(driver_rating) > 0 THEN (SUM(driver_rating) + 3.0 * 5) / (COUNT(driver_rating) + 5) ELSE 3.0 END
// const RECOMMENDATION_SCORE_EXPR: &str = "CASE WHEN COUNT(driver_rating.rating_value) > 0 THEN (SUM(driver_rating.rating_value) + 2.5 * 5) / (COUNT(driver_rating.rating_value) + 5) ELSE 2.5 END::FLOAT8";
const TOTAL_RATING_EXPR: &str = "COALESCE(SUM(driver_rating.rating_value), 0)";

// Wilson Score Interval
const RECOMMENDATION_SCORE_EXPR: &str = "
    CASE
        WHEN COUNT(driver_rating.rating_value) > 0 THEN
            (
                (
                    (COUNT(CASE WHEN driver_rating.rating_value >= 4 THEN 1 END)::FLOAT8 / COUNT(driver_rating.rating_value)) +
                    (1.96 * 1.96 / (2.0 * COUNT(driver_rating.rating_value)))
                    - 1.96 * SQRT(
                        (
                            (COUNT(CASE WHEN driver_rating.rating_value >= 4 THEN 1 END)::FLOAT8 / COUNT(driver_rating.rating_value)) *
                            (1.0 - (COUNT(CASE WHEN driver_rating.rating_value >= 4 THEN 1 END)::FLOAT8 / COUNT(driver_rating.rating_value)))
                        ) / COUNT(driver_rating.rating_value) +
                        (1.96 * 1.96 / (4.0 * COUNT(driver_rating.rating_value) * COUNT(driver_rating.rating_value)))
                    )
                ) /
                (1.0 + (1.96 * 1.96 / COUNT(driver_rating.rating_value)))
            )::FLOAT8
        ELSE 0.0::FLOAT8
    END";

/// Computes the weighted composite score per rating row then aggregates across
/// all rows for a driver.  Optional sub-scores fall back to `rating_value` so
/// customers who skip granular fields don't skew the result.
///
/// Weights:  overall 40 % | punctuality 20 % | driving 20 % | safety 10 % | cleanliness 10 %
///
/// Each per-row composite is clamped to [1.0, 5.0] before averaging, so
/// out-of-range input can never push avg_composite_score outside that range.
const COMPOSITE_STATS_SQL: &str = "
    SELECT
        COUNT(*)::BIGINT AS total_ratings,
        COALESCE(AVG(
            LEAST(5.0, GREATEST(1.0,
                (rating_value::FLOAT8                       * 0.40) +
                (COALESCE(punctuality,        rating_value)::FLOAT8 * 0.20) +
                (COALESCE(driving_behavior,   rating_value)::FLOAT8 * 0.20) +
                (COALESCE(safety_compliance,  rating_value)::FLOAT8 * 0.10) +
                (COALESCE(vehicle_cleanliness,rating_value)::FLOAT8 * 0.10)
            ))
        ), 0.0) AS avg_composite_score,
        COUNT(CASE WHEN
            LEAST(5.0, GREATEST(1.0,
                (rating_value::FLOAT8                       * 0.40) +
                (COALESCE(punctuality,        rating_value)::FLOAT8 * 0.20) +
                (COALESCE(driving_behavior,   rating_value)::FLOAT8 * 0.20) +
                (COALESCE(safety_compliance,  rating_value)::FLOAT8 * 0.10) +
                (COALESCE(vehicle_cleanliness,rating_value)::FLOAT8 * 0.10)
            )) >= 4.0 THEN 1 END)::BIGINT AS positive_count,
        COUNT(CASE WHEN was_offered_assistance = true THEN 1 END)::BIGINT AS assistance_count
    FROM driver_rating
    WHERE driver_id = $1
";

/// Wilson Score lower bound (95 % confidence interval).
/// Returns a value in [0, 1].  Already handles cold-start: low sample sizes
/// produce conservatively low scores.
fn wilson_score(positive: f64, total: f64) -> f64 {
    if total == 0.0 {
        return 0.0;
    }
    let z = 1.96_f64;
    let z2 = z * z;
    let p_hat = positive / total;
    let numerator = p_hat + z2 / (2.0 * total)
        - z * (p_hat * (1.0 - p_hat) / total + z2 / (4.0 * total * total))
            .sqrt();
    let denominator = 1.0 + z2 / total;
    (numerator / denominator).max(0.0)
}

impl DriverRating for Database {
    async fn rate_driver_for_ride(
        &self,
        create_rate_data: CreateRateData,
    ) -> Result<(), AppError> {
        let res = self
            .transaction(move |tx| {
                let driver_id = create_rate_data.driver_id.0.clone();
                let ride_id = create_rate_data.ride_id.0.clone();
                let customer_id = create_rate_data.customer_id.0.clone();
                let text_feedback = create_rate_data.feedback_details.clone();
                let attachment_id = create_rate_data.attachment_id.clone();
                async move {
                    // Step 1: insert the new rating row with all sub-scores
                    driver_rating::ActiveModel {
                        id: ActiveValue::Set(ulid_string()),
                        ride_id: ActiveValue::Set(ride_id),
                        driver_id: ActiveValue::Set(driver_id.clone()),
                        customer_id: ActiveValue::Set(customer_id),
                        rating_value: ActiveValue::Set(
                            create_rate_data.rating_value,
                        ),
                        punctuality: ActiveValue::Set(
                            create_rate_data.punctuality,
                        ),
                        driving_behavior: ActiveValue::Set(
                            create_rate_data.driving_behavior,
                        ),
                        safety_compliance: ActiveValue::Set(
                            create_rate_data.safety_compliance,
                        ),
                        vehicle_cleanliness: ActiveValue::Set(
                            create_rate_data.vehicle_cleanliness,
                        ),
                        was_offered_assistance: ActiveValue::Set(
                            create_rate_data.was_offered_assistance,
                        ),
                        feedback_details: ActiveValue::Set(text_feedback),
                        attachment_id: ActiveValue::Set(attachment_id),
                        ..Default::default()
                    }
                    .insert(&*tx)
                    .await?;

                    // Step 2: recompute composite aggregate for this driver
                    // (includes the row just inserted)
                    let maybe_stats = DriverCompositeStats::find_by_statement(
                        Statement::from_sql_and_values(
                            DbBackend::Postgres,
                            COMPOSITE_STATS_SQL,
                            [driver_id.clone().into()],
                        ),
                    )
                    .one(&*tx)
                    .await?;

                    if let Some(stats) = maybe_stats {
                        // Step 3: pull cancellation data from driver_stats
                        let (rides_cancelled, total_rides_assigned) =
                            driver_stats::Entity::find_by_id(driver_id.clone())
                                .one(&*tx)
                                .await?
                                .map(|ds| {
                                    (
                                        ds.rides_cancelled.unwrap_or(0),
                                        ds.total_rides_assigned.unwrap_or(0),
                                    )
                                })
                                .unwrap_or((0, 0));

                        // Step 4: compute the final recommendation score
                        let total = stats.total_ratings as f64;
                        let positive = stats.positive_count as f64;
                        let assistance = stats.assistance_count as f64;

                        let wilson = wilson_score(positive, total);

                        // Penalty: high cancellation rate (capped at 25 %)
                        let cancellation_penalty = if total_rides_assigned > 0 {
                            (rides_cancelled as f64
                                / total_rides_assigned as f64)
                                .min(0.25)
                        } else {
                            0.0
                        };

                        // Bonus: drivers who consistently offer assistance
                        // (up to +5 % on the final score)
                        let assistance_bonus = if total > 0.0 {
                            (assistance / total) * 0.05
                        } else {
                            0.0
                        };

                        let final_score = (wilson
                            * (1.0 - cancellation_penalty)
                            + assistance_bonus)
                            .clamp(0.0, 1.0);

                        // SQL already clamps each row to [1.0, 5.0] before
                        // averaging; the Rust clamp is a defensive last resort.
                        let avg_composite =
                            stats.avg_composite_score.clamp(1.0, 5.0);

                        // A rating is considered "valid" once there are
                        // at least 5 data points
                        let is_valid = total >= 5.0;

                        // Step 5: persist back to driver_stats in the same
                        // transaction — ride-matching queries read this column
                        // directly without re-aggregating
                        driver_stats::Entity::update_many()
                            .col_expr(
                                driver_stats::Column::Rating,
                                Expr::val(
                                    Decimal::try_from(final_score)
                                        .unwrap_or_default(),
                                )
                                .into(),
                            )
                            .col_expr(
                                driver_stats::Column::TotalRatings,
                                Expr::val(stats.total_ratings as i32).into(),
                            )
                            .col_expr(
                                driver_stats::Column::TotalRatingScore,
                                Expr::val(avg_composite).into(),
                            )
                            .col_expr(
                                driver_stats::Column::IsValidRating,
                                Expr::val(is_valid).into(),
                            )
                            .filter(
                                driver_stats::Column::DriverId.eq(driver_id),
                            )
                            .exec(&*tx)
                            .await?;
                    }

                    Ok(())
                }
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;

        Ok(res)
    }

    async fn get_driver_recommendation_score(
        &self,
        driver_id: DriverId,
    ) -> Result<Option<DriverRatingResponse>, AppError> {
        let result = self
            .transaction(move |tx| {
                let driver_id = driver_id.0.clone();
                //Define the query to get the driver recommendation scores
                let query = driver::Entity::find()
                    .join(
                        JoinType::LeftJoin,
                        driver::Relation::DriverRating.def(),
                    )
                    .select_only()
                    .column(driver::Column::Id)
                    .filter(driver::Column::Id.eq(driver_id))
                    .column_as(
                        Expr::col((
                            driver_rating::Entity,
                            driver_rating::Column::RatingValue,
                        ))
                        .count(),
                        "total_ratings",
                    )
                    .column_as(
                        SimpleExpr::Custom(AVG_RATING_EXPR.to_string()),
                        "avg_rating",
                    )
                    .column_as(
                        SimpleExpr::Custom(TOTAL_RATING_EXPR.to_string()),
                        "total_rating",
                    )
                    .column_as(
                        SimpleExpr::Custom(
                            RECOMMENDATION_SCORE_EXPR.to_string(),
                        ),
                        "recommendation_score",
                    )
                    .group_by(driver::Column::Id);
                Box::pin(async move {
                    let result = query
                        .into_model::<DriverRatingResponse>()
                        .one(&*tx)
                        .await?;
                    Ok(result)
                })
            })
            .await
            .map_err(|err| AppError::DatabaseError(err.to_string()))?;
        Ok(result)
    }
}

// ── Driver rating summary ─────────────────────────────────────────────────────

/// Composite score expression reused in both breakdown and recent-reviews SQL.
/// Weights: overall 40% | punctuality 20% | driving_behavior 20% | safety 10% | cleanliness 10%
/// Sub-scores fall back to rating_value when NULL so sparse rows are never penalised.
/// Result is clamped to [1, 5] then rounded to the nearest integer star bucket.
const COMPOSITE_STAR_EXPR: &str = "
    ROUND(
        LEAST(5.0, GREATEST(1.0,
            (rating_value::FLOAT8                            * 0.40) +
            (COALESCE(punctuality,        rating_value)::FLOAT8 * 0.20) +
            (COALESCE(driving_behavior,   rating_value)::FLOAT8 * 0.20) +
            (COALESCE(safety_compliance,  rating_value)::FLOAT8 * 0.10) +
            (COALESCE(vehicle_cleanliness,rating_value)::FLOAT8 * 0.10)
        ))
    )::INT";

/// Groups by the composite star bucket (1–5) rather than the raw rating_value,
/// so the breakdown reflects all four sub-scores when they are present.
const BREAKDOWN_SQL: &str = "
    WITH composite AS (
        SELECT
            ROUND(
                LEAST(5.0, GREATEST(1.0,
                    (rating_value::FLOAT8                            * 0.40) +
                    (COALESCE(punctuality,        rating_value)::FLOAT8 * 0.20) +
                    (COALESCE(driving_behavior,   rating_value)::FLOAT8 * 0.20) +
                    (COALESCE(safety_compliance,  rating_value)::FLOAT8 * 0.10) +
                    (COALESCE(vehicle_cleanliness,rating_value)::FLOAT8 * 0.10)
                ))
            )::INT AS rating_value
        FROM driver_rating
        WHERE driver_id = $1
    )
    SELECT rating_value, COUNT(*)::BIGINT AS cnt
    FROM composite
    GROUP BY rating_value
";

/// Recent reviews show the composite star instead of the raw rating_value.
const RECENT_REVIEWS_SQL: &str = "
    SELECT
        dr.id,
        ROUND(
            LEAST(5.0, GREATEST(1.0,
                (dr.rating_value::FLOAT8                               * 0.40) +
                (COALESCE(dr.punctuality,        dr.rating_value)::FLOAT8 * 0.20) +
                (COALESCE(dr.driving_behavior,   dr.rating_value)::FLOAT8 * 0.20) +
                (COALESCE(dr.safety_compliance,  dr.rating_value)::FLOAT8 * 0.10) +
                (COALESCE(dr.vehicle_cleanliness,dr.rating_value)::FLOAT8 * 0.10)
            ))
        )::INT AS rating_value,
        COALESCE(dr.feedback_details, '') AS feedback_details,
        TO_CHAR(dr.created_at AT TIME ZONE 'UTC', 'Mon FMDD, YYYY') AS review_date,
        p.first_name,
        p.last_name
    FROM driver_rating dr
    JOIN profile p ON p.id = dr.customer_id
    WHERE dr.driver_id = $1
    ORDER BY dr.created_at DESC
    LIMIT 10
";

#[derive(Debug, FromQueryResult)]
struct StarCountRow {
    rating_value: i32,
    cnt: i64,
}

#[derive(Debug, FromQueryResult)]
struct RecentReviewRow {
    id: String,
    rating_value: i32,
    feedback_details: String,
    review_date: String,
    first_name: String,
    last_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingSummaryData {
    pub average: f32,
    pub total_reviews: i64,
    pub recommend_percent: i32,
    pub breakdown: Vec<StarBreakdown>,
    pub recent_reviews: Vec<RecentReviewData>,
}

#[derive(Debug, Serialize)]
pub struct StarBreakdown {
    pub stars: i32,
    pub percent: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentReviewData {
    pub id: String,
    pub rider_name: String,
    pub rating: i32,
    pub comment: String,
    pub date: String,
}

fn mask_rider_name(first_name: &str, last_name: &str) -> String {
    match last_name.chars().next() {
        Some(initial) => format!("{} {}.", first_name, initial),
        None => first_name.to_owned(),
    }
}

pub trait DriverRatingSummary {
    fn get_driver_rating_summary(
        &self,
        driver_id: &str,
    ) -> impl std::future::Future<
        Output = Result<Option<RatingSummaryData>, AppError>,
    > + Send;
}

impl DriverRatingSummary for Database {
    async fn get_driver_rating_summary(
        &self,
        driver_id: &str,
    ) -> Result<Option<RatingSummaryData>, AppError> {
        let driver_exists = driver::Entity::find_by_id(driver_id)
            .one(self.conn())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .is_some();

        if !driver_exists {
            return Ok(None);
        }

        let stats = driver_stats::Entity::find_by_id(driver_id)
            .one(self.conn())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let average = stats
            .as_ref()
            .and_then(|s| s.total_rating_score)
            .map(|v| ((v * 10.0).round() / 10.0) as f32)
            .unwrap_or(0.0);

        let star_rows =
            StarCountRow::find_by_statement(Statement::from_sql_and_values(
                DbBackend::Postgres,
                BREAKDOWN_SQL,
                [driver_id.into()],
            ))
            .all(self.conn())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let total_reviews: i64 = star_rows.iter().map(|r| r.cnt).sum();

        let mut counts = [0i64; 6]; // index 1..=5
        for r in &star_rows {
            if (1..=5).contains(&r.rating_value) {
                counts[r.rating_value as usize] = r.cnt;
            }
        }

        let breakdown: Vec<StarBreakdown> = (1..=5_i32)
            .rev()
            .map(|stars| {
                let cnt = counts[stars as usize];
                let percent = if total_reviews > 0 {
                    ((cnt as f64 / total_reviews as f64) * 100.0).round() as i32
                } else {
                    0
                };
                StarBreakdown { stars, percent }
            })
            .collect();

        let recommend_percent = if total_reviews > 0 {
            let positive = counts[4] + counts[5];
            ((positive as f64 / total_reviews as f64) * 100.0).round() as i32
        } else {
            0
        };

        let review_rows =
            RecentReviewRow::find_by_statement(Statement::from_sql_and_values(
                DbBackend::Postgres,
                RECENT_REVIEWS_SQL,
                [driver_id.into()],
            ))
            .all(self.conn())
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let recent_reviews = review_rows
            .into_iter()
            .map(|row| RecentReviewData {
                id: row.id,
                rider_name: mask_rider_name(&row.first_name, &row.last_name),
                rating: row.rating_value,
                comment: row.feedback_details,
                date: row.review_date,
            })
            .collect();

        Ok(Some(RatingSummaryData {
            average,
            total_reviews,
            recommend_percent,
            breakdown,
            recent_reviews,
        }))
    }
}

// ── Rider rating ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CreateRiderRateData {
    pub ride_id: RideId,
    pub driver_id: DriverId,
    pub customer_id: CustomerId,
    pub punctuality: Option<i32>,
    pub respectfulness: Option<i32>,
    pub fare_readiness: Option<i32>,
    pub feedback_details: Option<String>,
    pub attachment_id: Option<String>,
}

pub trait RiderRating {
    fn rate_rider_for_ride(
        &self,
        data: CreateRiderRateData,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

/// Aggregates computed from all customer_rating rows for a given customer.
///
/// Composite = average of whichever sub-scores (punctuality, respectfulness,
/// fare_readiness) are non-null for each row.  Null sub-scores are excluded
/// from both the sum and the denominator so they never dilute the result.
/// The per-row composite is clamped to [1.0, 5.0] before averaging.
#[derive(Debug, FromQueryResult)]
struct RiderAggregateStats {
    total_ratings: i64,
    avg_composite_score: f64,
    positive_count: i64,
}

const RIDER_AGGREGATE_SQL: &str = "
    SELECT
        COUNT(*)::BIGINT AS total_ratings,
        COALESCE(AVG(
            LEAST(5.0, GREATEST(1.0,
                (
                    COALESCE(punctuality::FLOAT8,    0) +
                    COALESCE(respectfulness::FLOAT8, 0) +
                    COALESCE(fare_readiness::FLOAT8, 0)
                ) / GREATEST(
                    (CASE WHEN punctuality    IS NOT NULL THEN 1 ELSE 0 END +
                     CASE WHEN respectfulness IS NOT NULL THEN 1 ELSE 0 END +
                     CASE WHEN fare_readiness IS NOT NULL THEN 1 ELSE 0 END)::FLOAT8,
                    1
                )
            ))
        ), 0.0) AS avg_composite_score,
        COUNT(CASE WHEN
            LEAST(5.0, GREATEST(1.0,
                (
                    COALESCE(punctuality::FLOAT8,    0) +
                    COALESCE(respectfulness::FLOAT8, 0) +
                    COALESCE(fare_readiness::FLOAT8, 0)
                ) / GREATEST(
                    (CASE WHEN punctuality    IS NOT NULL THEN 1 ELSE 0 END +
                     CASE WHEN respectfulness IS NOT NULL THEN 1 ELSE 0 END +
                     CASE WHEN fare_readiness IS NOT NULL THEN 1 ELSE 0 END)::FLOAT8,
                    1
                )
            )) >= 4.0 THEN 1 END)::BIGINT AS positive_count
    FROM customer_rating
    WHERE customer_id = $1
";

/// SQL upsert: inserts the first time a rider is rated; updates thereafter.
/// All non-rating columns keep their DB defaults on first insert.
const RIDER_STATS_UPSERT_SQL: &str = "
    INSERT INTO rider_stats (customer_id, rating, total_ratings, total_rating_score, is_valid_rating, updated_at)
    VALUES ($1, $2, $3, $4, $5, NOW())
    ON CONFLICT (customer_id) DO UPDATE SET
        rating             = EXCLUDED.rating,
        total_ratings      = EXCLUDED.total_ratings,
        total_rating_score = EXCLUDED.total_rating_score,
        is_valid_rating    = EXCLUDED.is_valid_rating,
        updated_at         = NOW()
";

impl RiderRating for Database {
    async fn rate_rider_for_ride(
        &self,
        data: CreateRiderRateData,
    ) -> Result<(), AppError> {
        self.transaction(move |tx| {
            let customer_id = data.customer_id.0.clone();
            let driver_id = data.driver_id.0.clone();
            let ride_id = data.ride_id.0.clone();
            let text_feedback = data.feedback_details.clone();
            let attachment_id = data.attachment_id.clone();
            async move {
                // Derive rating_value from the average of provided sub-scores.
                // At least one sub-score must be present.
                let scores: Vec<i32> = [
                    data.punctuality,
                    data.respectfulness,
                    data.fare_readiness,
                ]
                .into_iter()
                .flatten()
                .collect();

                // if scores.is_empty() {
                //     return Err(sea_orm::DbErr::Custom(
                //         "At least one sub-score (punctuality, respectfulness, \
                //          fare_readiness) must be provided"
                //             .to_string()
                //     ));
                // }

                let rating_value = (scores.iter().sum::<i32>() as f64
                    / scores.len() as f64)
                    .round() as i32;

                // Step 1: insert the driver-to-rider rating row
                customer_rating::ActiveModel {
                    id: ActiveValue::Set(ulid_string()),
                    ride_id: ActiveValue::Set(ride_id),
                    customer_id: ActiveValue::Set(customer_id.clone()),
                    driver_id: ActiveValue::Set(driver_id),
                    rating_value: ActiveValue::Set(rating_value),
                    punctuality: ActiveValue::Set(data.punctuality),
                    respectfulness: ActiveValue::Set(data.respectfulness),
                    fare_readiness: ActiveValue::Set(data.fare_readiness),
                    feedback_details: ActiveValue::Set(text_feedback),
                    attachment_id: ActiveValue::Set(attachment_id),
                    ..Default::default()
                }
                .insert(&*tx)
                .await?;

                // Step 2: recompute aggregates (includes the just-inserted row)
                let maybe_stats = RiderAggregateStats::find_by_statement(
                    Statement::from_sql_and_values(
                        DbBackend::Postgres,
                        RIDER_AGGREGATE_SQL,
                        [customer_id.clone().into()],
                    ),
                )
                .one(&*tx)
                .await?;

                if let Some(stats) = maybe_stats {
                    let total = stats.total_ratings as f64;
                    let positive = stats.positive_count as f64;

                    // Wilson score lower bound — same formula used for drivers
                    let rating_score = wilson_score(positive, total);
                    // SQL already clamps each row to [1.0, 5.0]; Rust clamp is defensive
                    let avg_composite =
                        stats.avg_composite_score.clamp(1.0, 5.0);
                    let is_valid = total >= 5.0;

                    // Step 3: upsert rider_stats with the new aggregates
                    (&*tx)
                        .execute(Statement::from_sql_and_values(
                            DbBackend::Postgres,
                            RIDER_STATS_UPSERT_SQL,
                            [
                                customer_id.into(),
                                Decimal::try_from(rating_score)
                                    .unwrap_or_default()
                                    .into(),
                                (stats.total_ratings as i32).into(),
                                avg_composite.into(),
                                is_valid.into(),
                            ],
                        ))
                        .await?;
                }

                Ok(())
            }
        })
        .await
        .map_err(|err| AppError::DatabaseError(err.to_string()))
    }
}
