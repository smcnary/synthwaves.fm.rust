use crate::app_state::AppState;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
};
use infra::auth::{decode_jwt, decode_subsonic_password, secure_compare, validate_subsonic_token};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubsonicAuthQuery {
    pub u: Option<String>,
    pub t: Option<String>,
    pub s: Option<String>,
    pub p: Option<String>,
}

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(ToString::to_string)
}

pub fn require_internal_token(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    let token = bearer_token(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    if token == state.config.liquidsoap_api_token {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn require_jwt(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    let token = bearer_token(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    decode_jwt(&token, &state.config.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(())
}

pub async fn require_subsonic(
    Query(query): Query<SubsonicAuthQuery>,
    State(_state): State<AppState>,
) -> Result<(), StatusCode> {
    let _username = query.u.ok_or(StatusCode::UNAUTHORIZED)?;
    let stored_password = "demo-subsonic-password";
    let valid = match (query.t.as_deref(), query.s.as_deref(), query.p.as_deref()) {
        (Some(token), Some(salt), _) => validate_subsonic_token(stored_password, salt, token),
        (_, _, Some(password)) => {
            let decoded = decode_subsonic_password(password);
            secure_compare(&decoded, stored_password).unwrap_or(false)
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
