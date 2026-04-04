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
    /// When set (e.g. `https://radio.example.com`), browser stream URLs use this base instead of
    /// `ICECAST_PROTOCOL` + `ICECAST_HOST` + `ICECAST_PORT` (needed behind TLS or when admin host is internal).
    #[serde(default)]
    pub icecast_public_base_url: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let cfg = Config::builder()
            .set_default("host", "127.0.0.1")?
            .set_default("port", 4000)?
            .set_default("database_url", "sqlite:/data/development.sqlite3")?
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

    /// Base URL for listeners (no trailing slash). Used to build mount URLs for `<audio src>`.
    pub fn icecast_public_base(&self) -> String {
        if let Some(ref url) = self.icecast_public_base_url {
            let t = url.trim();
            if !t.is_empty() {
                return t.trim_end_matches('/').to_string();
            }
        }
        format!(
            "{}://{}:{}",
            self.icecast_protocol.trim_end_matches('/'),
            self.icecast_host,
            self.icecast_port
        )
    }
}
