mod app_state;
mod auth;
mod handlers;

use anyhow::Context;
use app_state::AppState;
use axum::{Router, routing::get};
use handlers::{admin, api, health, media, radio, web};
use infra::{config::AppConfig, db};
use jobs::scheduler::RecurringJob;
use std::{net::SocketAddr, path::Path};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

fn sqlite_file_path(database_url: &str) -> Option<String> {
    if let Some(p) = database_url.strip_prefix("sqlite:///") {
        return Some(format!("/{p}"));
    }
    if let Some(p) = database_url.strip_prefix("sqlite:/") {
        return Some(format!("/{p}"));
    }
    if let Some(p) = database_url.strip_prefix("sqlite:") {
        return Some(p.to_string());
    }
    None
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env().unwrap_or(AppConfig {
        host: "127.0.0.1".to_string(),
        port: 4000,
        database_url: "sqlite://storage/development.sqlite3".to_string(),
        jwt_secret: "dev-secret".to_string(),
        liquidsoap_api_token: "dev-liquidsoap-token".to_string(),
        rails_host: "localhost:4000".to_string(),
        rails_protocol: "http".to_string(),
        icecast_protocol: "http".to_string(),
        icecast_host: "localhost".to_string(),
        icecast_port: 8000,
        icecast_admin_username: "admin".to_string(),
        icecast_admin_password: "hackme".to_string(),
        icecast_public_base_url: None,
        youtube_import_enabled: true,
        youtube_import_max_items_per_run: 100,
        youtube_import_download_timeout_seconds: 180,
        youtube_import_default_sync_interval_minutes: 60,
        youtube_import_scheduler_enabled: false,
    });
    info!(
        host = %config.host,
        port = config.port,
        database_url = %config.database_url,
        "loaded application configuration"
    );

    if let Some(file_path) = sqlite_file_path(&config.database_url) {
        let db_path = Path::new(&file_path);
        let parent_dir = db_path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());
        info!(
            sqlite_file_path = %db_path.display(),
            sqlite_parent_dir = %parent_dir,
            "sqlite database configuration detected"
        );
    } else {
        warn!(
            database_url = %config.database_url,
            "non-sqlite database url configured; current build is optimized for sqlite workflows"
        );
    }

    let pool = db::connect(&config.database_url)
        .await
        .context("database connection/bootstrap failed during startup")?;
    sqlx::migrate!("../migrations")
        .run(&pool)
        .await
        .context("database migrations failed during startup")?;
    if config.youtube_import_enabled {
        if let Err(err) = infra::youtube_import::dependency_check() {
            warn!(error = %err, "youtube import dependencies are not healthy");
        }
    }
    info!("database connection established and migrations applied");
    let state = AppState { config, pool };

    if state.config.youtube_import_scheduler_enabled {
        let sync_every = (state.config.youtube_import_default_sync_interval_minutes.max(1) as u64) * 60;
        tokio::spawn(async move {
            jobs::scheduler::run_scheduler(vec![RecurringJob {
                name: "youtube_playlist_sync".to_string(),
                interval_seconds: sync_every,
            }])
            .await;
        });
        info!(
            interval_seconds = sync_every,
            "youtube import scheduler started"
        );
    }

    let app = Router::new()
        .route("/up", get(health::up))
        .route("/tracks/{id}/stream", get(media::track_stream))
        .route("/demo/stream", get(media::demo_stream))
        .merge(web::router())
        .nest("/rest", api::subsonic_router())
        .nest("/api/rest", api::subsonic_router())
        .nest("/api/v1", api::v1_router())
        .nest("/api/internal", radio::internal_router())
        .nest("/admin", admin::router())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    let addr: SocketAddr = format!("{}:{}", state.config.host, state.config.port)
        .parse()
        .context("invalid host/port for server bind")?;
    info!("axum listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
