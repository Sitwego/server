use api::api::driver::payment;
use api::api::{auth_handlers, docs, handlers};
use api::auth_middleware::auth_middleware;
use api::cache::drainner::RideLocationInfoDrainner;
use api::cache::process_on_going_ride_coordinates::{
    OnGoingRideCoordinatesData, ProcessOnGoingRideCoordinates,
};
use api::cache::process_stats::{ProcessStataData, ProcessStats};
use api::migrations::run_db_migrations;
use api::queries::bussines::SubscriptionsPlans;
use api::{APIContext, config, types::*};
use axum::routing::post;
use axum::{
    Extension, Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    routing::get,
};
use axum::{Json, middleware};
pub use db_store::{ConnectOptions, Database};
use dotenv::dotenv;
use mimalloc::MiMalloc;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::{env, path::Path};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_cron_scheduler::JobSchedulerError;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{info, info_span};
use utils::{Result, executor::Executor};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
#[allow(clippy::result_large_err)]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let tracing_layer = tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(|req: &axum::extract::Request| {
            let url = req.uri().to_string();
            info_span!("http_reuest", method = ?req.method(), url)
        });
    let config = envy::from_env::<config::config::Config>()
        .expect("Error loading config");
    let origin = config.origin.clone();

    setup_database(&config).await?;
    // Configure TCP listener and bind
    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));

    let (tx, rx): (Sender<DriverLocationEvent>, Receiver<DriverLocationEvent>) =
        mpsc::channel(100000);
    let (r_tx, r_rx): (
        Sender<OnGoingRideCoordinatesData>,
        Receiver<OnGoingRideCoordinatesData>,
    ) = mpsc::channel(100000);
    let (stats_tx, stats_rx): (
        Sender<ProcessStataData>,
        Receiver<ProcessStataData>,
    ) = mpsc::channel(1000);

    let api_ctx = APIContext::new(config, tx, r_tx, stats_tx).await?;

    let cors = CorsLayer::new()
        .allow_origin(origin.parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let mut public_router: Router<()> = Router::new()
        .route("/", get(handler))
        .route(
            "/get-profile-image/{ulid}/{path_id}",
            get(docs::get_profile_photo),
        )
        .route("/mpesa/callback", post(payment::mpesa_callback_url))
        .with_state(api_ctx.clone());
    public_router = public_router.merge(auth_handlers(api_ctx.clone()));

    let auth_router: Router<()> = Router::new()
        .merge(handlers(api_ctx.clone()))
        .layer(ServiceBuilder::new().layer(middleware::from_fn_with_state(
            api_ctx.clone(),
            auth_middleware,
        )));

    let app = public_router
        .merge(auth_router)
        .layer(tracing_layer)
        .layer(cors)
        .layer(Extension(api_ctx.clone()));

    let drainner = RideLocationInfoDrainner {
        executor: Executor,
        delay: 5,
        capacity: 4,
        nearby_driver_threshold: 4,
        map_size: 30,
    };

    let process_on_going_ride_coordinates = ProcessOnGoingRideCoordinates {
        executor: Executor,
        batch_size: 20,
    };

    drainner.run(rx, api_ctx.redis.clone()).await;

    // Start the process_on_going_ride_coordinates
    // in a separate task
    process_on_going_ride_coordinates.run(r_rx, api_ctx.db.clone()).await?;
    // Start the process_stats in a separate task
    let process_stats = ProcessStats {
        executor: Executor,
        db: api_ctx.db.clone(),
        sts_rx: stats_rx,
    };
    process_stats.run().await;

    // Start the job to update subscriptions
    let mut scheduler = update_subscriptions(api_ctx.clone())
        .await
        .expect("failed to start job scheduler");

    // Private admin plane (Option B). Opt-in: starts only when an internal
    // token is configured. Bound to a private interface (loopback by default)
    // so it is unreachable from the public internet — only the co-located admin
    // BFF, which carries `X-Internal-Token`, can call it.
    if !api_ctx.config.admin_internal_token.is_empty() {
        let admin_ctx = api_ctx.clone();
        let admin_bind = format!(
            "{}:{}",
            admin_ctx.config.effective_admin_bind_addr(),
            admin_ctx.config.admin_port
        );
        let admin_app =
            api::api::admin::admin_handlers(admin_ctx).into_make_service();
        tokio::spawn(async move {
            match TcpListener::bind(&admin_bind).await {
                Ok(l) => {
                    info!("🔒 Private admin plane on http://{admin_bind}");
                    if let Err(e) = axum::serve(l, admin_app).await {
                        tracing::error!("admin server error: {e}");
                    }
                }
                Err(e) => tracing::error!(
                    "failed to bind admin listener on {admin_bind}: {e}"
                ),
            }
        });
    } else {
        info!("Admin plane disabled (ADMIN_INTERNAL_TOKEN not set)");
    }

    let listener = TcpListener::bind(&address).await.unwrap();
    info!("🚀 Server running at http://localhost:{}", address.port());
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    scheduler.shutdown().await.ok();
    Ok(())
}

pub(crate) async fn update_subscriptions(
    ctx: Arc<APIContext>,
) -> utils::Result<tokio_cron_scheduler::JobScheduler, JobSchedulerError> {
    let scheduler = tokio_cron_scheduler::JobScheduler::new().await?;

    let ctx_clone = ctx.clone();
    let job = tokio_cron_scheduler::JobBuilder::new()
        .with_timezone(chrono_tz::Africa::Nairobi)
        .with_cron_job_type()
        .with_schedule("0 0 0 * * *")
        .unwrap()
        .with_run_async(Box::new(move |uuid, mut l| {
            let ctx = ctx_clone.clone();
            Box::pin(async move {
                info!("Running job to update subscriptions for JHB every day at midnight {:?}", uuid);

                let _ = ctx.db.update_due_amount().await.map_err(|_| {
                    JobSchedulerError::NotifyOnStateError
                });
                let next_tick = l.next_tick_for_job(uuid).await;
                match next_tick {
                    Ok(Some(ts)) => info!("Next time for JHB is {:?}", ts),
                    _ => tracing::warn!("Could not get next tick for job"),
                }
            })
        }))
        .build()
        .unwrap();

    scheduler.add(job).await?;
    scheduler.start().await?;

    Ok(scheduler)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    info!("Received Ctrl+C, shutting down...");
}

pub(crate) async fn setup_database(
    config: &config::config::Config,
) -> Result<()> {
    let conn_options = ConnectOptions::new(config.database_url.clone());
    let db = Database::new(conn_options, Executor).await?;
    let migrations_path =
        config.migrations_path.as_deref().unwrap_or_else(|| {
            let default_migrations =
                concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");
            Path::new(default_migrations)
        });

    let migrations = run_db_migrations(migrations_path, db.options()).await?;
    for (migration, duration) in migrations {
        info!(
            "Migrated {} {} {:?}",
            migration.version, migration.description, duration
        );
    }
    drop(db);

    Ok(())
}

/// Capture crate version from Cargo
static CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn handler() -> Json<RootResponse> {
    Json(RootResponse {
        server: "👋😋",
        version: CRATE_VERSION,
    })
}

/// Successful root response
#[derive(serde::Serialize, Debug)]
pub struct RootResponse {
    server: &'static str,
    version: &'static str,
}
