use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use infra::auth::{hash_password, issue_jwt, verify_password};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::{app_state::AppState, auth};

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub email: Option<String>,
    pub password: Option<String>,
    // Backward compatibility for old clients using api-key style names.
    pub client_id: Option<String>,
    pub secret_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
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
        .route("/auth/register", post(create_register))
        .route("/native/credentials", get(native_credentials))
        .route("/me", get(me))
        .nest("/admin", super::youtube_import::router())
}

fn issue_token_response(user_id: Uuid, jwt_secret: &str) -> Result<TokenResponse, StatusCode> {
    let token = issue_jwt(user_id, Uuid::nil(), jwt_secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(TokenResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    })
}

pub async fn create_token(
    State(state): State<AppState>,
    Json(req): Json<TokenRequest>,
) -> impl IntoResponse {
    let email = req
        .email
        .as_deref()
        .or(req.client_id.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let password = req
        .password
        .as_deref()
        .or(req.secret_key.as_deref())
        .filter(|s| !s.is_empty());

    let (Some(email), Some(password)) = (email, password) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
            }),
        )
            .into_response();
    };

    let row = match sqlx::query(
        r#"
        SELECT id, password_hash
        FROM users
        WHERE lower(email_address) = lower(?)
        LIMIT 1
        "#,
    )
    .bind(email)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(row) => row,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal Server Error".to_string(),
                }),
            )
                .into_response();
        }
    };
    let Some(row) = row else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
            }),
        )
            .into_response();
    };

    let password_hash = row.try_get::<String, _>("password_hash").ok();
    let valid = password_hash
        .as_deref()
        .map(|hash| verify_password(password, hash))
        .unwrap_or(false);
    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Unauthorized".to_string(),
            }),
        )
            .into_response();
    }

    let user_id_raw: String = match row.try_get("id") {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal Server Error".to_string(),
                }),
            )
                .into_response();
        }
    };
    let user_id = match Uuid::parse_str(&user_id_raw) {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal Server Error".to_string(),
                }),
            )
                .into_response();
        }
    };
    let token_response = match issue_token_response(user_id, &state.config.jwt_secret) {
        Ok(payload) => payload,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal Server Error".to_string(),
                }),
            )
                .into_response();
        }
    };
    (
        StatusCode::OK,
        Json(token_response),
    )
        .into_response()
}

