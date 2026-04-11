use askama::Template;
use axum::{
    Router,
    extract::{Path, State},
    response::Html,
    routing::get,
};
use sqlx::Row;
use std::collections::HashSet;

use crate::app_state::AppState;
use infra::config::AppConfig;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    title: &'a str,
}

#[derive(Template)]
#[template(path = "music.html")]
struct MusicTemplate<'a> {
    heading: &'a str,
}

#[derive(Template)]
#[template(path = "radio_test.html")]
struct RadioTestTemplate<'a> {
    default_token: &'a str,
    default_url: &'a str,
}

#[derive(Template)]
#[template(path = "radio.html")]
struct RadioTemplate<'a> {
    stations: &'a [RadioCard],
}

#[derive(Template)]
#[template(path = "radio_station.html")]
struct RadioStationTemplate<'a> {
    station: &'a RadioStationDetail,
    up_next: &'a [RadioTrackRow],
    recent: &'a [RadioTrackRow],
    internal_token: &'a str,
}

#[derive(Template)]
#[template(path = "placeholder.html")]
struct PlaceholderTemplate<'a> {
    page_title: &'a str,
    heading: &'a str,
    description: &'a str,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate<'a> {
    page_title: &'a str,
}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate<'a> {
    page_title: &'a str,
}

#[derive(Debug, Clone)]
struct RadioCard {
    id: i64,
    name: String,
    mount_point: String,
    stream_url: String,
    status: String,
}

#[derive(Debug, Clone)]
struct RadioStationDetail {
    id: i64,
    name: String,
    mount_point: String,
    stream_url: String,
    status: String,
    listeners: i64,
    bitrate_kbps: i64,
}

#[derive(Debug, Clone)]
struct RadioTrackRow {
    title: String,
    source: String,
    duration_label: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/home", get(home))
        .route("/music", get(music))
        .route("/radio", get(radio))
        .route("/radio/{id}", get(radio_station))
        .route("/radio/test", get(radio_test))
        .route("/library", get(library))
        .route("/artists", get(artists))
        .route("/albums", get(albums))
        .route("/tracks", get(tracks))
        .route("/playlists", get(playlists))
        .route("/favorites", get(favorites))
        .route("/my", get(my))
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/stats", get(stats))
        .route("/search", get(search))
}

pub async fn home(State(_state): State<AppState>) -> Html<String> {
    Html(
        HomeTemplate {
            title: "synthwaves.fm (Rust/Axum)",
        }
        .render()
        .unwrap_or_else(|_| "<h1>synthwaves.fm</h1>".to_string()),
    )
}

pub async fn music() -> Html<String> {
    Html(
        MusicTemplate { heading: "Music" }
            .render()
            .unwrap_or_else(|_| "<h1>Music</h1>".to_string()),
    )
}

pub async fn radio(State(state): State<AppState>) -> Html<String> {
    let stations = load_radio_cards(&state).await.unwrap_or_default();
    Html(
        RadioTemplate {
            stations: &stations,
        }
        .render()
        .unwrap_or_else(|_| "<h1>Radio</h1>".to_string()),
    )
}

pub async fn radio_test(State(state): State<AppState>) -> Html<String> {
    Html(
        RadioTestTemplate {
            default_token: &state.config.liquidsoap_api_token,
            default_url: "https://www.youtube.com/watch?v=6aouLxiL4Cw&list=PLfAwSvgqO_M_aT7SOI4jdCCpJbZvDvOT-",
        }
        .render()
        .unwrap_or_else(|_| "<h1>Radio Test</h1>".to_string()),
    )
}

pub async fn radio_station(State(state): State<AppState>, Path(id): Path<i64>) -> Html<String> {
    let default_mount = format!("/radio/{id}.mp3");
    let station = load_station_detail(&state, id)
        .await
        .unwrap_or(RadioStationDetail {
            id,
            name: format!("Station #{id}"),
            mount_point: default_mount.clone(),
            stream_url: station_stream_url(&state.config, &default_mount),
            status: "live".to_string(),
            listeners: 0,
            bitrate_kbps: 192,
        });
    let up_next = load_up_next_rows(&state, id).await.unwrap_or_default();
    let recent = load_recent_rows(&state, id).await.unwrap_or_default();
    Html(
        RadioStationTemplate {
            station: &station,
            up_next: &up_next,
            recent: &recent,
            internal_token: &state.config.liquidsoap_api_token,
        }
        .render()
        .unwrap_or_else(|_| "<h1>Radio Station</h1>".to_string()),
    )
}

