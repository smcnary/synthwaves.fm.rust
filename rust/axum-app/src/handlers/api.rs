use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use infra::auth::issue_jwt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::AppState, auth};

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub client_id: String,
    pub secret_key: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub fn v1_router() -> Router<AppState> {
    Router::new()
        .route("/auth/token", post(create_token))
        .route("/native/credentials", get(native_credentials))
        .route("/me", get(me))
}

pub async fn create_token(
    State(state): State<AppState>,
    Json(req): Json<TokenRequest>,
) -> impl IntoResponse {
    if req.client_id.is_empty() || req.secret_key.is_empty() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
            }),
        )
            .into_response();
    }

    let token = issue_jwt(Uuid::new_v4(), Uuid::new_v4(), &state.config.jwt_secret)
        .unwrap_or_else(|_| "invalid".to_string());
    (
        StatusCode::OK,
        Json(TokenResponse {
            token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
        }),
    )
        .into_response()
}

pub async fn native_credentials() -> impl IntoResponse {
    Json(serde_json::json!({
        "supports_subsonic": true,
        "supports_jwt": true
    }))
}

pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if auth::require_jwt(&headers, &state).is_err() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
            }),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "service": "axum"})),
    )
        .into_response()
}

pub fn subsonic_router() -> Router<AppState> {
    Router::new().route("/ping", get(subsonic_ping))
}

pub async fn subsonic_ping(
    State(state): State<AppState>,
    query: Query<crate::auth::SubsonicAuthQuery>,
) -> impl IntoResponse {
    if crate::auth::require_subsonic(query, State(state)).await.is_err() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"subsonic-response": {"status": "failed", "error": {"code": 40, "message": "Wrong username or password"}}})),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({"subsonic-response": {"status": "ok", "version": "1.16.1"}})),
    )
        .into_response()
}