pub async fn create_register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') || req.password.trim().len() < 8 {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(ErrorResponse {
                error: "Email and password are required (password min 8 chars)".to_string(),
            }),
        )
            .into_response();
    }

    let existing_count = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE lower(email_address) = lower(?)",
    )
    .bind(&email)
    .fetch_one(&state.pool)
    .await
    {
        Ok(count) => count,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal Server Error".to_string(),
                }),
            )
                .into_response();
        }
    };
    if existing_count > 0 {
        return (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Email is already registered".to_string(),
            }),
        )
            .into_response();
    }

    let password_hash = match hash_password(req.password.trim()) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal Server Error".to_string(),
                }),
            )
                .into_response();
        }
    };
    let user_id = Uuid::new_v4();
    let inserted = sqlx::query(
        r#"
        INSERT INTO users (id, email_address, password_hash, admin, theme)
        VALUES (?, ?, ?, 0, 'synthwave')
        "#,
    )
    .bind(user_id.to_string())
    .bind(&email)
    .bind(password_hash)
    .execute(&state.pool)
    .await;
    if inserted.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal Server Error".to_string(),
            }),
        )
            .into_response();
    }

    let token_response = match issue_token_response(user_id, &state.config.jwt_secret) {
        Ok(payload) => payload,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal Server Error".to_string(),
                }),
            )
                .into_response();
        }
    };
    (StatusCode::CREATED, Json(token_response)).into_response()
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
    if crate::auth::require_subsonic(query, State(state))
        .await
        .is_err()
    {
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

#[cfg(test)]
mod tests {
    use super::{RegisterRequest, TokenRequest, create_register, create_token};
    use crate::app_state::AppState;
    use axum::{Json, body::to_bytes, extract::State, http::StatusCode, response::IntoResponse};
    use infra::{auth::hash_password, config::AppConfig};
    use sqlx::SqlitePool;
    use uuid::Uuid;

    fn test_config() -> AppConfig {
        AppConfig {
            host: "127.0.0.1".to_string(),
            port: 4000,
            database_url: "sqlite::memory:".to_string(),
            jwt_secret: "test-secret".to_string(),
            liquidsoap_api_token: "token".to_string(),
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
            bootstrap_admin_email: None,
            bootstrap_admin_password: None,
        }
    }

    async fn setup_pool() -> anyhow::Result<SqlitePool> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        sqlx::query(
            r#"
            CREATE TABLE users (
              id TEXT PRIMARY KEY,
              email_address TEXT NOT NULL UNIQUE,
              password_hash TEXT,
              subsonic_password TEXT,
              theme TEXT NOT NULL DEFAULT 'synthwave',
              admin INTEGER NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
              updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await?;
        Ok(pool)
    }

    #[tokio::test]
    async fn create_token_returns_unauthorized_for_missing_user() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        let state = AppState {
            config: test_config(),
            pool,
        };
        let resp = create_token(
            State(state),
            Json(TokenRequest {
                email: Some("nobody@example.com".to_string()),
                password: Some("bad".to_string()),
                client_id: None,
                secret_key: None,
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[tokio::test]
    async fn create_token_returns_unauthorized_for_bad_password() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        let user_id = Uuid::new_v4().to_string();
        let password_hash = hash_password("correct-password")?;
        sqlx::query(
            "INSERT INTO users (id, email_address, password_hash, admin) VALUES (?, ?, ?, 0)",
        )
        .bind(&user_id)
        .bind("listener@example.com")
        .bind(password_hash)
        .execute(&pool)
        .await?;
        let state = AppState {
            config: test_config(),
            pool,
        };
        let resp = create_token(
            State(state),
            Json(TokenRequest {
                email: Some("listener@example.com".to_string()),
                password: Some("wrong-password".to_string()),
                client_id: None,
                secret_key: None,
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[tokio::test]
    async fn create_token_returns_token_for_valid_credentials() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        let user_id = Uuid::new_v4().to_string();
        let password_hash = hash_password("correct-password")?;
        sqlx::query(
            "INSERT INTO users (id, email_address, password_hash, admin) VALUES (?, ?, ?, 1)",
        )
        .bind(&user_id)
        .bind("admin@example.com")
        .bind(password_hash)
        .execute(&pool)
        .await?;
        let state = AppState {
            config: test_config(),
            pool,
        };
        let resp = create_token(
            State(state),
            Json(TokenRequest {
                email: Some("admin@example.com".to_string()),
                password: Some("correct-password".to_string()),
                client_id: None,
                secret_key: None,
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1024 * 1024).await?;
        let value: serde_json::Value = serde_json::from_slice(&body)?;
        assert_eq!(value["token_type"], "Bearer");
        assert_eq!(value["expires_in"], 3600);
        assert!(value["token"].as_str().unwrap_or_default().len() > 16);
        Ok(())
    }

    #[tokio::test]
    async fn create_register_creates_user_and_returns_token() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        let state = AppState {
            config: test_config(),
            pool: pool.clone(),
        };
        let resp = create_register(
            State(state),
            Json(RegisterRequest {
                email: "new-user@example.com".to_string(),
                password: "strongpassword".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let created_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE lower(email_address)=lower('new-user@example.com')",
        )
        .fetch_one(&pool)
        .await?;
        assert_eq!(created_count, 1);

        let body = to_bytes(resp.into_body(), 1024 * 1024).await?;
        let value: serde_json::Value = serde_json::from_slice(&body)?;
        assert_eq!(value["token_type"], "Bearer");
        assert!(value["token"].as_str().unwrap_or_default().len() > 16);
        Ok(())
    }

    #[tokio::test]
    async fn create_register_rejects_duplicate_email() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        let password_hash = hash_password("password123")?;
        sqlx::query("INSERT INTO users (id, email_address, password_hash, admin) VALUES (?, ?, ?, 0)")
            .bind(Uuid::new_v4().to_string())
            .bind("dupe@example.com")
            .bind(password_hash)
            .execute(&pool)
            .await?;

        let state = AppState {
            config: test_config(),
            pool,
        };
        let resp = create_register(
            State(state),
            Json(RegisterRequest {
                email: "dupe@example.com".to_string(),
                password: "anotherpassword".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        Ok(())
    }

    #[tokio::test]
    async fn create_register_rejects_invalid_payload() -> anyhow::Result<()> {
        let pool = setup_pool().await?;
        let state = AppState {
            config: test_config(),
            pool,
        };
        let resp = create_register(
            State(state),
            Json(RegisterRequest {
                email: "invalid-email".to_string(),
                password: "short".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        Ok(())
    }
}
