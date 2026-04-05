use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, auth};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/youtube_sources", get(list_sources).post(create_source))
        .route("/youtube_sources/{id}", patch(update_source))
        .route("/youtube_sources/{id}/run", post(run_source))
        .route("/youtube_sources/{id}/runs", get(list_runs))
}

#[derive(Debug, Deserialize)]
pub struct CreateSourceRequest {
    pub name: String,
    pub playlist_url: String,
    pub target_playlist_name: String,
    pub enabled: Option<bool>,
    pub sync_interval_minutes: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSourceRequest {
    pub name: Option<String>,
    pub target_playlist_name: Option<String>,
    pub enabled: Option<bool>,
    pub sync_interval_minutes: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RunsQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct SourceResponse {
    pub id: i64,
    pub name: String,
    pub playlist_url: String,
    pub playlist_id: String,
    pub target_playlist_name: String,
    pub target_playlist_id: Option<i64>,
    pub enabled: bool,
    pub sync_interval_minutes: i64,
    pub last_synced_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub id: i64,
    pub source_id: i64,
    pub triggered_by: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub imported_count: i64,
    pub skipped_count: i64,
    pub failed_count: i64,
    pub last_error: Option<String>,
}

pub async fn list_sources(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if auth::require_admin_jwt(&headers, &state).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match infra::youtube_import::list_sources(&state.pool).await {
        Ok(sources) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "sources": sources.into_iter().map(map_source).collect::<Vec<_>>()
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": err.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_source(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateSourceRequest>,
) -> impl IntoResponse {
    let claims = match auth::require_admin_jwt(&headers, &state).await {
        Ok(claims) => claims,
        Err(code) => return code.into_response(),
    };
    if payload.name.trim().is_empty()
        || payload.playlist_url.trim().is_empty()
        || payload.target_playlist_name.trim().is_empty()
    {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": "name, playlist_url, and target_playlist_name are required"})),
        )
            .into_response();
    }
    let input = infra::youtube_import::CreateYoutubeSourceInput {
        name: payload.name.trim().to_string(),
        playlist_url: payload.playlist_url.trim().to_string(),
        target_playlist_name: payload.target_playlist_name.trim().to_string(),
        enabled: payload.enabled.unwrap_or(true),
        sync_interval_minutes: payload
            .sync_interval_minutes
            .unwrap_or(state.config.youtube_import_default_sync_interval_minutes)
            .max(1),
        created_by_user_id: Some(claims.user_id.to_string()),
    };
    match infra::youtube_import::create_source(&state.pool, input).await {
        Ok(source) => (StatusCode::CREATED, Json(map_source(source))).into_response(),
        Err(err) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": err.to_string()})),
        )
            .into_response(),
    }
}

pub async fn update_source(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateSourceRequest>,
) -> impl IntoResponse {
    if auth::require_admin_jwt(&headers, &state).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let input = infra::youtube_import::UpdateYoutubeSourceInput {
        name: payload.name.map(|v| v.trim().to_string()),
        target_playlist_name: payload.target_playlist_name.map(|v| v.trim().to_string()),
        enabled: payload.enabled,
        sync_interval_minutes: payload.sync_interval_minutes,
    };
    match infra::youtube_import::update_source(&state.pool, id, input).await {
        Ok(source) => (StatusCode::OK, Json(map_source(source))).into_response(),
        Err(err) if err.to_string().contains("not found") => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "source not found"})),
        )
            .into_response(),
        Err(err) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": err.to_string()})),
        )
            .into_response(),
    }
}

pub async fn run_source(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if auth::require_admin_jwt(&headers, &state).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !state.config.youtube_import_enabled {
        return (
            StatusCode::FAILED_DEPENDENCY,
            Json(serde_json::json!({"error": "youtube imports are disabled by configuration"})),
        )
            .into_response();
    }
    match infra::youtube_import::run_source_import(&state.pool, &state.config, id, "manual").await {
        Ok(result) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "run_id": result.run_id,
                "source_id": result.source_id,
                "status": result.status,
                "imported_count": result.imported_count,
                "skipped_count": result.skipped_count,
                "failed_count": result.failed_count,
                "last_error": result.last_error,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": err.to_string()})),
        )
            .into_response(),
    }
}

pub async fn list_runs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(query): Query<RunsQuery>,
) -> impl IntoResponse {
    if auth::require_admin_jwt(&headers, &state).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let limit = query.limit.unwrap_or(20).clamp(1, 200);
    match infra::youtube_import::list_runs(&state.pool, id, limit).await {
        Ok(runs) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "runs": runs.into_iter().map(map_run).collect::<Vec<_>>()
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": err.to_string()})),
        )
            .into_response(),
    }
}

fn map_source(source: infra::youtube_import::YoutubePlaylistSource) -> SourceResponse {
    SourceResponse {
        id: source.id,
        name: source.name,
        playlist_url: source.playlist_url,
        playlist_id: source.playlist_id,
        target_playlist_name: source.target_playlist_name,
        target_playlist_id: source.target_playlist_id,
        enabled: source.enabled,
        sync_interval_minutes: source.sync_interval_minutes,
        last_synced_at: source.last_synced_at,
        last_error: source.last_error,
    }
}

fn map_run(run: infra::youtube_import::YoutubeImportRun) -> RunResponse {
    RunResponse {
        id: run.id,
        source_id: run.source_id,
        triggered_by: run.triggered_by,
        status: run.status,
        started_at: run.started_at,
        finished_at: run.finished_at,
        imported_count: run.imported_count,
        skipped_count: run.skipped_count,
        failed_count: run.failed_count,
        last_error: run.last_error,
    }
}
