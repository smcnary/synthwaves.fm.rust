use infra::config::AppConfig;
use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub pool: Pool<Sqlite>,
}
