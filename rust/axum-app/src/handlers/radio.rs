use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{app_state::AppState, auth};

pub fn internal_router() -> Router<AppState> {
    Router::new()
        .route("/liquidsoap_config", get(liquidsoap_config))
        .route("/radio_stations/active", get(active))
        .route("/radio_stations/{id}/import_youtube", post(import_youtube))
        .route("/radio_stations/{id}/next_track", get(next_track))
        .route("/radio_stations/{id}/up_next", get(up_next))
        .route("/radio_stations/{id}/recent", get(recent))
        .route("/radio_stations/{id}/stats", get(stats))
        .route("/radio_stations/{id}/notify", post(notify))
}

#[derive(Debug, Deserialize)]
pub struct ImportYoutubeRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ImportYoutubeResponse {
    pub station_id: i64,
    pub playlist_id: String,
    pub imported: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct UpNextItem {
    pub position: i64,
    pub title: String,
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct RecentTrackItem {
    pub title: String,
    pub source: String,
    pub duration_seconds: Option<i64>,
    pub played_at: String,
}

#[derive(Debug, Serialize)]
pub struct RadioStatsResponse {
    pub listeners: i64,
    pub bitrate_kbps: i64,
}

#[derive(Debug, Serialize)]
pub struct ActiveStationRow {
    pub id: i64,
    pub mount_point: String,
    pub playlist_name: String,
    pub bitrate: i32,
    pub stream_url: String,
}

#[derive(Debug, Deserialize)]
pub struct NotifyPayload {
    pub event: Option<String>,
    pub title: Option<String>,
    pub source: Option<String>,
    pub duration_seconds: Option<i64>,
    pub video_id: Option<String>,
}

pub async fn active(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match load_radio_stations_for_liquidsoap(&state.pool).await {
        Ok(stations) => {
            let public_base = state.config.icecast_public_base();
            let rows: Vec<ActiveStationRow> = stations
                .into_iter()
                .map(|s| ActiveStationRow {
                    stream_url: infra::icecast::icecast_stream_url(&public_base, &s.mount_point),
                    id: s.id,
                    mount_point: s.mount_point,
                    playlist_name: s.playlist_name,
                    bitrate: s.bitrate,
                })
                .collect();
            Json(serde_json::json!({ "stations": rows })).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn liquidsoap_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let stations = match load_radio_stations_for_liquidsoap(&state.pool).await {
        Ok(s) => s,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let body = infra::liquidsoap::generate_config(
        &stations,
        &state.config.rails_host,
        &state.config.rails_protocol,
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(body))
        .unwrap()
}

pub async fn next_track(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if ensure_import_tables(&state).await.is_err() || ensure_detail_tables(&state).await.is_err() {
        return Json(serde_json::json!({
            "url": format!("{}://{}/demo/stream", state.config.rails_protocol, state.config.rails_host),
            "reason": "import table init failed"
        }))
        .into_response();
    }

    if let Ok(Some(item)) = next_import_item(&state, id).await {
        let _ = upsert_runtime_state(
            &state,
            id,
            Some(item.video_id.as_str()),
            &item.title,
            "YouTube",
            None,
            true,
        )
        .await;
        let stream_url = tokio::task::spawn_blocking(move || {
            infra::youtube::resolve_audio_stream_url(&item.video_id)
        })
        .await
        .ok()
        .and_then(Result::ok);

        let url = stream_url.unwrap_or_else(|| {
            format!(
                "{}://{}/demo/stream",
                state.config.rails_protocol, state.config.rails_host
            )
        });
        return Json(serde_json::json!({ "url": url, "title": item.title })).into_response();
    }

    let track_id = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT pt.track_id
        FROM radio_stations rs
        JOIN playlist_tracks pt ON pt.playlist_id = rs.playlist_id
        WHERE rs.id = ?
        ORDER BY pt.position ASC
        LIMIT 1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    let url = match track_id {
        Some(track_id) => format!(
            "{}://{}/tracks/{track_id}/stream",
            state.config.rails_protocol, state.config.rails_host
        ),
        None => format!(
            "{}://{}/demo/stream",
            state.config.rails_protocol, state.config.rails_host
        ),
    };

    Json(serde_json::json!({ "url": url })).into_response()
}

pub async fn up_next(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if ensure_import_tables(&state).await.is_err() || ensure_detail_tables(&state).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "failed to initialize radio detail tables"})),
        )
            .into_response();
    }
    match get_up_next_items(&state, id, 8).await {
        Ok(items) => Json(serde_json::json!({ "items": items })).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "failed to query up-next items"})),
        )
            .into_response(),
    }
}

