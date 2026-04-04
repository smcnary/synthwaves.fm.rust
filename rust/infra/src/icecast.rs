use anyhow::Context;
use serde_json::Value;

/// Builds a full stream URL for a mount such as `/radio/1.mp3`.
pub fn icecast_stream_url(public_base: &str, mount_point: &str) -> String {
    let base = public_base.trim_end_matches('/');
    let mount = mount_point.trim();
    let path = if mount.starts_with('/') {
        mount.to_string()
    } else {
        format!("/{mount}")
    };
    format!("{base}{path}")
}

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

#[cfg(test)]
mod stream_url_tests {
    use super::icecast_stream_url;

    #[test]
    fn joins_base_and_mount() {
        assert_eq!(
            icecast_stream_url("http://localhost:8000", "/radio/1.mp3"),
            "http://localhost:8000/radio/1.mp3"
        );
    }

    #[test]
    fn trims_base_slash_and_normalizes_mount() {
        assert_eq!(
            icecast_stream_url("https://stream.example/", "radio/2.mp3"),
            "https://stream.example/radio/2.mp3"
        );
    }
}
