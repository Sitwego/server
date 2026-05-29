pub mod api;
pub mod api_responses;
pub mod auth_middleware;
pub mod auth_token;
pub mod cache;
pub mod config;
pub mod db;
pub mod dispatch;
pub mod helper;
pub mod jobs;
pub mod middleware;
pub mod migrations;
pub mod notif;
pub mod request;
pub mod simd_json;
pub mod tracking;
pub mod types;
use cache::process_on_going_ride_coordinates::OnGoingRideCoordinatesData;
use cache::process_stats::ProcessStataData;
pub use db::*;

use notif_api::GorushClient;
use redis_store::RedisConfig;
use sms_api::TwilioVerifyClient;

use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use utils::Result;
use utils::executor::Executor;

use crate::types::*;
use config::config::Config;
use db_store::ConnectOptions;
use db_store::Database;
use redis_store::RedisConnectionPool;

pub struct APIContext {
    pub db: Arc<Database>,
    pub config: Config,
    pub redis: Arc<RedisConnectionPool>,
    pub tx: Sender<DriverLocationEvent>,
    pub r_tx: Sender<OnGoingRideCoordinatesData>,
    pub stats_tx: Sender<ProcessStataData>,
    pub notif: Arc<GorushClient>,
    pub verify: Arc<TwilioVerifyClient>,

    pub driver_pool_manager: Arc<dispatch::dispatch::DriverPoolManager>,
    pub dispatch_api_manager: Arc<api::ride_request::DispatchApiManager>,
    pub dispatcher_queue: Arc<api::ride_request::DispatcherQueue>,
}

impl APIContext {
    pub async fn new(
        config: Config,
        tx: Sender<DriverLocationEvent>,
        r_tx: Sender<OnGoingRideCoordinatesData>,
        stats_tx: Sender<ProcessStataData>,
    ) -> Result<Arc<Self>> {
        let mut connection_options =
            ConnectOptions::new(config.database_url.clone());
        let redis_config = if config.is_dev() {
            tracing::info!(
                "APP_ENV={} - using standalone Redis at localhost:6379",
                config.app_env
            );
            RedisConfig {
                host: "localhost".to_string(),
                port: 6379,
                cluster_enabled: false,
                cluster_urls: Vec::new(),
                use_legacy_version: false,
                pool_size: 50,
                reconnect_max_attempts: 10,
                reconnect_delay: 5000,
                default_ttl: 3600,
                default_hash_ttl: 3600,
                stream_read_count: 100,
                partition: 0,
            }
        } else {
            let cluster_urls: Vec<String> = config
                .redis_cluster_urls
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
            RedisConfig {
                host: config.redis_host.clone(),
                port: config.redis_port,
                cluster_enabled: config.redis_cluster_enabled,
                cluster_urls,
                use_legacy_version: false,
                pool_size: 50,
                reconnect_max_attempts: 10,
                reconnect_delay: 5000,
                default_ttl: 3600,
                default_hash_ttl: 3600,
                stream_read_count: 100,
                partition: 0,
            }
        };
        let redis = Arc::new(
            RedisConnectionPool::new(redis_config)
                .await
                .expect("Failed to create Redis connection pool"),
        );
        connection_options.max_connections(config.database_max_connections);
        let db = Arc::new(Database::new(connection_options, Executor).await?);
        let notif = Arc::new(GorushClient::new(&config.gorush_url));
        let verify = Arc::new(TwilioVerifyClient::new(
            &config.twilio_account_sid,
            &config.twilio_auth_token,
            &config.twilio_verify_service_sid,
        ));

        let driver_pool_manager = Arc::new(
            dispatch::dispatch::DriverPoolManager::new(
                redis.clone(),
                "{drivers}:pool",
                "{drivers}:inflight",
                "{drivers}:request",
            )
            .await
            .expect("Failed to create DriverPoolManager"),
        );
        let dispatch_api_manager =
            Arc::new(api::ride_request::DispatchApiManager::new());

        let (d, job_rx) = api::ride_request::DispatcherQueue::new(100, 1000);
        let dispatcher_queue = Arc::new(d);

        let ctx = Arc::new(Self {
            dispatch_api_manager: dispatch_api_manager.clone(),
            dispatcher_queue: dispatcher_queue.clone(),
            driver_pool_manager: driver_pool_manager.clone(),
            db,
            config,
            tx,
            r_tx,
            stats_tx,
            redis,
            notif,
            verify,
        });
        dispatcher_queue
            .start_workers(
                job_rx,
                driver_pool_manager,
                dispatch_api_manager.clone(),
                dispatcher_queue.semaphore.clone(),
                ctx.clone(),
            )
            .await;

        // Spawn a task to periodically clean up expired driver pools
        tokio::spawn(async move {
            let cleanup_interval = tokio::time::Duration::from_secs(60); // Cleanup every 60 seconds
            let mut interval = tokio::time::interval(cleanup_interval);
            let duration = tokio::time::Duration::from_secs(300); // 5 minutes
            loop {
                interval.tick().await;
                dispatch_api_manager.cleanup_old_requests(duration);
            }
        });

        Ok(ctx)
    }
}
