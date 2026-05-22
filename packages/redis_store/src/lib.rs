pub mod r_types;
pub mod redis;
pub mod events;

use crate::r_types::*;
use fred::interfaces::ClientLike;
use fred::prelude::{PubsubInterface, StreamsInterface};
use fred::types::config::{ConnectionConfig, TcpConfig};
use fred::{prelude::Pool, types::ConnectHandle};
use serde::Deserialize;
use socket2::TcpKeepalive;
use std::ops::Deref;
use std::sync::{
    Arc,
    atomic::{self},
};
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub cluster_enabled: bool,
    pub cluster_urls: Vec<String>,
    pub use_legacy_version: bool,
    pub pool_size: usize,
    pub reconnect_max_attempts: u32,
    /// Reconnect delay in milliseconds
    pub reconnect_delay: u32,
    /// TTL in seconds
    pub default_ttl: u32,
    /// TTL for hash-tables in seconds
    pub default_hash_ttl: u32,
    pub stream_read_count: u64,
    pub partition: usize,
}

impl Default for RedisConfig {
    fn default() -> Self {
        RedisConfig {
            host: String::from("localhost"),
            port: 6379,
            cluster_enabled: false,
            cluster_urls: Vec::new(),
            use_legacy_version: false,
            pool_size: 10,
            reconnect_max_attempts: 5,
            reconnect_delay: 1000,
            default_ttl: 3600,
            default_hash_ttl: 3600,
            stream_read_count: 100,
            partition: 0,
        }
    }
}

impl RedisConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        host: String,
        port: u16,
        pool_size: usize,
        partition: usize,
        reconnect_max_attempts: u32,
        reconnect_delay: u32,
        default_ttl: u32,
        default_hash_ttl: u32,
        stream_read_count: u64,
    ) -> Self {
        RedisConfig {
            host,
            port,
            cluster_enabled: false,
            cluster_urls: Vec::new(),
            use_legacy_version: false,
            pool_size,
            reconnect_max_attempts,
            reconnect_delay,
            default_ttl,
            default_hash_ttl,
            stream_read_count,
            partition,
        }
    }
}

#[derive(Debug)]
pub struct RedisConnectionPool {
    pub join_handle: Vec<fred::types::ConnectHandle>,
    pub available: Arc<atomic::AtomicBool>,
    pub pool: fred::prelude::Pool,
}

pub struct RedisClient {
    inner: fred::prelude::Client,
}

impl Deref for RedisClient {
    type Target = fred::prelude::Client;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RedisClient {
    pub async fn new(
        config: fred::types::config::Config,
        perf: fred::types::config::PerformanceConfig,
        conn: fred::types::config::ConnectionConfig,
        reconnect_policy: fred::types::config::ReconnectPolicy,
    ) -> Result<Self, RedisError> {
        let client = fred::prelude::Client::new(
            config,
            Some(perf),
            Some(conn),
            Some(reconnect_policy),
        );
        let _ = client
            .connect()
            .await
            .map_err(|err| RedisError::RedisConnectionError(err.into()))?;
        client
            .wait_for_connect()
            .await
            .map_err(RedisError::RedisConnectionError)?;

        Ok(Self { inner: client })
    }
}
impl RedisConnectionPool {
    pub async fn new(conf: RedisConfig) -> Result<Self, RedisError> {
        let (pool, join_handle) = Self::init(&conf).await?;
        Ok(Self {
            join_handle,
            available: Arc::new(atomic::AtomicBool::new(true)),
            pool,
        })
    }

    async fn init(
        conf: &RedisConfig,
    ) -> Result<(Pool, Vec<ConnectHandle>), RedisError> {
        let mut redis_config = Self::build_cluster_config(conf)?;

        redis_config.blocking = fred::types::config::Blocking::Error;
        redis_config.tracing = fred::types::config::TracingConfig::new(true);
        redis_config.blocking = fred::types::config::Blocking::Error;
        if conf.use_legacy_version {
            redis_config.version = fred::types::RespVersion::RESP3;
        }
        let reconnect_policy =
            fred::types::config::ReconnectPolicy::new_constant(
                conf.reconnect_max_attempts,
                conf.reconnect_delay,
            );

        let mut per_conf = fred::types::config::PerformanceConfig::default();
        per_conf.broadcast_channel_capacity = 1024 * 10;
        let pool = fred::prelude::Pool::new(
            redis_config,
            Some(per_conf),
            Some(Self::build_connection_config()),
            Some(reconnect_policy),
            conf.pool_size,
        )
        .map_err(RedisError::RedisConnectionError)?;

        let join_handle = pool.connect_pool();
        pool.wait_for_connect()
            .await
            .map_err(RedisError::RedisConnectionError)?;

        Ok((pool, join_handle))
    }

