use anyhow::Context;
use std::process::Command;

pub async fn audio_conversion_job(input: &str, output: &str) -> anyhow::Result<()> {
    let status = Command::new("ffmpeg")
        .args(["-y", "-i", input, "-b:a", "192k", output])
        .status()
        .context("failed to spawn ffmpeg for audio conversion")?;
    anyhow::ensure!(status.success(), "ffmpeg audio conversion failed");
    Ok(())
}

pub async fn video_conversion_job(input: &str, output: &str) -> anyhow::Result<()> {
    let status = Command::new("ffmpeg")
        .args([
            "-y", "-i", input, "-c:v", "libx264", "-c:a", "aac", "-movflags", "+faststart", output,
        ])
        .status()
        .context("failed to spawn ffmpeg for video conversion")?;
    anyhow::ensure!(status.success(), "ffmpeg video conversion failed");
    Ok(())
}

pub async fn station_listener_sync_job() -> anyhow::Result<()> {
    Ok(())
}

pub async fn database_backup_job() -> anyhow::Result<()> {
    Ok(())
}
