use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use sqlx::Row;
use tokio::fs;

use crate::app_state::AppState;

pub async fn track_stream(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    let row = sqlx::query(
        r#"
        SELECT b.key, COALESCE(b.content_type, 'audio/mpeg') AS content_type
        FROM active_storage_attachments a
        JOIN active_storage_blobs b ON b.id = a.blob_id
        WHERE a.record_type = 'Track' AND a.name = 'audio_file' AND a.record_id = ?
        LIMIT 1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    let Ok(Some(row)) = row else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let key: String = row.get("key");
    let content_type: String = row.get("content_type");
    match read_storage_blob(&key).await {
        Ok(bytes) => file_response(bytes, &content_type),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn demo_stream() -> Response {
    match fs::read("storage/demo.mp3").await {
        Ok(bytes) => file_response(bytes, "audio/mpeg"),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

fn file_response(bytes: Vec<u8>, content_type: &str) -> Response {
    let mut response = Response::new(Body::from(bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type).unwrap_or(HeaderValue::from_static("audio/mpeg")),
    );
    response
        .headers_mut()
        .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    response
}

async fn read_storage_blob(key: &str) -> std::io::Result<Vec<u8>> {
    let nested = if key.len() >= 4 {
        format!("storage/{}/{}/{}", &key[0..2], &key[2..4], key)
    } else {
        format!("storage/{key}")
    };
    let candidates = [
        nested,
        format!("../storage/{key}"),
        format!("storage/{key}"),
    ];
    for candidate in &candidates {
        if let Ok(bytes) = fs::read(candidate).await {
            return Ok(bytes);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "blob file not found",
    ))
}