pub async fn library() -> Html<String> {
    Html("<h1>Library</h1>".to_string())
}

pub async fn artists() -> Html<String> {
    placeholder_page(
        "Artists",
        "Artists",
        "Artist browsing and management UI is planned. This is a placeholder route.",
    )
}

pub async fn albums() -> Html<String> {
    placeholder_page(
        "Albums",
        "Albums",
        "Album browsing and moderation UI is planned. This is a placeholder route.",
    )
}

pub async fn tracks() -> Html<String> {
    placeholder_page(
        "Tracks",
        "Tracks",
        "Track management and deep metadata tools are planned. This is a placeholder route.",
    )
}

pub async fn playlists() -> Html<String> {
    placeholder_page(
        "Playlists",
        "Playlists",
        "Playlist curation pages are planned. This is a placeholder route.",
    )
}

pub async fn favorites() -> Html<String> {
    placeholder_page(
        "Favorites",
        "Favorites",
        "Favorites and likes pages are planned. This is a placeholder route.",
    )
}

pub async fn my() -> Html<String> {
    placeholder_page(
        "My",
        "My",
        "User-personalized pages are planned. This is a placeholder route.",
    )
}

pub async fn login() -> Html<String> {
    Html(
        LoginTemplate {
            page_title: "Login",
        }
        .render()
        .unwrap_or_else(|_| "<h1>Login</h1>".to_string()),
    )
}

pub async fn register() -> Html<String> {
    Html(
        RegisterTemplate {
            page_title: "Register",
        }
        .render()
        .unwrap_or_else(|_| "<h1>Register</h1>".to_string()),
    )
}

pub async fn stats() -> Html<String> {
    Html("<h1>Stats</h1>".to_string())
}

pub async fn search() -> Html<String> {
    Html("<h1>Search</h1>".to_string())
}

fn placeholder_page(title: &str, heading: &str, description: &str) -> Html<String> {
    Html(
        PlaceholderTemplate {
            page_title: title,
            heading,
            description,
        }
        .render()
        .unwrap_or_else(|_| format!("<h1>{heading}</h1><p>{description}</p>")),
    )
}

fn station_stream_url(config: &AppConfig, mount_point: &str) -> String {
    infra::icecast::icecast_stream_url(&config.icecast_public_base(), mount_point)
}

async fn load_radio_cards(state: &AppState) -> Result<Vec<RadioCard>, sqlx::Error> {
    let table_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'radio_stations'",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    if table_exists == 0 {
        return Ok(Vec::new());
    }

    let column_rows = sqlx::query("PRAGMA table_info('radio_stations')")
        .fetch_all(&state.pool)
        .await?;
    let mut columns: HashSet<String> = HashSet::new();
    for row in column_rows {
        if let Ok(name) = row.try_get::<String, _>("name") {
            columns.insert(name);
        }
    }

    let name_expr = if columns.contains("name") {
        "COALESCE(name, 'Station #' || id)"
    } else {
        "'Station #' || id"
    };
    let mount_expr = if columns.contains("mount_point") {
        "COALESCE(mount_point, '/radio/' || id || '.mp3')"
    } else {
        "'/radio/' || id || '.mp3'"
    };
    let status_expr = if columns.contains("status") {
        "COALESCE(status, 'live')"
    } else {
        "'live'"
    };

    let sql = format!(
        "SELECT id, {name_expr} AS station_name, {mount_expr} AS station_mount, {status_expr} AS station_status FROM radio_stations ORDER BY id DESC"
    );

    let rows = sqlx::query(&sql).fetch_all(&state.pool).await?;
    let mut stations = Vec::with_capacity(rows.len());
    for row in rows {
        let mount_point = row
            .try_get::<String, _>("station_mount")
            .unwrap_or_else(|_| "/radio/unknown.mp3".to_string());
        let stream_url = station_stream_url(&state.config, &mount_point);
        stations.push(RadioCard {
            id: row.try_get::<i64, _>("id").unwrap_or_default(),
            name: row
                .try_get::<String, _>("station_name")
                .unwrap_or_else(|_| "Untitled Station".to_string()),
            mount_point,
            stream_url,
            status: row
                .try_get::<String, _>("station_status")
                .unwrap_or_else(|_| "live".to_string()),
        });
    }
    Ok(stations)
}

