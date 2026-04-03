use anyhow::Context;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::path::Path;

pub async fn connect(database_url: &str) -> anyhow::Result<Pool<Sqlite>> {
    // SQLite URLs look like "sqlite:/absolute/path" (sqlx absolute),
    // "sqlite:///absolute/path" (three-slash legacy), or a bare file path.
    // Strip the scheme prefix to get the raw filesystem path so we can
    // ensure the parent directory exists before sqlx opens the file.
    let file_path = database_url
        .strip_prefix("sqlite:///")
        .map(|p| format!("/{p}"))
        .or_else(|| database_url.strip_prefix("sqlite:/").map(|p| format!("/{p}")))
        .unwrap_or_else(|| database_url.to_string());

    if let Some(parent) = Path::new(file_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create database directory: {}", parent.display())
            })?;
        }
    }

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .with_context(|| format!("failed to connect to sqlite at {database_url}"))
}