pub async fn recent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if ensure_detail_tables(&state).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "failed to initialize recent-track tables"})),
        )
            .into_response();
    }
    let rows = sqlx::query(
        r#"
        SELECT title, source, duration_seconds, played_at
        FROM radio_station_recent_tracks
        WHERE station_id = ?
        ORDER BY datetime(played_at) DESC
        LIMIT 12
        "#,
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await;
    match rows {
        Ok(rows) => {
            let items: Vec<RecentTrackItem> = rows
                .into_iter()
                .map(|row| RecentTrackItem {
                    title: row.get::<String, _>("title"),
                    source: row
                        .try_get::<String, _>("source")
                        .unwrap_or_else(|_| "YouTube".to_string()),
                    duration_seconds: row.try_get::<i64, _>("duration_seconds").ok(),
                    played_at: row.get::<String, _>("played_at"),
                })
                .collect();
            Json(serde_json::json!({ "items": items })).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"failed to query recent tracks"})),
        )
            .into_response(),
    }
}

pub async fn stats(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if ensure_detail_tables(&state).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"failed to initialize stats tables"})),
        )
            .into_response();
    }

    let (mount_point, bitrate) = get_station_mount_and_bitrate(&state, id).await;
    let listeners = match refresh_listener_stats(&state, id, &mount_point).await {
        Ok(value) => value,
        Err(_) => read_listener_stats(&state, id).await.unwrap_or(0),
    };
    (
        StatusCode::OK,
        Json(RadioStatsResponse {
            listeners,
            bitrate_kbps: bitrate,
        }),
    )
        .into_response()
}

pub async fn import_youtube(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<ImportYoutubeRequest>,
) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let Some(playlist_id) = infra::youtube::extract_playlist_id(&payload.url) else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": "playlist URL must include list=..."})),
        )
            .into_response();
    };

    let playlist_url = payload.url.clone();
    let entries_result =
        tokio::task::spawn_blocking(move || infra::youtube::fetch_playlist_entries(&playlist_url))
            .await;

    let entries = match entries_result {
        Ok(Ok(entries)) => entries,
        Ok(Err(err)) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": format!("yt-dlp playlist import failed: {err}")})),
            )
                .into_response();
        }
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": format!("yt-dlp worker join failed: {err}")})),
            )
                .into_response();
        }
    };

    if entries.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error": "playlist contains no videos"})),
        )
            .into_response();
    }

    if ensure_import_tables(&state).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "failed to initialize import tables"})),
        )
            .into_response();
    }

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "failed to open transaction"})),
            )
                .into_response();
        }
    };

    if sqlx::query("DELETE FROM radio_station_import_items WHERE station_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "failed to clear previous import items"})),
        )
            .into_response();
    }

    for (idx, item) in entries.iter().enumerate() {
        if sqlx::query(
            r#"
            INSERT INTO radio_station_import_items (station_id, position, video_id, title)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind((idx + 1) as i64)
        .bind(&item.video_id)
        .bind(&item.title)
        .execute(&mut *tx)
        .await
        .is_err()
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "failed to save imported items"})),
            )
                .into_response();
        }
    }

    let _ = sqlx::query(
        r#"
        INSERT INTO radio_station_import_state (station_id, next_position)
        VALUES (?, 1)
        ON CONFLICT(station_id) DO UPDATE SET next_position = 1, updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(id)
    .execute(&mut *tx)
    .await;

    let _ = sqlx::query("UPDATE radio_stations SET youtube_url = ? WHERE id = ?")
        .bind(&payload.url)
        .bind(id)
        .execute(&mut *tx)
        .await;

    if tx.commit().await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "failed to commit import"})),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(ImportYoutubeResponse {
            station_id: id,
            playlist_id,
            imported: entries.len(),
        }),
    )
        .into_response()
}

