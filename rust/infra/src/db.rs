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

    tracing::debug!(file_path, "resolved sqlite file path after stripping scheme prefix");

    if let Some(parent) = Path::new(&file_path).parent() {
        if !parent.as_os_str().is_empty() {
            let parent_exists_before = parent.exists();
            tracing::debug!(
                parent = %parent.display(),
                exists = parent_exists_before,
                "parent directory status before create_dir_all"
            );

            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create database directory: {}", parent.display())
            })?;

            let parent_exists_after = parent.exists();
            tracing::debug!(
                parent = %parent.display(),
                exists = parent_exists_after,
                "parent directory status after create_dir_all"
            );

            // Check whether the directory is actually writable by inspecting
            // its metadata and permissions.
            match std::fs::metadata(parent) {
                Ok(meta) => {
                    let readonly = meta.permissions().readonly();
                    tracing::debug!(
                        parent = %parent.display(),
                        readonly,
                        "parent directory metadata after create_dir_all"
                    );
                    if readonly {
                        tracing::warn!(
                            parent = %parent.display(),
                            "parent directory is read-only; sqlite will be unable to create the database file"
                        );
                    }
                }
                Err(err) => {
                    tracing::warn!(
                        parent = %parent.display(),
                        error = %err,
                        "failed to read metadata for parent directory"
                    );
                }
            }
        }
    }

    tracing::debug!(database_url, "connecting to sqlite with database_url");

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .with_context(|| {
            format!(
                "failed to connect to sqlite at {database_url} (file path: {file_path})"
            )
        })
}
