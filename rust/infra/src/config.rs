use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub liquidsoap_api_token: String,
    pub rails_host: String,
    pub rails_protocol: String,
    pub icecast_protocol: String,
    pub icecast_host: String,
    pub icecast_port: u16,
    pub icecast_admin_username: String,
    pub icecast_admin_password: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let cfg = Config::builder()
            .set_default("host", "127.0.0.1")?
            .set_default("port", 4000)?
            .set_default("database_url", "sqlite:storage/development.sqlite3")?
            .set_default("jwt_secret", "dev-secret")?
            .set_default("liquidsoap_api_token", "dev-liquidsoap-token")?
            .set_default("rails_host", "localhost:4000")?
            .set_default("rails_protocol", "http")?
            .set_default("icecast_protocol", "http")?
            .set_default("icecast_host", "localhost")?
            .set_default("icecast_port", 8000)?
            .set_default("icecast_admin_username", "admin")?
            .set_default("icecast_admin_password", "hackme")?
            .add_source(Environment::default().separator("_"))
            .build()?;
        Ok(cfg.try_deserialize()?)
    }
}
