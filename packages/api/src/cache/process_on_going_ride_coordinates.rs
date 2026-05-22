use std::sync::Arc;

use db_store::Database;
use redis_store::r_types::GeoPoint;
use tokio::sync::mpsc::Receiver;
use utils::{Result, executor::Executor};

use crate::{
    queries::ride::RideQueries,
    types::{DriverId, RideId},
};

pub struct ProcessOnGoingRideCoordinates {
    pub executor: Executor,
    pub batch_size: usize,
}

pub type OnGoingRideCoordinatesData = (RideId, DriverId, Vec<GeoPoint>);
impl ProcessOnGoingRideCoordinates {
    pub fn new(executor: Executor, batch_size: usize) -> Self {
        Self {
            executor,
            batch_size,
        }
    }
    pub async fn run(
        &self,
        r_rx: Receiver<OnGoingRideCoordinatesData>,
        db: Arc<Database>,
    ) -> Result<()> {
        let futures = Self::process_on_going_ride_coordinates(r_rx, db);
        self.executor.spawn_detached_task(futures);
        Ok(())
    }

    pub async fn process_on_going_ride_coordinates(
        mut r_rx: Receiver<OnGoingRideCoordinatesData>,
        db: Arc<Database>,
    ) {
        let mut timer =
            tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            tokio::select! {
              incoming_ride_data = r_rx.recv() => {
                match incoming_ride_data {
                    Some((ride_id, driver_id, ride_coordinates)) => {
                        // Process the ride coordinates
                        // For example, save them to the database or perform some calculations
                        if ride_coordinates.len() > 2 {
                            // println!("Processing ride coordinates: {:?}", ride_coordinates);
                            // Here you can call the database function to process the coordinates
                            let _ = db.process_ride_coordinates(&driver_id, &ride_id, &ride_coordinates)
                                .await;
                        }
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
}
