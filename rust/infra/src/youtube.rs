use anyhow::{Context, anyhow};
use reqwest::Url;
use serde_json::Value;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PlaylistVideo {
    pub video_id: String,
    pub title: String,
}

pub fn extract_playlist_id(input: &str) -> Option<String> {
    let url = Url::parse(input).ok()?;
    url.query_pairs()
        .find_map(|(k, v)| (k == "list").then(|| v.to_string()))
}

pub fn fetch_playlist_entries(playlist_url: &str) -> anyhow::Result<Vec<PlaylistVideo>> {
    let output = run_yt_dlp(&[
        "--yes-playlist",
        "--flat-playlist",
        "--dump-single-json",
        "--no-warnings",
        "--skip-download",
        playlist_url,
    ])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("yt-dlp failed while fetching playlist: {stderr}"));
    }

    let root: Value =
        serde_json::from_slice(&output.stdout).context("failed to parse yt-dlp playlist JSON")?;
    let mut videos = Vec::new();
    if let Some(entries) = root["entries"].as_array() {
        videos.reserve(entries.len());
        for entry in entries {
            let Some(id) = entry["id"].as_str() else {
                continue;
            };
            let title = entry["title"]
                .as_str()
                .map(ToString::to_string)
                .unwrap_or_else(|| "Untitled".to_string());
            videos.push(PlaylistVideo {
                video_id: id.to_string(),
                title,
            });
        }
    } else if let Some(id) = root["id"].as_str() {
        // Some YouTube list URLs resolve as a single playable "mix" item.
        let title = root["title"]
            .as_str()
            .map(ToString::to_string)
            .unwrap_or_else(|| "Untitled".to_string());
        videos.push(PlaylistVideo {
            video_id: id.to_string(),
            title,
        });
    }
    Ok(videos)
}

pub fn resolve_audio_stream_url(video_id: &str) -> anyhow::Result<String> {
    let watch_url = format!("https://www.youtube.com/watch?v={video_id}");
    let output = run_yt_dlp(&["-f", "bestaudio", "-g", "--no-warnings", &watch_url])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "yt-dlp failed to resolve audio stream URL: {stderr}"
        ));
    }

    let line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| anyhow!("yt-dlp produced empty stream URL output"))?
        .trim()
        .to_string();
    Ok(line)
}

fn run_yt_dlp(args: &[&str]) -> anyhow::Result<std::process::Output> {
    let candidates = ["yt-dlp", "/opt/homebrew/bin/yt-dlp"];
    let mut last_err = None;
    for bin in candidates {
        match Command::new(bin).args(args).output() {
            Ok(out) => return Ok(out),
            Err(err) => last_err = Some((bin, err)),
        }
    }
    if let Some((bin, err)) = last_err {
        Err(anyhow!(
            "failed to run yt-dlp (tried {bin}): {err}. Install yt-dlp and ensure it is on PATH"
        ))
    } else {
        Err(anyhow!("failed to run yt-dlp"))
    }
}
