use anyhow::Context;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct IcecastConfig {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub admin_username: String,
    pub admin_password: String,
}

pub async fn fetch_listener_count(cfg: &IcecastConfig, mount_point: &str) -> anyhow::Result<i64> {
    let url = format!(
        "{}://{}:{}/status-json.xsl",
        cfg.protocol, cfg.host, cfg.port
    );
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .basic_auth(&cfg.admin_username, Some(&cfg.admin_password))
        .send()
        .await
        .context("failed to request icecast status-json.xsl")?
        .error_for_status()
        .context("icecast returned error status")?;
    let body: Value = response
        .json()
        .await
        .context("failed to parse icecast status JSON")?;

    let source = &body["icestats"]["source"];
    let normalize = |m: &str| {
        if m.starts_with('/') {
            m.to_string()
        } else {
            format!("/{m}")
        }
    };
    let target = normalize(mount_point);

    if source.is_object() {
        let listenurl = source["listenurl"].as_str().unwrap_or_default();
        let mount = source["server_name"].as_str().unwrap_or(listenurl);
        let fallback = source["stream_start"].as_str().unwrap_or_default();
        if listenurl.contains(&target) || mount.contains(&target) || fallback.contains(&target) {
            return Ok(source["listeners"].as_i64().unwrap_or(0));
        }
        return Ok(0);
    }

    let Some(items) = source.as_array() else {
        return Ok(0);
    };
    for item in items {
        let listenurl = item["listenurl"].as_str().unwrap_or_default();
        let mount = item["server_name"].as_str().unwrap_or(listenurl);
        let fallback = item["stream_start"].as_str().unwrap_or_default();
        if listenurl.contains(&target) || mount.contains(&target) || fallback.contains(&target) {
            return Ok(item["listeners"].as_i64().unwrap_or(0));
        }
    }
    Ok(0)
}
