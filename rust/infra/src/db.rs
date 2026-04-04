use anyhow::Context;
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;
use std::str::FromStr;

pub async fn connect(database_url: &str) -> anyhow::Result<Pool<Sqlite>> {
    // SQLite URLs look like:
    //   "sqlite:relative/path"       (relative, no leading slash — sqlx relative)
    //   "sqlite:/absolute/path"      (sqlx absolute, single slash)
    //   "sqlite:///absolute/path"    (three-slash legacy absolute)
    //   or a bare file path with no scheme at all.
    // Strip the scheme prefix to get the raw filesystem path so we can
    // ensure the parent directory exists before sqlx opens the file.
    let file_path = if let Some(p) = database_url.strip_prefix("sqlite:///") {
        // Three-slash form: "sqlite:///abs/path" → "/abs/path"
        format!("/{p}")
    } else if let Some(p) = database_url.strip_prefix("sqlite:/") {
        // Single-slash absolute form: "sqlite:/abs/path" → "/abs/path"
        format!("/{p}")
    } else if let Some(p) = database_url.strip_prefix("sqlite:") {
        // Relative form: "sqlite:rel/path" → "rel/path" (resolved against cwd)
        p.to_string()
    } else {
        // Bare path, no scheme
        database_url.to_string()
    };

    tracing::debug!(
        file_path,
        "resolved sqlite file path after stripping scheme prefix"
    );

    if let Some(parent) = Path::new(&file_path).parent()
        && !parent.as_os_str().is_empty()
    {
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

    tracing::debug!(database_url, "connecting to sqlite with database_url");

    let connect_options = SqliteConnectOptions::from_str(database_url)
        .with_context(|| format!("invalid sqlite database url: {database_url}"))?
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(connect_options)
        .await
        .with_context(|| {
            format!("failed to connect to sqlite at {database_url} (file path: {file_path})")
        })
}

#[cfg(test)]
mod tests {
    use super::connect;
    use uuid::Uuid;

    #[tokio::test]
    async fn connect_creates_missing_parent_directory_for_sqlite_file() -> anyhow::Result<()> {
        let root =
            std::env::temp_dir().join(format!("synthwaves-db-connect-test-{}", Uuid::new_v4()));
        let db_path = root.join("nested").join("development.sqlite3");

        let parent = db_path.parent().expect("db path should have parent");
        assert!(
            !parent.exists(),
            "test precondition: parent directory should not exist"
        );

        let database_url = format!("sqlite://{}", db_path.display());
        let pool = connect(&database_url).await?;
        sqlx::query("SELECT 1").execute(&pool).await?;
        drop(pool);

        assert!(parent.exists(), "connect should create parent directory");
        assert!(
            db_path.exists(),
            "sqlite file should be created after first query"
        );

        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir_all(&root);

        Ok(())
    }
}