pub async fn notify(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    payload: Option<Json<NotifyPayload>>,
) -> impl IntoResponse {
    if auth::require_internal_token(&headers, &state).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if ensure_detail_tables(&state).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let payload = payload.map(|j| j.0).unwrap_or(NotifyPayload {
        event: None,
        title: None,
        source: None,
        duration_seconds: None,
        video_id: None,
    });

    match payload.event.as_deref() {
        Some("idle") => {
            let _ = upsert_runtime_state(&state, id, None, "Idle", "System", None, false).await;
        }
        Some("track_started") | Some("started") => {
            let title = payload
                .title
                .clone()
                .unwrap_or_else(|| "Untitled".to_string());
            let source = payload
                .source
                .clone()
                .unwrap_or_else(|| "YouTube".to_string());
            let _ = upsert_runtime_state(
                &state,
                id,
                payload.video_id.as_deref(),
                &title,
                &source,
                payload.duration_seconds,
                true,
            )
            .await;
            let _ =
                append_recent_track(&state, id, &title, &source, payload.duration_seconds).await;
        }
        _ => {}
    }
    StatusCode::NO_CONTENT.into_response()
}

#[derive(Debug, Clone)]
struct ImportedQueueItem {
    video_id: String,
    title: String,
}

async fn load_radio_stations_for_liquidsoap(
    pool: &sqlx::SqlitePool,
) -> Result<Vec<domain::models::RadioStation>, sqlx::Error> {
    let table_ok: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'radio_stations'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    if table_ok == 0 {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT
            rs.id,
            p.name AS playlist_name,
            rs.mount_point,
            rs.bitrate,
            rs.crossfade,
            rs.crossfade_duration
        FROM radio_stations rs
        JOIN playlists p ON p.id = rs.playlist_id
        ORDER BY rs.id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| domain::models::RadioStation {
            id: row.get("id"),
            playlist_name: row.get::<String, _>("playlist_name"),
            mount_point: row.get::<String, _>("mount_point"),
            bitrate: row.get::<i64, _>("bitrate") as i32,
            crossfade: row.get::<i64, _>("crossfade") != 0,
            crossfade_duration: row.get::<i64, _>("crossfade_duration") as i32,
        })
        .collect())
}

async fn ensure_import_tables(state: &AppState) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS radio_station_import_items (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          station_id INTEGER NOT NULL,
          position INTEGER NOT NULL,
          video_id TEXT NOT NULL,
          title TEXT,
          created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
          UNIQUE(station_id, position)
        )
        "#,
    )
    .execute(&state.pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS radio_station_import_state (
          station_id INTEGER PRIMARY KEY,
          next_position INTEGER NOT NULL DEFAULT 1,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&state.pool)
    .await?;

    Ok(())
}