    fn build_connection_config() -> ConnectionConfig {
        // Send TCP keepalive probes every 15s after 30s idle so HAProxy
        // never sees the connection as idle and resets it (ECONNRESET / code 104).
        let keepalive = TcpKeepalive::new()
            .with_time(Duration::from_secs(30))
            .with_interval(Duration::from_secs(15));
        ConnectionConfig {
            tcp: TcpConfig {
                keepalive: Some(keepalive),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn build_cluster_config(
        conf: &RedisConfig,
    ) -> Result<fred::prelude::Config, RedisError> {
        let redis_url = conf
            .cluster_enabled
            .then(|| {
                let nodes = conf
                    .cluster_urls
                    .iter()
                    .map(|url| {
                        // Handle both "host:port" and full URL formats
                        let clean_url =
                            url.trim().trim_start_matches("redis://");
                        format!("node={}", clean_url)
                    })
                    .collect::<Vec<String>>()
                    .join("&");
                format!("redis-cluster://{}:{}?{}", conf.host, conf.port, nodes)
            })
            .unwrap_or(format!(
                "redis://{}:{}/{}",
                conf.host, conf.port, conf.partition
            ));

        tracing::info!(
            "Connecting to Redis ({}): {:?}",
            if conf.cluster_enabled { "cluster" } else { "standalone" },
            redis_url
        );
        // "redis-cluster://notif_test-redis-cluster-1:6379"
        let mut redis_config: fred::prelude::Config =
            fred::types::config::Config::from_url(&redis_url)
                .map_err(|err| RedisError::RedisConnectionError(err))?;
        // only set_cluster_discovery_policy if cluster is enabled
        if conf.cluster_enabled {
            redis_config
                .server
                .set_cluster_discovery_policy(
                    fred::types::config::ClusterDiscoveryPolicy::ConfigEndpoint,
                )
                .expect("Failed to set discovery policy.");
        }

        Ok(redis_config)
    }

    ///A generic function to publish messages to a Redis channel.
    /// # arguments
    /// * `channel` - The Redis channel to publish to.
    /// * `message` - The message to publish.
    pub async fn publish_message<T: serde::Serialize + Send + Sync>(
        &self,
        channel: &str,
        message: &T,
    ) -> Result<(), RedisError> {
        let clients = self.pool.next();
        let serialized_message = serde_json::to_string(message)
            .map_err(|err| RedisError::SerializationError(err.to_string()))?;
        clients
            .publish::<(), _, _>(channel, serialized_message)
            .await
            .map_err(RedisError::RedisPublishError)?;
        Ok(())
    }

    /// Serialize `event` as JSON and append it to `stream` with an auto-generated ID (`*`).
    /// Returns the generated message ID (e.g. `"1678885450410-0"`).
    pub async fn xadd_event<T: serde::Serialize + Send + Sync>(
        &self,
        stream: &str,
        event: &T,
    ) -> Result<String, RedisError> {
        let client = self.pool.next();
        let payload = serde_json::to_string(event)
            .map_err(|e| RedisError::SerializationError(e.to_string()))?;
        client
            .xadd(stream, false, None, "*", ("payload", payload))
            .await
            .map_err(RedisError::RedisStreamError)
    }

    /// Create a consumer group on `stream` if it does not already exist.
    /// Passes `MKSTREAM` so the stream key is created automatically when absent.
    /// Silently ignores `BUSYGROUP` errors produced on service restart.
    pub async fn ensure_consumer_group(
        &self,
        stream: &str,
        group: &str,
    ) -> Result<(), RedisError> {
        let client = self.pool.next();
        match client
            .xgroup_create::<(), _, _, _>(stream, group, "$", true)
            .await
        {
            Ok(()) => Ok(()),
            Err(e) if e.details().contains("BUSYGROUP") => Ok(()),
            Err(e) => Err(RedisError::RedisStreamError(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{
        DriverArrivedEvent, DriverArrivedEventPayload, DriverArrivedPayload, DriverInfo,
        EventPayload, EventsManger, GeoLocation, NextDriverOfferEvent,
        NextDriverOfferEventPayload, Rating, RideCancelPayload, RideCanceledEvent, RideEndEvent,
        RideEndEventPayload, RideEndPayload, RideStartEvent, RideStartEventPayload,
        RideStartPayload, NEXT_DRIVER_OFFERS_GROUP, NEXT_DRIVER_OFFERS_STREAM,
        RIDE_EVENTS_GROUP, RIDE_EVENTS_STREAM,
    };

    // ── Helpers ──────────────────────────────────────────────────────────────────

    fn localhost_config() -> RedisConfig {
        RedisConfig {
            host: "localhost".to_string(),
            port: 6379,
            ..Default::default()
        }
    }

    fn geo_location(lat: f64, lng: f64) -> GeoLocation {
        GeoLocation { latitude: lat, longitude: lng, address: String::new(), place_id: String::new() }
    }

    // ── RedisConfig ──────────────────────────────────────────────────────────────

    #[test]
    fn test_redis_config_default_values() {
        let c = RedisConfig::default();
        assert_eq!(c.host, "localhost");
        assert_eq!(c.port, 6379);
        assert!(!c.cluster_enabled);
        assert!(c.cluster_urls.is_empty());
        assert!(!c.use_legacy_version);
        assert_eq!(c.pool_size, 10);
        assert_eq!(c.reconnect_max_attempts, 5);
        assert_eq!(c.reconnect_delay, 1000);
        assert_eq!(c.default_ttl, 3600);
        assert_eq!(c.default_hash_ttl, 3600);
        assert_eq!(c.stream_read_count, 100);
        assert_eq!(c.partition, 0);
    }

    #[test]
    fn test_redis_config_new_sets_all_fields() {
        let c = RedisConfig::new(
            "myhost".to_string(), 6380, 5, 1, 3, 500, 7200, 7200, 50,
        );
        assert_eq!(c.host, "myhost");
        assert_eq!(c.port, 6380);
        assert_eq!(c.pool_size, 5);
        assert_eq!(c.partition, 1);
        assert_eq!(c.reconnect_max_attempts, 3);
        assert_eq!(c.reconnect_delay, 500);
        assert_eq!(c.default_ttl, 7200);
        assert_eq!(c.default_hash_ttl, 7200);
        assert_eq!(c.stream_read_count, 50);
        // new() always sets these two to false
        assert!(!c.cluster_enabled);
        assert!(!c.use_legacy_version);
    }

    // ── build_cluster_config ─────────────────────────────────────────────────────

    #[test]
    fn test_build_cluster_config_standalone_succeeds() {
        let conf = RedisConfig { port: 6379, partition: 2, ..Default::default() };
        assert!(RedisConnectionPool::build_cluster_config(&conf).is_ok());
    }

    #[test]
    fn test_build_cluster_config_standalone_uses_partition() {
        // port 6380, partition 3 → URL should embed those values
        let conf = RedisConfig { port: 6380, partition: 3, ..Default::default() };
        // Just assert it parses without error; fred validates the URL internally
        assert!(RedisConnectionPool::build_cluster_config(&conf).is_ok());
    }

    #[test]
    fn test_build_cluster_config_cluster_mode_succeeds() {
        let conf = RedisConfig {
            host: "redis-node-1".to_string(),
            port: 6379,
            cluster_enabled: true,
            cluster_urls: vec![
                "redis-node-2:6379".to_string(),
                "redis-node-3:6379".to_string(),
            ],
            ..Default::default()
        };
        assert!(RedisConnectionPool::build_cluster_config(&conf).is_ok());
    }

    #[test]
    fn test_build_cluster_config_cluster_strips_redis_prefix() {
        // URLs with "redis://" prefix should be cleaned before embedding
        let conf = RedisConfig {
            host: "primary".to_string(),
            port: 6379,
            cluster_enabled: true,
            cluster_urls: vec!["redis://node-2:6379".to_string()],
            ..Default::default()
        };
        assert!(RedisConnectionPool::build_cluster_config(&conf).is_ok());
    }

    // ── Stream / group constants ─────────────────────────────────────────────────

    #[test]
    fn test_stream_and_group_constants_match_backend() {
        assert_eq!(RIDE_EVENTS_STREAM, "swg:stream:ride_events");
        assert_eq!(NEXT_DRIVER_OFFERS_STREAM, "swg:stream:next_driver_offers");
        assert_eq!(RIDE_EVENTS_GROUP, "ride-event-workers");
        assert_eq!(NEXT_DRIVER_OFFERS_GROUP, "driver-offer-workers");
    }

    // ── EventsManger ─────────────────────────────────────────────────────────────

    #[test]
    fn test_events_manager_stores_stream_key() {
        let em = EventsManger::new("swg:stream:ride_events");
        assert_eq!(em.stream_key, "swg:stream:ride_events");
    }

    #[test]
    fn test_events_manager_accepts_arbitrary_stream_key() {
        let em = EventsManger::new("custom:stream");
        assert_eq!(em.stream_key, "custom:stream");
    }

    // ── RideCanceledEvent serde ──────────────────────────────────────────────────

    #[test]
    fn test_ride_canceled_event_round_trip() {
        let event = RideCanceledEvent {
            event_id: "evt-001".to_string(),
            correlation_id: Some("corr-001".to_string()),
            timestamp: 1_700_000_000,
            event_type: "RideCancel".to_string(),
            ride_id: "ride-001".to_string(),
            driver_id: "driver-001".to_string(),
            rider_id: "rider-001".to_string(),
            priority: 1,
            ack_required: true,
            event_payload: EventPayload {
                ride_cancel: RideCancelPayload {
                    reason: "user_request".to_string(),
                    canceled_by: 1,
                    refund_amount: 50.0,
                    cancellation_fee: "0".to_string(),
                    note: "no issues".to_string(),
                },
            },
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: RideCanceledEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.event_id, "evt-001");
        assert_eq!(decoded.ride_id, "ride-001");
        assert_eq!(decoded.correlation_id, Some("corr-001".to_string()));
        assert_eq!(decoded.event_payload.ride_cancel.reason, "user_request");
        assert!((decoded.event_payload.ride_cancel.refund_amount - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ride_canceled_event_no_correlation_id() {
        let event = RideCanceledEvent { correlation_id: None, ..Default::default() };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: RideCanceledEvent = serde_json::from_str(&json).expect("deserialize");
        assert!(decoded.correlation_id.is_none());
    }

    #[test]
    fn test_event_payload_serde_rename_ride_cancel() {
        let payload = EventPayload {
            ride_cancel: RideCancelPayload { reason: "test".to_string(), ..Default::default() },
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(json.contains("\"RideCancel\""), "expected 'RideCancel' rename in: {json}");
        assert!(!json.contains("\"ride_cancel\""), "snake_case key must not appear in: {json}");
    }

    // ── DriverArrivedEvent serde ─────────────────────────────────────────────────

    #[test]
    fn test_driver_arrived_event_round_trip() {
        let event = DriverArrivedEvent {
            event_id: "evt-002".to_string(),
            timestamp: 1_700_000_001,
            event_type: "DriverArrived".to_string(),
            ride_id: "ride-001".to_string(),
            driver_id: "driver-001".to_string(),
            rider_id: "rider-001".to_string(),
            priority: 2,
            ack_required: false,
            event_payload: DriverArrivedEventPayload {
                driver_arrived: DriverArrivedPayload {
                    arrival_location: geo_location(-1.2921, 36.8219),
                    actual_arrival_time: 1_700_000_001,
                    wait_time_seconds: 60,
                },
            },
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: DriverArrivedEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.event_id, "evt-002");
        assert_eq!(decoded.event_payload.driver_arrived.wait_time_seconds, 60);
        assert!((decoded.event_payload.driver_arrived.arrival_location.latitude - (-1.2921)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_driver_arrived_payload_serde_rename() {
        let payload = DriverArrivedEventPayload {
            driver_arrived: DriverArrivedPayload {
                arrival_location: geo_location(0.0, 0.0),
                actual_arrival_time: 0,
                wait_time_seconds: 0,
            },
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(json.contains("\"DriverArrived\""), "expected 'DriverArrived' rename in: {json}");
    }

    // ── RideStartEvent serde ─────────────────────────────────────────────────────

    #[test]
    fn test_ride_start_event_with_driver_info_round_trip() {
        let event = RideStartEvent {
            event_id: "evt-003".to_string(),
            timestamp: 1_700_000_002,
            event_type: "RideStart".to_string(),
            ride_id: "ride-001".to_string(),
            driver_id: "driver-001".to_string(),
            rider_id: "rider-001".to_string(),
            priority: 1,
            ack_required: true,
            event_payload: RideStartEventPayload {
                ride_start: RideStartPayload {
                    start_location: geo_location(-1.2921, 36.8219),
                    destination: geo_location(-1.3000, 36.8300),
                    estimated_fare: 250.0,
                    vehicle_type: "boda".to_string(),
                    vehicle_number: "KBZ 001A".to_string(),
                    driver_info: Some(DriverInfo {
                        driver_id: "driver-001".to_string(),
                        name: "Jane Doe".to_string(),
                        phone: "+254700000000".to_string(),
                        photo_url: "https://example.com/photo.jpg".to_string(),
                        rating: 4.9,
                        license_plate: "KBZ 001A".to_string(),
                    }),
                    estimated_duration: 900,
                },
            },
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: RideStartEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.event_payload.ride_start.estimated_duration, 900);
        let info = decoded.event_payload.ride_start.driver_info.expect("driver_info present");
        assert_eq!(info.name, "Jane Doe");
        assert!((info.rating - 4.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ride_start_event_no_driver_info() {
        let event = RideStartEvent {
            event_id: "evt-004".to_string(),
            timestamp: 0,
            event_type: "RideStart".to_string(),
            ride_id: "r-2".to_string(),
            driver_id: "d-2".to_string(),
            rider_id: "r-2".to_string(),
            priority: 1,
            ack_required: false,
            event_payload: RideStartEventPayload {
                ride_start: RideStartPayload {
                    start_location: geo_location(0.0, 0.0),
                    destination: geo_location(0.0, 0.0),
                    estimated_fare: 0.0,
                    vehicle_type: String::new(),
                    vehicle_number: String::new(),
                    driver_info: None,
                    estimated_duration: 0,
                },
            },
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: RideStartEvent = serde_json::from_str(&json).expect("deserialize");
        assert!(decoded.event_payload.ride_start.driver_info.is_none());
    }

    #[test]
    fn test_ride_start_payload_serde_rename() {
        let payload = RideStartEventPayload {
            ride_start: RideStartPayload {
                start_location: geo_location(0.0, 0.0),
                destination: geo_location(0.0, 0.0),
                estimated_fare: 0.0,
                vehicle_type: String::new(),
                vehicle_number: String::new(),
                driver_info: None,
                estimated_duration: 0,
            },
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(json.contains("\"RideStart\""), "expected 'RideStart' rename in: {json}");
    }

    // ── RideEndEvent serde ───────────────────────────────────────────────────────

    #[test]
    fn test_ride_end_event_with_ratings_round_trip() {
        let event = RideEndEvent {
            event_id: "evt-005".to_string(),
            timestamp: 1_700_000_003,
            event_type: "RideEnd".to_string(),
            ride_id: "ride-001".to_string(),
            driver_id: "driver-001".to_string(),
            rider_id: "rider-001".to_string(),
            priority: 1,
            ack_required: false,
            event_payload: RideEndEventPayload {
                ride_end: RideEndPayload {
                    end_location: geo_location(-1.3000, 36.8300),
                    distance_km: 5.2,
                    duration_seconds: 920,
                    final_fare: 260.0,
                    rider_rating: Some(Rating {
                        score: 5,
                        comment: "Great ride!".to_string(),
                        tags: vec!["polite".to_string(), "on-time".to_string()],
                    }),
                    driver_rating: None,
                },
            },
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: RideEndEvent = serde_json::from_str(&json).expect("deserialize");
        assert!((decoded.event_payload.ride_end.distance_km - 5.2).abs() < f64::EPSILON);
        let rr = decoded.event_payload.ride_end.rider_rating.expect("rider_rating present");
        assert_eq!(rr.score, 5);
        assert_eq!(rr.tags, vec!["polite", "on-time"]);
        assert!(decoded.event_payload.ride_end.driver_rating.is_none());
    }

    #[test]
    fn test_ride_end_event_both_ratings_none() {
        let event = RideEndEvent {
            event_id: "evt-006".to_string(),
            timestamp: 0,
            event_type: "RideEnd".to_string(),
            ride_id: "r-3".to_string(),
            driver_id: "d-3".to_string(),
            rider_id: "r-3".to_string(),
            priority: 0,
            ack_required: false,
            event_payload: RideEndEventPayload {
                ride_end: RideEndPayload {
                    end_location: geo_location(0.0, 0.0),
                    distance_km: 0.0,
                    duration_seconds: 0,
                    final_fare: 0.0,
                    rider_rating: None,
                    driver_rating: None,
                },
            },
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: RideEndEvent = serde_json::from_str(&json).expect("deserialize");
        assert!(decoded.event_payload.ride_end.rider_rating.is_none());
        assert!(decoded.event_payload.ride_end.driver_rating.is_none());
    }

    #[test]
    fn test_ride_end_payload_serde_rename() {
        let payload = RideEndEventPayload {
            ride_end: RideEndPayload {
                end_location: geo_location(0.0, 0.0),
                distance_km: 0.0,
                duration_seconds: 0,
                final_fare: 0.0,
                rider_rating: None,
                driver_rating: None,
            },
        };
        let json = serde_json::to_string(&payload).expect("serialize");
        assert!(json.contains("\"RideEnd\""), "expected 'RideEnd' rename in: {json}");
    }

    // ── NextDriverOfferEvent serde ───────────────────────────────────────────────

    #[test]
    fn test_next_driver_offer_event_with_payload_round_trip() {
        let event = NextDriverOfferEvent {
            event_id: "evt-007".to_string(),
            timestamp: 1_700_000_004,
            ride_request_id: "req-001".to_string(),
            status: 1,
            payload: Some(NextDriverOfferEventPayload {
                rider_id: "rider-001".to_string(),
                driver_image: "https://example.com/img.jpg".to_string(),
                driver_name: "John".to_string(),
                driver_rating: 4.8,
                ride_id: "ride-001".to_string(),
                latitude: -1.2921,
                longitude: 36.8219,
                distance_km: 2.5,
                estimated_arrival_time: 5.0,
                timestamp: 1_700_000_004,
            }),
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: NextDriverOfferEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.event_id, "evt-007");
        assert_eq!(decoded.ride_request_id, "req-001");
        let p = decoded.payload.expect("payload present");
        assert_eq!(p.driver_name, "John");
        assert!((p.driver_rating - 4.8).abs() < f64::EPSILON);
        assert!((p.distance_km - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_next_driver_offer_event_none_payload() {
        let event = NextDriverOfferEvent {
            event_id: "evt-008".to_string(),
            timestamp: 0,
            ride_request_id: "req-002".to_string(),
            status: 0,
            payload: None,
        };
        let json = serde_json::to_string(&event).expect("serialize");
        let decoded: NextDriverOfferEvent = serde_json::from_str(&json).expect("deserialize");
        assert!(decoded.payload.is_none());
    }

    #[test]
    fn test_next_driver_offer_event_equality() {
        let base = NextDriverOfferEvent {
            event_id: "e".to_string(),
            timestamp: 100,
            ride_request_id: "r".to_string(),
            status: 1,
            payload: None,
        };
        assert_eq!(base, base.clone());
    }

    // ── Integration tests (need live Redis on localhost:6379) ────────────────────

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_redis_connection_pool_creation() {
        let result = RedisConnectionPool::new(localhost_config()).await;
        assert!(result.is_ok(), "pool creation failed: {:?}", result.err());
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_xadd_event_returns_valid_message_id() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        let event = RideCanceledEvent { event_id: "xadd-test".to_string(), ..Default::default() };
        let msg_id = pool.xadd_event("swg:test:xadd", &event).await.expect("xadd_event");
        assert!(!msg_id.is_empty());
        assert!(msg_id.contains('-'), "ID must be <ms>-<seq>: {msg_id}");
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_ensure_consumer_group_creates_group() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        pool.xadd_event("swg:test:group_stream", &"seed").await.unwrap();
        let result = pool.ensure_consumer_group("swg:test:group_stream", "test-workers").await;
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_ensure_consumer_group_idempotent_busygroup() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        pool.xadd_event("swg:test:idem_stream", &"seed").await.unwrap();
        pool.ensure_consumer_group("swg:test:idem_stream", "idem-workers").await.unwrap();
        // Second call must silently succeed (BUSYGROUP ignored)
        let result = pool.ensure_consumer_group("swg:test:idem_stream", "idem-workers").await;
        assert!(result.is_ok(), "BUSYGROUP must be silently ignored: {:?}", result.err());
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_ensure_all_groups() {
        use crate::events::ensure_all_groups;
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        let result = ensure_all_groups(&pool).await;
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_events_manager_publish_event() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        let em = EventsManger::new("swg:test:publish");
        let event = RideCanceledEvent { event_id: "pub-1".to_string(), ..Default::default() };
        let result = em.publish_event(Some(&event), &pool).await;
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_events_manager_publish_none_event() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        let em = EventsManger::new("swg:test:publish");
        // None serializes as JSON null in the payload field — still a valid stream entry
        let result = em.publish_event::<String>(None, &pool).await;
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_publish_message_to_channel() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        let msg = r_types::LocationEvent {
            entity_id: "entity_123".to_string(),
            ride_id: "ride_001".to_string(),
            latitude: -1.2921,
            longitude: 36.8219,
            timestamp: chrono::Utc::now().timestamp_millis(),
            accuracy: 20.5,
            speed: 30,
            bearing: 150.0,
        };
        let result = pool.publish_message("driver_location_change_channel", &msg).await;
        assert!(result.is_ok(), "{:?}", result.err());
    }

    // ── Publish + read-back tests (real Redis, full message round-trip) ───────────

    /// Helper: read all entries from `stream` via a non-blocking XREAD and return
    /// the `payload` field values.  Works because the pool uses `Blocking::Error`
    /// only for *blocking* calls; `block=None` is a plain synchronous XREAD.
    async fn xread_all_payloads(
        pool: &RedisConnectionPool,
        stream: &str,
    ) -> Vec<String> {
        use fred::interfaces::StreamsInterface;
        let client = pool.pool.next();
        let result: Result<
            fred::types::streams::XReadResponse<String, String, String, String>,
            _,
        > = client
            .xread_map(Some(100u64), None::<u64>, stream, "0-0")
            .await;
        match result {
            Ok(response) => response
                .get(stream)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|(_, fields)| fields.get("payload").cloned())
                .collect(),
            Err(e) if e.details().contains("Cannot convert to map") => vec![],
            Err(e) => panic!("xread_map failed: {e}"),
        }
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_ride_canceled_event_published_to_stream_and_readable() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        let stream = "swg:test:ride_cancel_verify";

        let unique_id = format!("verify-cancel-{}", chrono::Utc::now().timestamp_millis());
        let event = RideCanceledEvent {
            event_id: unique_id.clone(),
            ride_id: "ride-verify-001".to_string(),
            driver_id: "driver-verify-001".to_string(),
            rider_id: "rider-verify-001".to_string(),
            event_type: "RideCancel".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            event_payload: EventPayload {
                ride_cancel: RideCancelPayload {
                    reason: "user_request".to_string(),
                    canceled_by: 1,
                    refund_amount: 100.0,
                    ..Default::default()
                },
            },
            ..Default::default()
        };

        let em = EventsManger::new(stream);
        em.publish_event(Some(&event), &pool).await.expect("publish_event failed");

        // Read all payloads and find the one we just published by event_id
        let payloads = xread_all_payloads(&pool, stream).await;
        assert!(!payloads.is_empty(), "stream should have at least one message");

        let decoded = payloads
            .iter()
            .filter_map(|p| serde_json::from_str::<RideCanceledEvent>(p).ok())
            .find(|e| e.event_id == unique_id)
            .expect("published event not found in stream");
        println!("Decoded event from stream: {:?}", decoded);

        assert_eq!(decoded.ride_id, "ride-verify-001");
        assert_eq!(decoded.driver_id, "driver-verify-001");
        assert_eq!(decoded.rider_id, "rider-verify-001");
        assert_eq!(decoded.event_payload.ride_cancel.reason, "user_request");
        assert!((decoded.event_payload.ride_cancel.refund_amount - 100.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_next_driver_offer_event_published_to_stream_and_readable() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        let stream = "swg:test:next_offer_verify";

        let unique_id = format!("verify-offer-{}", chrono::Utc::now().timestamp_millis());
        let event = NextDriverOfferEvent {
            event_id: unique_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            ride_request_id: "req-verify-001".to_string(),
            status: 1,
            payload: Some(NextDriverOfferEventPayload {
                rider_id: "rider-verify-001".to_string(),
                driver_image: "https://example.com/driver.jpg".to_string(),
                driver_name: "Test Driver".to_string(),
                driver_rating: 4.7,
                ride_id: "ride-verify-001".to_string(),
                latitude: -1.2921,
                longitude: 36.8219,
                distance_km: 1.5,
                estimated_arrival_time: 3.0,
                timestamp: chrono::Utc::now().timestamp_millis(),
            }),
        };

        let em = EventsManger::new(stream);
        em.publish_event(Some(&event), &pool).await.expect("publish_event failed");

        let payloads = xread_all_payloads(&pool, stream).await;
        assert!(!payloads.is_empty(), "stream should have at least one message");

        let decoded = payloads
            .iter()
            .filter_map(|p| serde_json::from_str::<NextDriverOfferEvent>(p).ok())
            .find(|e| e.event_id == unique_id)
            .expect("published event not found in stream");

        assert_eq!(decoded.ride_request_id, "req-verify-001");
        assert_eq!(decoded.status, 1);
        let p = decoded.payload.expect("payload present");
        assert_eq!(p.driver_name, "Test Driver");
        assert!((p.driver_rating - 4.7).abs() < f64::EPSILON);
        assert!((p.distance_km - 1.5).abs() < f64::EPSILON);
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_ride_start_event_published_to_stream_and_readable() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        let stream = "swg:test:ride_start_verify";

        let unique_id = format!("verify-start-{}", chrono::Utc::now().timestamp_millis());
        let event = RideStartEvent {
            event_id: unique_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            event_type: "RideStart".to_string(),
            ride_id: "ride-start-001".to_string(),
            driver_id: "driver-start-001".to_string(),
            rider_id: "rider-start-001".to_string(),
            priority: 1,
            ack_required: true,
            event_payload: RideStartEventPayload {
                ride_start: RideStartPayload {
                    start_location: geo_location(-1.2921, 36.8219),
                    destination: geo_location(-1.3000, 36.8300),
                    estimated_fare: 300.0,
                    vehicle_type: "boda".to_string(),
                    vehicle_number: "KCA 001X".to_string(),
                    driver_info: Some(DriverInfo {
                        driver_id: "driver-start-001".to_string(),
                        name: "Test Driver".to_string(),
                        phone: "+254700000001".to_string(),
                        photo_url: String::new(),
                        rating: 4.6,
                        license_plate: "KCA 001X".to_string(),
                    }),
                    estimated_duration: 600,
                },
            },
        };

        let em = EventsManger::new(stream);
        em.publish_event(Some(&event), &pool).await.expect("publish_event failed");

        let payloads = xread_all_payloads(&pool, stream).await;
        let decoded = payloads
            .iter()
            .filter_map(|p| serde_json::from_str::<RideStartEvent>(p).ok())
            .find(|e| e.event_id == unique_id)
            .expect("RideStartEvent not found in stream");

        assert_eq!(decoded.ride_id, "ride-start-001");
        let info = decoded.event_payload.ride_start.driver_info.expect("driver_info");
        assert_eq!(info.name, "Test Driver");
        assert_eq!(decoded.event_payload.ride_start.estimated_duration, 600);
    }

    #[tokio::test]
    #[ignore = "requires Redis on localhost:6379"]
    async fn test_ride_end_event_published_to_stream_and_readable() {
        let pool = RedisConnectionPool::new(localhost_config()).await.unwrap();
        let stream = "swg:test:ride_end_verify";

        let unique_id = format!("verify-end-{}", chrono::Utc::now().timestamp_millis());
        let event = RideEndEvent {
            event_id: unique_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            event_type: "RideEnd".to_string(),
            ride_id: "ride-end-001".to_string(),
            driver_id: "driver-end-001".to_string(),
            rider_id: "rider-end-001".to_string(),
            priority: 1,
            ack_required: false,
            event_payload: RideEndEventPayload {
                ride_end: RideEndPayload {
                    end_location: geo_location(-1.3000, 36.8300),
                    distance_km: 8.4,
                    duration_seconds: 1200,
                    final_fare: 420.0,
                    rider_rating: Some(Rating {
                        score: 5,
                        comment: "Smooth ride".to_string(),
                        tags: vec!["on-time".to_string()],
                    }),
                    driver_rating: Some(Rating {
                        score: 4,
                        comment: "Good passenger".to_string(),
                        tags: vec![],
                    }),
                },
            },
        };

        let em = EventsManger::new(stream);
        em.publish_event(Some(&event), &pool).await.expect("publish_event failed");

        let payloads = xread_all_payloads(&pool, stream).await;
        let decoded = payloads
            .iter()
            .filter_map(|p| serde_json::from_str::<RideEndEvent>(p).ok())
            .find(|e| e.event_id == unique_id)
            .expect("RideEndEvent not found in stream");

        assert_eq!(decoded.ride_id, "ride-end-001");
        assert!((decoded.event_payload.ride_end.distance_km - 8.4).abs() < f64::EPSILON);
        assert!((decoded.event_payload.ride_end.final_fare - 420.0).abs() < f64::EPSILON);
        assert_eq!(decoded.event_payload.ride_end.rider_rating.unwrap().score, 5);
        assert_eq!(decoded.event_payload.ride_end.driver_rating.unwrap().score, 4);
    }
}