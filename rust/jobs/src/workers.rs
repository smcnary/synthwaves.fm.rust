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
            "-y",
            "-i",
            input,
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            "-movflags",
            "+faststart",
            output,
        ])
        .status()
        .context("failed to spawn ffmpeg for video conversion")?;
    anyhow::ensure!(status.success(), "ffmpeg video conversion failed");
    Ok(())
}

pub async fn station_listener_sync_job() -> anyhow::Result<()> {
    let cfg = infra::config::AppConfig::from_env()?;
    let pool = infra::db::connect(&cfg.database_url).await?;
    let radio_rows = sqlx::query(
        r#"
        SELECT id, COALESCE(mount_point, '/radio/' || id || '.mp3') AS mount_point
        FROM radio_stations
        "#,
    )
    .fetch_all(&pool)
    .await
    .context("failed to load radio stations for listener sync")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS radio_station_listener_stats (
          station_id INTEGER PRIMARY KEY,
          listener_count INTEGER NOT NULL DEFAULT 0,
          updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&pool)
    .await
    .context("failed to ensure listener stats table")?;

    let icecast_cfg = infra::icecast::IcecastConfig {
        protocol: cfg.icecast_protocol,
        host: cfg.icecast_host,
        port: cfg.icecast_port,
        admin_username: cfg.icecast_admin_username,
        admin_password: cfg.icecast_admin_password,
    };

    for row in radio_rows {
        let station_id: i64 = sqlx::Row::try_get(&row, "id").unwrap_or_default();
        let mount_point: String =
            sqlx::Row::try_get(&row, "mount_point").unwrap_or_else(|_| "/".to_string());
        let listeners = infra::icecast::fetch_listener_count(&icecast_cfg, &mount_point)
            .await
            .unwrap_or(0);
        let _ = sqlx::query(
            r#"
            INSERT INTO radio_station_listener_stats (station_id, listener_count)
            VALUES (?, ?)
            ON CONFLICT(station_id) DO UPDATE SET listener_count = excluded.listener_count, updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(station_id)
        .bind(listeners)
        .execute(&pool)
        .await;
    }
    Ok(())
}

pub async fn database_backup_job() -> anyhow::Result<()> {
    Ok(())
}

pub async fn youtube_playlist_sync_job() -> anyhow::Result<()> {
    let cfg = infra::config::AppConfig::from_env()?;
    if !cfg.youtube_import_enabled || !cfg.youtube_import_scheduler_enabled {
        return Ok(());
    }
    let pool = infra::db::connect(&cfg.database_url).await?;
    let source_ids =
        infra::youtube_import::due_source_ids(&pool, cfg.youtube_import_default_sync_interval_minutes)
            .await?;
    for source_id in source_ids {
        let _ = infra::youtube_import::run_source_import(&pool, &cfg, source_id, "scheduler").await;
    }
    Ok(())
}