async fn ensure_detail_tables(state: &AppState) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS radio_station_runtime_state (
          station_id INTEGER PRIMARY KEY,
          current_video_id TEXT,
          current_title TEXT,
          current_source TEXT DEFAULT 'YouTube',
          current_duration_seconds INTEGER,
          last_track_started_at DATETIME,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&state.pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS radio_station_recent_tracks (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          station_id INTEGER NOT NULL,
          title TEXT NOT NULL,
          source TEXT NOT NULL DEFAULT 'YouTube',
          duration_seconds INTEGER,
          played_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&state.pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS radio_station_listener_stats (
          station_id INTEGER PRIMARY KEY,
          listener_count INTEGER NOT NULL DEFAULT 0,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&state.pool)
    .await?;
    Ok(())
}

async fn next_import_item(
    state: &AppState,
    station_id: i64,
) -> Result<Option<ImportedQueueItem>, sqlx::Error> {
    let mut tx = state.pool.begin().await?;

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM radio_station_import_items WHERE station_id = ?",
    )
    .bind(station_id)
    .fetch_one(&mut *tx)
    .await?;

    if count <= 0 {
        tx.commit().await?;
        return Ok(None);
    }

    let next_pos = sqlx::query_scalar::<_, i64>(
        "SELECT next_position FROM radio_station_import_state WHERE station_id = ?",
    )
    .bind(station_id)
    .fetch_optional(&mut *tx)
    .await?
    .unwrap_or(1);

    let normalized_pos = next_pos.clamp(1, count);
    let row = sqlx::query(
        r#"
        SELECT video_id, COALESCE(title, 'Untitled') AS title
        FROM radio_station_import_items
        WHERE station_id = ? AND position = ?
        LIMIT 1
        "#,
    )
    .bind(station_id)
    .bind(normalized_pos)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.commit().await?;
        return Ok(None);
    };

    let item = ImportedQueueItem {
        video_id: row.get("video_id"),
        title: row.get("title"),
    };
    let following = if normalized_pos >= count {
        1
    } else {
        normalized_pos + 1
    };

    sqlx::query(
        r#"
        INSERT INTO radio_station_import_state (station_id, next_position)
        VALUES (?, ?)
        ON CONFLICT(station_id) DO UPDATE SET next_position = excluded.next_position, updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(station_id)
    .bind(following)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Some(item))
}

async fn get_up_next_items(
    state: &AppState,
    station_id: i64,
    limit: i64,
) -> Result<Vec<UpNextItem>, sqlx::Error> {
    let next_pos = sqlx::query_scalar::<_, i64>(
        "SELECT next_position FROM radio_station_import_state WHERE station_id = ?",
    )
    .bind(station_id)
    .fetch_optional(&state.pool)
    .await?
    .unwrap_or(1);

    let rows = sqlx::query(
        r#"
        SELECT position, COALESCE(title, 'Untitled') AS title
        FROM radio_station_import_items
        WHERE station_id = ?
        ORDER BY position
        "#,
    )
    .bind(station_id)
    .fetch_all(&state.pool)
    .await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let mut ordered = Vec::with_capacity(rows.len());
    for row in rows {
        ordered.push(UpNextItem {
            position: row.get::<i64, _>("position"),
            title: row.get::<String, _>("title"),
            source: "YouTube".to_string(),
        });
    }
    ordered.sort_by_key(|i| i.position);
    let start_idx = ordered
        .iter()
        .position(|i| i.position == next_pos)
        .unwrap_or(0);

    let mut out = Vec::new();
    for i in 0..ordered.len().min(limit as usize) {
        let idx = (start_idx + i) % ordered.len();
        out.push(ordered[idx].clone());
    }
    Ok(out)
}

async fn upsert_runtime_state(
    state: &AppState,
    station_id: i64,
    video_id: Option<&str>,
    title: &str,
    source: &str,
    duration_seconds: Option<i64>,
    started: bool,
) -> Result<(), sqlx::Error> {
    let started_at_expr = if started {
        "CURRENT_TIMESTAMP"
    } else {
        "last_track_started_at"
    };
    let sql = format!(
        r#"
        INSERT INTO radio_station_runtime_state (
          station_id, current_video_id, current_title, current_source, current_duration_seconds, last_track_started_at
        )
        VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(station_id) DO UPDATE SET
          current_video_id = excluded.current_video_id,
          current_title = excluded.current_title,
          current_source = excluded.current_source,
          current_duration_seconds = excluded.current_duration_seconds,
          last_track_started_at = {started_at_expr},
          updated_at = CURRENT_TIMESTAMP
        "#
    );
    sqlx::query(&sql)
        .bind(station_id)
        .bind(video_id.unwrap_or_default())
        .bind(title)
        .bind(source)
        .bind(duration_seconds)
        .execute(&state.pool)
        .await?;
    Ok(())
}

