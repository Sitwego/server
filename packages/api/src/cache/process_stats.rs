use std::sync::Arc;

use db_store::Database;
use pathfinding::num_traits::{FromPrimitive, Zero};
use redis_store::r_types::AppError;
use sea_orm::prelude::Decimal;
use tokio::sync::mpsc::Receiver;
use utils::{Result, executor::Executor};

use crate::queries::driver_stats::{DriverStatsQueries, RatingStatData};
use crate::{queries::rating::DriverRating, types::DriverId};
// use crate::schemas::driver_stats::*;

pub struct ProcessStats {
    pub executor: Executor,
    pub db: Arc<Database>,
    pub sts_rx: Receiver<ProcessStataData>,
}

pub type ProcessStataData = DriverId;
impl ProcessStats {
    pub fn new(
        executor: Executor,
        db: Arc<Database>,
        sts_rx: Receiver<ProcessStataData>,
    ) -> Self {
        Self {
            executor,
            db,
            sts_rx,
        }
    }
    pub async fn run(self) {
        let executor = self.executor.clone();
        executor.spawn_detached_task(async move {
            Self::process_stats(self).await;
        });
    }

    pub async fn process_stats(mut p: ProcessStats) {
        let mut timer =
            tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            tokio::select! {
              incoming_stats = p.sts_rx.recv() => {
                match incoming_stats {
                    Some(driver_id) => {
                        let _ = p.update_driver_stats_with_retry(driver_id)
                          .await;
                    }
                    None => {

                    }
                }
              },
              _ = timer.tick() => {

              }
            }
        }
    }

    async fn get_and_process_driver_stats(
        &self,
        driver_id: DriverId,
    ) -> Result<(), AppError> {
        let stats = self
            .db
            .get_driver_recommendation_score(driver_id.to_owned())
            .await
            .map_err(|err| AppError::InternalError(err.to_string()));
        // Process the stats here
        match stats {
            Ok(stats) => match stats {
                Some(stats) => {
                    // Process the stats and update the database
                    let rating = RatingStatData {
                        rating: Decimal::from_f64(
                            stats.avg_rating.unwrap_or(0.0),
                        )
                        .unwrap_or(Decimal::zero()),
                        total_ratings: stats.total_rating as i32,
                        total_rating_score: stats.recommendation_score,
                        is_valid_rating: true,
                    };

                    let _ =
                        self.db.update_driver_rating(driver_id.0, rating).await;
                }
                None => {
                    return Err(AppError::InternalError(
                        "No stats found".to_string(),
                    ));
                }
            },
            Err(err) => {
                return Err(err);
            }
        }

        Ok(())
    }

    async fn update_driver_stats_with_retry(
        &self,
        driver_id: DriverId,
    ) -> Result<(), AppError> {
        let mut attempts = 0;
        let max_attempts = 3;
        loop {
            attempts += 1;
            match self.get_and_process_driver_stats(driver_id.to_owned()).await
            {
                Ok(()) => return Ok(()),
                Err(e) if attempts < max_attempts => {
                    tracing::error!(
                        "Error processing driver stats: {}. Retrying {}/{}",
                        e,
                        attempts,
                        max_attempts
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(1))
                        .await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
