use anyhow::Context;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};

pub async fn connect(database_url: &str) -> anyhow::Result<Pool<Sqlite>> {
    SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .with_context(|| format!("failed to connect to sqlite at {database_url}"))
}