async fn append_recent_track(
    state: &AppState,
    station_id: i64,
    title: &str,
    source: &str,
    duration_seconds: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO radio_station_recent_tracks (station_id, title, source, duration_seconds)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(station_id)
    .bind(title)
    .bind(source)
    .bind(duration_seconds)
    .execute(&state.pool)
    .await?;
    let _ = sqlx::query(
        r#"
        DELETE FROM radio_station_recent_tracks
        WHERE station_id = ?
          AND id NOT IN (
            SELECT id FROM radio_station_recent_tracks
            WHERE station_id = ?
            ORDER BY datetime(played_at) DESC
            LIMIT 25
          )
        "#,
    )
    .bind(station_id)
    .bind(station_id)
    .execute(&state.pool)
    .await;
    Ok(())
}

async fn get_station_mount_and_bitrate(state: &AppState, station_id: i64) -> (String, i64) {
    let rows = sqlx::query("PRAGMA table_info('radio_stations')")
        .fetch_all(&state.pool)
        .await
        .unwrap_or_default();
    let mut has_mount = false;
    let mut has_bitrate = false;
    for row in rows {
        let name: String = row.try_get("name").unwrap_or_default();
        if name == "mount_point" {
            has_mount = true;
        } else if name == "bitrate" {
            has_bitrate = true;
        }
    }
    let mount_expr = if has_mount {
        "COALESCE(mount_point, '/radio/' || id || '.mp3')"
    } else {
        "'/radio/' || id || '.mp3'"
    };
    let bitrate_expr = if has_bitrate {
        "COALESCE(bitrate, 192)"
    } else {
        "192"
    };
    let sql = format!(
        "SELECT {mount_expr} AS station_mount, {bitrate_expr} AS station_bitrate FROM radio_stations WHERE id = ? LIMIT 1"
    );
    let row = sqlx::query(&sql)
        .bind(station_id)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten();
    if let Some(row) = row {
        let mount = row
            .try_get::<String, _>("station_mount")
            .unwrap_or_else(|_| format!("/radio/{station_id}.mp3"));
        let bitrate = row.try_get::<i64, _>("station_bitrate").unwrap_or(192);
        (mount, bitrate)
    } else {
        (format!("/radio/{station_id}.mp3"), 192)
    }
}

async fn read_listener_stats(state: &AppState, station_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT listener_count FROM radio_station_listener_stats WHERE station_id = ?",
    )
    .bind(station_id)
    .fetch_optional(&state.pool)
    .await
    .map(|v| v.unwrap_or(0))
}

async fn refresh_listener_stats(
    state: &AppState,
    station_id: i64,
    mount_point: &str,
) -> anyhow::Result<i64> {
    let cfg = infra::icecast::IcecastConfig {
        protocol: state.config.icecast_protocol.clone(),
        host: state.config.icecast_host.clone(),
        port: state.config.icecast_port,
        admin_username: state.config.icecast_admin_username.clone(),
        admin_password: state.config.icecast_admin_password.clone(),
    };
    let listeners = infra::icecast::fetch_listener_count(&cfg, mount_point).await?;
    let _ = sqlx::query(
        r#"
        INSERT INTO radio_station_listener_stats (station_id, listener_count)
        VALUES (?, ?)
        ON CONFLICT(station_id) DO UPDATE SET listener_count = excluded.listener_count, updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(station_id)
    .bind(listeners)
    .execute(&state.pool)
    .await;
    Ok(listeners)
}
