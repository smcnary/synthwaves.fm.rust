use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};

use crate::{app_state::AppState, auth};

pub fn internal_router() -> Router<AppState> {
    Router::new()
        .route("/radio_stations/active", get(active))
        .route("/radio_stations/{id}/next_track", get(next_track))
        .route("/radio_stations/{id}/notify", post(notify))
}

pub async fn active(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    Json(serde_json::json!({ "stations": [] })).into_response()
}

pub async fn next_track(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    Json(serde_json::json!({
        "url": format!("https://example.invalid/tracks/{id}.mp3")
    }))
    .into_response()
}

pub async fn notify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(_id): Path<i64>,
) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}
