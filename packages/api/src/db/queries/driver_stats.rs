use chrono::Utc;
use db_store::Database;
use sea_orm::{ActiveValue, entity::prelude::*};

use crate::{
    schemas::driver_stats::{self},
    types::DriverId,
};

pub trait DriverStatsQueries {
    fn create_driver_stats_tx(
        &self,
        driver_id: String,
    ) -> impl Future<Output = utils::Result<()>> + Send;

    fn update_driver_stats_total_rides(
        &self,
        driver_id: String,
        n: &i32,
    ) -> impl Future<Output = utils::Result<()>> + Send;
    fn update_driver_stats_total_rides_assigned(
        &self,
        driver_id: String,
        n: &i32,
    ) -> impl Future<Output = utils::Result<()>> + Send;
    fn update_driver_stats_rides_cancelled(
        &self,
        driver_id: String,
        n: &i32,
    ) -> impl Future<Output = utils::Result<()>> + Send;

    fn update_driver_rating(
        &self,
        driver_id: String,
        rating: RatingStatData,
    ) -> impl Future<Output = utils::Result<()>> + Send;

    fn update_driver_idle_since(
        &self,
        driver_id: String,
        idle_since: Option<chrono::DateTime<Utc>>,
    ) -> impl Future<Output = utils::Result<()>> + Send;

    fn update_driver_total_earnings(
        &self,
        driver_id: &DriverId,
        n: &i32,
    ) -> impl Future<Output = utils::Result<()>> + Send;
    // fn get_driver_stats_by_ids(
    //     &self,
    //     driver_ids: Vec<String>,
    // ) -> impl Future<Output = utils::Result<Vec<driver_stats::Model>>> + Send;
}

pub struct RatingStatData {
    pub rating: Decimal,
    pub total_ratings: i32,
    pub total_rating_score: f64,
    pub is_valid_rating: bool,
}
impl DriverStatsQueries for Database {
    async fn create_driver_stats_tx(
        &self,
        driver_id: String,
    ) -> utils::Result<()> {
        let _ = self
            .transaction(move |tx| {
                let driver_id = driver_id.clone();
                async move {
                    let _ = driver_stats::Entity::insert(
                        driver_stats::ActiveModel {
                            driver_id: ActiveValue::set(driver_id),
                            ..Default::default()
                        },
                    )
                    .exec(&*tx)
                    .await
                    .expect(
                        "Failed to insert driver stats into driver_stats table",
                    );
                    Ok(())
                }
            })
            .await?;
        Ok(())
    }

    async fn update_driver_idle_since(
        &self,
        driver_id: String,
        idle_since: Option<chrono::DateTime<Utc>>,
    ) -> utils::Result<()> {
        self.transaction(move |tx| {
            let driver_id = driver_id.clone();
            Box::pin(async move {
                let mut query = driver_stats::Entity::update_many()
                    .col_expr(
                        driver_stats::Column::UpdatedAt,
                        Expr::val(Utc::now()).into(),
                    )
                    .filter(driver_stats::Column::DriverId.eq(driver_id));
                if let Some(idle_since) = idle_since {
                    query = query.col_expr(
                        driver_stats::Column::IdleSince,
                        Expr::val(idle_since).into(),
                    );
                }
                query.exec(&*tx).await?;
                Ok(())
            })
        })
        .await
    }

    async fn update_driver_stats_total_rides(
        &self,
        driver_id: String,
        n: &i32,
    ) -> utils::Result<()> {
        update_stats_helper(
            &self,
            vec![(driver_stats::Column::TotalRides, *n)],
            &driver_id,
        )
        .await
    }

    async fn update_driver_stats_total_rides_assigned(
        &self,
        driver_id: String,
        n: &i32,
    ) -> utils::Result<()> {
        update_stats_helper(
            &self,
            vec![(driver_stats::Column::TotalRidesAssigned, *n)],
            &driver_id,
        )
        .await
    }

    async fn update_driver_stats_rides_cancelled(
        &self,
        driver_id: String,
        n: &i32,
    ) -> utils::Result<()> {
        update_stats_helper(
            &self,
            vec![(driver_stats::Column::RidesCancelled, *n)],
            &driver_id,
        )
        .await
    }

    async fn update_driver_total_earnings(
        &self,
        driver_id: &DriverId,
        n: &i32,
    ) -> utils::Result<()> {
        update_stats_helper(
            &self,
            vec![(driver_stats::Column::TotalEarnings, *n)],
            &driver_id.0,
        )
        .await
    }

    async fn update_driver_rating(
        &self,
        driver_id: String,
        rating_data: RatingStatData,
    ) -> utils::Result<()> {
        self.transaction(move |tx| {
            let driver_id = driver_id.clone();
            Box::pin(async move {
                let mut query = driver_stats::Entity::update_many()
                    .col_expr(
                        driver_stats::Column::UpdatedAt,
                        Expr::val(Utc::now()).into(),
                    )
                    .filter(driver_stats::Column::DriverId.eq(driver_id));
                if rating_data.is_valid_rating {
                    query = query
                        .col_expr(
                            driver_stats::Column::TotalRatingScore,
                            Expr::val(rating_data.total_rating_score).into(),
                        )
                        .col_expr(
                            driver_stats::Column::TotalRatings,
                            Expr::val(rating_data.total_ratings).into(),
                        )
                        .col_expr(
                            driver_stats::Column::Rating,
                            Expr::val(rating_data.rating).into(),
                        )
                        .col_expr(
                            driver_stats::Column::IsValidRating,
                            Expr::val(rating_data.is_valid_rating).into(),
                        );
                }
                query.exec(&*tx).await?;
                Ok(())
            })
        })
        .await?;
        Ok(())
    }
}

async fn update_stats_helper(
    db: &Database,
    cols_inc: Vec<(impl ColumnTrait, i32)>,
    driver_id: &str,
) -> utils::Result<()> {
    db.transaction(move |tx| {
        let driver_id = driver_id.to_string();
        let cols_inc = cols_inc.clone();
        Box::pin(async move {
            let mut query = driver_stats::Entity::update_many()
                .col_expr(
                    driver_stats::Column::UpdatedAt,
                    Expr::val(Utc::now()).into(),
                )
                .filter(driver_stats::Column::DriverId.eq(driver_id));

            for (column, increment_by) in cols_inc {
                query =
                    query.col_expr(column, Expr::col(column).add(increment_by));
            }

            query.exec(&*tx).await?;
            Ok(())
        })
    })
    .await?;
    Ok(())
}
