//! `beckn-bpp-adapter` binary entry point.

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use beckn_bpp_adapter::AppState;
use beckn_bpp_adapter::adapters::{
    DiscoveryAdapter, DispatchAdapter, HttpDispatchAdapter, SitwegoDiscovery,
    UnconfiguredDiscovery, UnconfiguredDispatch,
};
use beckn_bpp_adapter::app::router;
use beckn_bpp_adapter::callbacks::HttpCallbackClient;
use beckn_bpp_adapter::config::Config;
use beckn_bpp_adapter::correlation::{
    CorrelationStore, MemoryCorrelationStore,
};
use beckn_bpp_adapter::crypto::{
    HttpRegistryClient, RegistryClient, Signer, StaticRegistry,
};
use db_store::Database;
use dotenv::dotenv;
use redis_store::{RedisConfig, RedisConnectionPool};
use tokio::net::TcpListener;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let mut config = Config::from_env().unwrap_or_else(|e| {
        tracing::warn!("config from env failed ({e}); using defaults");
        Config::default()
    });

    // Verification needs a registry to resolve signers' keys. Without one,
    // dev degrades to unverified; anything else refuses to boot.
    let registry: Arc<dyn RegistryClient> = match &config.beckn_registry_url {
        Some(url) => Arc::new(HttpRegistryClient::new(url.clone())),
        None if config.beckn_verify_signatures => {
            if !config.is_dev() {
                anyhow::bail!(
                    "BECKN_REGISTRY_URL is required when signature \
                     verification is enabled outside dev"
                );
            }
            warn!(
                "no BECKN_REGISTRY_URL — DISABLING signature verification \
                 (dev only)"
            );
            config.beckn_verify_signatures = false;
            Arc::new(StaticRegistry::default())
        }
        None => Arc::new(StaticRegistry::default()),
    };

    // Fail fast on a malformed signing key; missing is allowed (dev) but loud.
    let signer = Signer::from_config(&config)?;
    if signer.is_none() {
        warn!(
            "no BECKN_SIGNING_PRIVATE_KEY — outbound on_* callbacks will be \
             UNSIGNED and rejected by network peers"
        );
    }

    // Discovery + correlation need Postgres, Redis and the routing service —
    // the same backing services the main app uses. Missing pieces degrade in
    // dev (search ACKs but no on_search goes out) and refuse to boot elsewhere.
    type Backing = (
        Arc<dyn CorrelationStore>,
        Arc<dyn DiscoveryAdapter>,
        Option<Arc<RedisConnectionPool>>,
    );
    let (correlation, discovery, redis): Backing =
        match (&config.database_url, &config.routes_api_url) {
            (Some(db_url), Some(routes_url)) => {
                let db = Arc::new(
                    Database::new(
                        db_store::ConnectOptions::new(db_url.clone()),
                        utils::executor::Executor,
                    )
                    .await?,
                );
                let redis = Arc::new(
                    RedisConnectionPool::new(redis_config(&config))
                        .await
                        .map_err(|e| {
                            anyhow::anyhow!("redis connect failed: {e:?}")
                        })?,
                );
                let discovery = SitwegoDiscovery::new(
                    db.clone(),
                    redis.clone(),
                    routes_url.clone(),
                );
                (db, Arc::new(discovery), Some(redis))
            }
            _ if config.is_dev() => {
                warn!(
                    "DATABASE_URL / ROUTES_API_URL not set — using in-memory \
                 correlation and NO discovery (dev only): search will ACK \
                 but no on_search will be sent"
                );
                (
                    Arc::new(MemoryCorrelationStore::default()),
                    Arc::new(UnconfiguredDiscovery),
                    None,
                )
            }
            _ => anyhow::bail!(
                "DATABASE_URL and ROUTES_API_URL are required outside dev"
            ),
        };

    // Dispatch handoff (Step 6): confirm → api service's private internal
    // plane. All three settings are required together; dev may run without
    // (confirm ACKs, then reports the failure to the BAP).
    let dispatch: Arc<dyn DispatchAdapter> = match (
        &config.sitwego_internal_api_url,
        &config.beckn_internal_token,
        &config.beckn_network_rider_id,
    ) {
        (Some(url), Some(token), Some(rider_id)) => {
            Arc::new(HttpDispatchAdapter::new(
                url.clone(),
                token.clone(),
                rider_id.clone(),
            ))
        }
        _ if config.is_dev() => {
            warn!(
                "SITWEGO_INTERNAL_API_URL / BECKN_INTERNAL_TOKEN / \
                 BECKN_NETWORK_RIDER_ID not set — dispatch handoff DISABLED \
                 (dev only): confirm will ACK but report an error callback"
            );
            Arc::new(UnconfiguredDispatch)
        }
        _ => anyhow::bail!(
            "SITWEGO_INTERNAL_API_URL, BECKN_INTERNAL_TOKEN and \
             BECKN_NETWORK_RIDER_ID are required outside dev"
        ),
    };

    // The ride-event consumer holds blocking XREADGROUPs, and fred rejects
    // any command routed to a connection that is mid-block — sharing its pool
    // with the request path makes discovery/correlation Redis calls fail at
    // random. Blocking reads get a small dedicated pool instead.
    let events_redis = match &redis {
        Some(_) => {
            let mut consumer_config = redis_config(&config);
            consumer_config.pool_size = 2;
            Some(Arc::new(
                RedisConnectionPool::new(consumer_config).await.map_err(
                    |e| anyhow::anyhow!("events redis connect failed: {e:?}"),
                )?,
            ))
        }
        None => None,
    };

    let port = config.beckn_port;
    info!(
        subscriber_id = %config.beckn_subscriber_id,
        domain = %config.beckn_domain,
        verify_signatures = config.beckn_verify_signatures,
        signed_callbacks = signer.is_some(),
        "starting beckn-bpp-adapter"
    );

    let callbacks = Arc::new(HttpCallbackClient::new(signer));
    let state = AppState::new(
        config,
        callbacks,
        registry,
        correlation,
        discovery,
        dispatch,
    );

    // Step 7: follow every ride's lifecycle (arrived/started/ended/cancelled)
    // off the shared Redis stream and translate the Beckn-owned ones into
    // unsolicited on_status/on_cancel callbacks.
    match &events_redis {
        Some(events_redis) => {
            tokio::spawn(beckn_bpp_adapter::events::run_ride_event_consumer(
                state.clone(),
                events_redis.clone(),
            ));
        }
        None => warn!(
            "no Redis — ride lifecycle events will NOT reach the network \
             (dev only)"
        ),
    }

    let app = router(state);

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let listener = TcpListener::bind(addr).await?;
    info!("🚀 beckn-bpp-adapter listening on http://{addr}");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

/// Mirror the api crate's Redis setup: standalone in dev, cluster otherwise.
fn redis_config(config: &Config) -> RedisConfig {
    if config.is_dev() {
        info!(
            "APP_ENV={} - using standalone Redis at localhost:6379",
            config.app_env
        );
        return RedisConfig {
            host: "localhost".to_string(),
            port: 6379,
            cluster_enabled: false,
            cluster_urls: Vec::new(),
            use_legacy_version: false,
            pool_size: 10,
            reconnect_max_attempts: 10,
            reconnect_delay: 5000,
            default_ttl: 3600,
            default_hash_ttl: 3600,
            stream_read_count: 100,
            partition: 0,
        };
    }
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
        pool_size: 10,
        reconnect_max_attempts: 10,
        reconnect_delay: 5000,
        default_ttl: 3600,
        default_hash_ttl: 3600,
        stream_read_count: 100,
        partition: 0,
    }
}
