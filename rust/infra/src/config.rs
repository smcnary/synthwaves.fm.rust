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
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let cfg = Config::builder()
            .add_source(Environment::default().separator("_"))
            .build()?;
        Ok(cfg.try_deserialize()?)
    }
}