async fn load_station_detail(
    state: &AppState,
    station_id: i64,
) -> Result<RadioStationDetail, sqlx::Error> {
    let table_exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'radio_stations'",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);
    if table_exists == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    let column_rows = sqlx::query("PRAGMA table_info('radio_stations')")
        .fetch_all(&state.pool)
        .await?;
    let mut columns: HashSet<String> = HashSet::new();
    for row in column_rows {
        if let Ok(name) = row.try_get::<String, _>("name") {
            columns.insert(name);
        }
    }
    let name_expr = if columns.contains("name") {
        "COALESCE(name, 'Station #' || id)"
    } else {
        "'Station #' || id"
    };
    let mount_expr = if columns.contains("mount_point") {
        "COALESCE(mount_point, '/radio/' || id || '.mp3')"
    } else {
        "'/radio/' || id || '.mp3'"
    };
    let status_expr = if columns.contains("status") {
        "COALESCE(status, 'live')"
    } else {
        "'live'"
    };
    let bitrate_expr = if columns.contains("bitrate") {
        "COALESCE(bitrate, 192)"
    } else {
        "192"
    };
    let sql = format!(
        "SELECT id, {name_expr} AS station_name, {mount_expr} AS station_mount, {status_expr} AS station_status, {bitrate_expr} AS station_bitrate FROM radio_stations WHERE id = ? LIMIT 1"
    );
    let row = sqlx::query(&sql)
        .bind(station_id)
        .fetch_one(&state.pool)
        .await?;
    let listeners = sqlx::query_scalar::<_, i64>(
        "SELECT listener_count FROM radio_station_listener_stats WHERE station_id = ?",
    )
    .bind(station_id)
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None)
    .unwrap_or(0);
    let mount_point = row
        .try_get::<String, _>("station_mount")
        .unwrap_or_else(|_| format!("/radio/{station_id}.mp3"));
    let stream_url = station_stream_url(&state.config, &mount_point);
    Ok(RadioStationDetail {
        id: row.try_get::<i64, _>("id").unwrap_or(station_id),
        name: row
            .try_get::<String, _>("station_name")
            .unwrap_or_else(|_| format!("Station #{station_id}")),
        mount_point,
        stream_url,
        status: row
            .try_get::<String, _>("station_status")
            .unwrap_or_else(|_| "live".to_string()),
        listeners,
        bitrate_kbps: row.try_get::<i64, _>("station_bitrate").unwrap_or(192),
    })
}

async fn load_up_next_rows(
    state: &AppState,
    station_id: i64,
) -> Result<Vec<RadioTrackRow>, sqlx::Error> {
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
    let mut ordered: Vec<(i64, String)> = rows
        .into_iter()
        .map(|row| (row.get::<i64, _>("position"), row.get::<String, _>("title")))
        .collect();
    ordered.sort_by_key(|(position, _)| *position);
    let start = ordered
        .iter()
        .position(|(position, _)| *position == next_pos)
        .unwrap_or(0);
    let mut out = Vec::new();
    for i in 0..ordered.len().min(8) {
        let (_, title) = &ordered[(start + i) % ordered.len()];
        out.push(RadioTrackRow {
            title: title.clone(),
            source: "YouTube".to_string(),
            duration_label: "--".to_string(),
        });
    }
    Ok(out)
}

async fn load_recent_rows(
    state: &AppState,
    station_id: i64,
) -> Result<Vec<RadioTrackRow>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT title, source, duration_seconds
        FROM radio_station_recent_tracks
        WHERE station_id = ?
        ORDER BY datetime(played_at) DESC
        LIMIT 12
        "#,
    )
    .bind(station_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| RadioTrackRow {
            title: row
                .try_get::<String, _>("title")
                .unwrap_or_else(|_| "Untitled".to_string()),
            source: row
                .try_get::<String, _>("source")
                .unwrap_or_else(|_| "YouTube".to_string()),
            duration_label: row
                .try_get::<i64, _>("duration_seconds")
                .map(|v| format!("{v}s"))
                .unwrap_or_else(|_| "--".to_string()),
        })
        .collect())
}
