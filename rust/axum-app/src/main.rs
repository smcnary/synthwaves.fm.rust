mod app_state;
mod auth;
mod handlers;

use anyhow::Context;
use app_state::AppState;
use axum::{Router, routing::get};
use handlers::{admin, api, health, radio, web};
use infra::{config::AppConfig, db};
use std::net::SocketAddr;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::info;

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
    });
    let pool = db::connect(&config.database_url).await?;
    let state = AppState { config, pool };

    let app = Router::new()
        .route("/up", get(health::up))
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
