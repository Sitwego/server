use std::sync::Arc;

use dashmap::DashMap;
use redis_store::r_types::{GeoPoint, Radius};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::time::{Duration, Instant};
use tracing::{info_span, warn, warn_span};

use crate::APIContext;
use crate::dispatch::dispatch::DriverPoolManager;
use crate::dispatch::state_machine::{DispatchEvent, DispatchStateMachine};
use crate::types::VehicleCategory;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestRideData {
    pub geo_point: GeoPoint,
    pub street: Option<String>,
    pub city: Option<String>,
    pub road: Option<String>,
    pub country: Option<String>,
    pub building: Option<String>,
    pub floor: Option<String>,
    pub door: Option<String>,
    pub area_code: Option<String>,
    pub ward: Option<String>,
    pub place_id: Option<String>,
    pub instructions: Option<String>,
    pub extras: Option<sea_orm::entity::prelude::Json>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDriver {
    pub from: RequestRideData,
    pub to: RequestRideData,
    pub fare: i32,
    pub dx: f64,
    pub duration: i32,
    pub vehicle_type: Option<Vec<VehicleCategory>>,
    pub radius: Radius,
    pub rider_profile: RiderDataInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiderDataInfo {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub rating: Option<f64>,
    pub total_rating_score: Option<f64>,
    pub email: String,
    pub phone_number: String,
    pub mobile_country_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RideResponse {
    pub request_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct DriverResponsePayload {
    pub request_id: String,
    pub accepted: bool,
}

#[derive(Debug, Serialize)]
pub struct DriverResponseResult {
    pub success: bool,
    pub message: String,
}

pub struct RideRequestState {
    pub tx: broadcast::Sender<DispatchEvent>,
    pub metadata: RequestMetadata,
}

#[derive(Debug, Clone)]
pub struct RequestMetadata {
    pub rider_id: String,
    pub created_at: Instant,
}

pub struct DispatchApiManager {
    pub requests: DashMap<String, RideRequestState>,
}

impl Default for DispatchApiManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatchApiManager {
    pub fn new() -> Self {
        Self {
            requests: DashMap::new(),
        }
    }

    pub fn register_ride_request(
        &self,
        ride_req_id: String,
        rider_id: String,
    ) -> Result<broadcast::Receiver<DispatchEvent>, String> {
        let entry = self.requests.entry(ride_req_id.clone());
        match entry {
            dashmap::mapref::entry::Entry::Occupied(_) => {
                warn_span!("duplicate_ride_request", ride_req_id = %ride_req_id)
                    .in_scope(|| {
                        tracing::warn!(
                            "Duplicate ride request detected for request ID: {}",
                            ride_req_id
                        );
                    });
                Err(format!(
                    "Ride request {} is already being dispatched",
                    ride_req_id
                ))
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let (tx, rx) = broadcast::channel(128);
                entry.insert(RideRequestState {
                    tx,
                    metadata: RequestMetadata {
                        rider_id,
                        created_at: Instant::now(),
                    },
                });
                info_span!("register_ride_request", ride_req_id = %ride_req_id)
                    .in_scope(|| {
                        tracing::info!("Registered new ride request");
                    });
                Ok(rx)
            }
        }
    }

    /// Send driver response to the appropriate dispatcher
    pub fn send_driver_response(
        &self,
        ride_req_id: &str,
        response: DispatchEvent,
    ) -> Result<(), String> {
        match self.requests.get(ride_req_id) {
            Some(state) => {
                state.tx.send(response).map(|_| ()).map_err(|e| {
                    format!("Failed to send driver response: {}", e)
                })
            }
            None => Err(format!(
                "!!!No channel found for ride request ID: {}",
                ride_req_id
            )),
        }
    }

    /// Cleanup completed request
    pub fn cleanup_request(&self, ride_req_id: &str) {
        self.requests.remove(ride_req_id);
        info_span!("cleanup_request", ride_req_id = %ride_req_id).in_scope(
            || {
                tracing::info!("🧹Cleaned up ride request");
            },
        );
    }

    /// Periodic cleanup of old requests
    pub fn cleanup_old_requests(&self, max_age: Duration) {
        let now = Instant::now();
        self.requests.retain(|req_id, state| {
            let keep = now.duration_since(state.metadata.created_at) < max_age;
            if !keep {
                info_span!("cleanup_old_requests", ride_req_id = %req_id)
                    .in_scope(|| {
                        tracing::info!("🧹 Completed cleanup of old requests");
                    });
            }
            keep
        });
    }
}

pub struct DispatcherQueue {
    pub pending: tokio::sync::mpsc::Sender<DispatchJob>,
    pub semaphore: Arc<tokio::sync::Semaphore>,
}

pub struct DispatchJob {
    pub request_id: String,
    pub rider_id: String,
    pub request: RequestDriver,
    pub ride_search_result: crate::dispatch::state_machine::RideSearchResult,
    pub response_rx: broadcast::Receiver<DispatchEvent>,
}

impl DispatcherQueue {
    pub fn new(
        max_concurrent_dispatches: usize,
        queue_size: usize,
    ) -> (Self, tokio::sync::mpsc::Receiver<DispatchJob>) {
        let (tx, rx) = tokio::sync::mpsc::channel(queue_size);
        let semaphore =
            Arc::new(tokio::sync::Semaphore::new(max_concurrent_dispatches));
        (
            Self {
                pending: tx,
                semaphore,
            },
            rx,
        )
    }

    pub async fn enqueue_dispatch(
        &self,
        job: DispatchJob,
    ) -> Result<(), String> {
        warn!(
            tag = "enqueue_dispatch",
            request_id = %job.request_id,
            rider_id = %job.rider_id,
            "Enqueuing dispatch job for ride request"
        );
        self.pending
            .send(job)
            .await
            .map_err(|e| format!("Failed to enqueue dispatch job: {}", e))
    }

    pub async fn start_workers(
        &self,
        mut rx: tokio::sync::mpsc::Receiver<DispatchJob>,
        driver_pool_manager: Arc<DriverPoolManager>,
        dispatch_api_manager: Arc<DispatchApiManager>,
        semaphore: Arc<tokio::sync::Semaphore>,
        ctx: Arc<APIContext>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                let mgr = driver_pool_manager.clone();
                let dispatch_api_manager = dispatch_api_manager.clone();
                let sem = semaphore.clone();
                let request_id = job.request_id.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let permit =
                        sem.acquire_owned().await.expect("Semaphore closed");
                    let pick_up_location = job.request.from.geo_point;
                    let vc = job.request.vehicle_type.clone();
                    let dispatch_sm: DispatchStateMachine =
                        DispatchStateMachine::new(
                            mgr,
                            request_id.clone(),
                            20_000,
                            job.response_rx,
                            20,
                            300,
                            pick_up_location,
                            vc,
                            ctx,
                            job.request,
                            job.rider_id,
                            job.ride_search_result,
                        )
                        .await;

                    tokio::time::sleep(Duration::from_millis(1000)).await;

                    let result = dispatch_sm.start().await;
                    tracing::info!(
                        tag = "dispatch_worker",
                        request_id = %request_id,
                        result = ?result,
                        "Dispatch state machine finished ✅, cleaning up channel 🧹🧹"
                    );
                    dispatch_api_manager.cleanup_request(&request_id);
                    drop(permit);
                });
            }
            warn_span!("dispatcher_queue_worker_shutdown").in_scope(|| {
                tracing::warn!("Dispatcher queue worker shutting down");
            });
        })
    }
}
