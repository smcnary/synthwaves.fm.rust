use anyhow::Context;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::path::Path;

pub async fn connect(database_url: &str) -> anyhow::Result<Pool<Sqlite>> {
    // SQLite URLs look like "sqlite:///absolute/path/to/db.sqlite3" or
    // "sqlite://relative/path/to/db.sqlite3". Strip the scheme prefix and
    // ensure the parent directory exists so SQLite can create the file.
    let file_path = database_url
        .strip_prefix("sqlite://")
        .unwrap_or(database_url);

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
