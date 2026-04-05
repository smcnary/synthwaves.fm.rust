use anyhow::{Context, anyhow};
use sqlx::{Row, SqlitePool};
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::youtube;

#[derive(Debug, Clone)]
pub struct YoutubePlaylistSource {
    pub id: i64,
    pub name: String,
    pub playlist_url: String,
    pub playlist_id: String,
    pub target_playlist_name: String,
    pub target_playlist_id: Option<i64>,
    pub enabled: bool,
    pub sync_interval_minutes: i64,
    pub last_synced_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateYoutubeSourceInput {
    pub name: String,
    pub playlist_url: String,
    pub target_playlist_name: String,
    pub enabled: bool,
    pub sync_interval_minutes: i64,
    pub created_by_user_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateYoutubeSourceInput {
    pub name: Option<String>,
    pub target_playlist_name: Option<String>,
    pub enabled: Option<bool>,
    pub sync_interval_minutes: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct YoutubeImportRun {
    pub id: i64,
    pub source_id: i64,
    pub triggered_by: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub imported_count: i64,
    pub skipped_count: i64,
    pub failed_count: i64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct YoutubeImportRunResult {
    pub run_id: i64,
    pub source_id: i64,
    pub status: String,
    pub imported_count: i64,
    pub skipped_count: i64,
    pub failed_count: i64,
    pub last_error: Option<String>,
}

pub async fn ensure_schema(pool: &SqlitePool) -> anyhow::Result<()> {
    // Keep this runtime-safe for existing environments where migrations may lag.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS youtube_playlist_sources (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          playlist_url TEXT NOT NULL,
          playlist_id TEXT NOT NULL,
          target_playlist_name TEXT NOT NULL,
          target_playlist_id INTEGER,
          enabled INTEGER NOT NULL DEFAULT 1,
          sync_interval_minutes INTEGER NOT NULL DEFAULT 60,
          last_synced_at TEXT,
          last_error TEXT,
          created_by_user_id TEXT,
          created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
          UNIQUE(playlist_id, target_playlist_name)
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS youtube_import_runs (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          source_id INTEGER NOT NULL,
          triggered_by TEXT NOT NULL,
          status TEXT NOT NULL DEFAULT 'running',
          started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
          finished_at TEXT,
          imported_count INTEGER NOT NULL DEFAULT 0,
          skipped_count INTEGER NOT NULL DEFAULT 0,
          failed_count INTEGER NOT NULL DEFAULT 0,
          last_error TEXT,
          created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS youtube_import_items (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          source_id INTEGER NOT NULL,
          video_id TEXT NOT NULL,
          title TEXT NOT NULL,
          latest_position INTEGER NOT NULL,
          import_status TEXT NOT NULL DEFAULT 'pending',
          last_error TEXT,
          imported_track_id INTEGER,
          last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
          last_imported_at TEXT,
          updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
          UNIQUE(source_id, video_id)
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS playlists (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tracks (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          title TEXT NOT NULL,
          artist TEXT NOT NULL DEFAULT 'Unknown Artist',
          album TEXT NOT NULL DEFAULT 'Unknown Album',
          source TEXT NOT NULL DEFAULT 'youtube',
          youtube_video_id TEXT,
          duration_seconds INTEGER,
          created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
          updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_youtube_video_id ON tracks(youtube_video_id) WHERE youtube_video_id IS NOT NULL",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS playlist_tracks (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          playlist_id INTEGER NOT NULL,
          track_id INTEGER NOT NULL,
          position INTEGER NOT NULL,
          created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
          UNIQUE(playlist_id, track_id),
          UNIQUE(playlist_id, position)
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS active_storage_blobs (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          key TEXT NOT NULL UNIQUE,
          filename TEXT NOT NULL,
          content_type TEXT,
          byte_size INTEGER NOT NULL DEFAULT 0,
          created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS active_storage_attachments (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          name TEXT NOT NULL,
          record_type TEXT NOT NULL,
          record_id INTEGER NOT NULL,
          blob_id INTEGER NOT NULL,
          created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
          UNIQUE(record_type, record_id, name)
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn create_source(
    pool: &SqlitePool,
    input: CreateYoutubeSourceInput,
) -> anyhow::Result<YoutubePlaylistSource> {
    ensure_schema(pool).await?;
    let playlist_id = youtube::extract_playlist_id(&input.playlist_url)
        .ok_or_else(|| anyhow!("playlist URL must include list=..."))?;
    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO youtube_playlist_sources (
          name, playlist_url, playlist_id, target_playlist_name, enabled, sync_interval_minutes, created_by_user_id
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(input.name)
    .bind(input.playlist_url)
    .bind(playlist_id)
    .bind(input.target_playlist_name)
    .bind(if input.enabled { 1 } else { 0 })
    .bind(input.sync_interval_minutes.max(1))
    .bind(input.created_by_user_id)
    .fetch_one(pool)
    .await?;
    get_source(pool, id).await
}

pub async fn update_source(
    pool: &SqlitePool,
    source_id: i64,
    input: UpdateYoutubeSourceInput,
) -> anyhow::Result<YoutubePlaylistSource> {
    ensure_schema(pool).await?;
    let existing = get_source(pool, source_id).await?;
    let next_name = input.name.unwrap_or(existing.name);
    let next_target = input
        .target_playlist_name
        .unwrap_or(existing.target_playlist_name);
    let next_enabled = input.enabled.unwrap_or(existing.enabled);
    let next_interval = input
        .sync_interval_minutes
        .unwrap_or(existing.sync_interval_minutes)
        .max(1);
    sqlx::query(
        r#"
        UPDATE youtube_playlist_sources
        SET name = ?, target_playlist_name = ?, enabled = ?, sync_interval_minutes = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(next_name)
    .bind(next_target)
    .bind(if next_enabled { 1 } else { 0 })
    .bind(next_interval)
    .bind(source_id)
    .execute(pool)
    .await?;
    get_source(pool, source_id).await
}

pub async fn get_source(
    pool: &SqlitePool,
    source_id: i64,
) -> anyhow::Result<YoutubePlaylistSource> {
    ensure_schema(pool).await?;
    let row = sqlx::query(
        r#"
        SELECT id, name, playlist_url, playlist_id, target_playlist_name, target_playlist_id, enabled,
               sync_interval_minutes, last_synced_at, last_error
        FROM youtube_playlist_sources
        WHERE id = ?
        LIMIT 1
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await?;
    let row = row.ok_or_else(|| anyhow!("source not found"))?;
    Ok(map_source_row(&row))
}

pub async fn list_sources(pool: &SqlitePool) -> anyhow::Result<Vec<YoutubePlaylistSource>> {
    ensure_schema(pool).await?;
    let rows = sqlx::query(
        r#"
        SELECT id, name, playlist_url, playlist_id, target_playlist_name, target_playlist_id, enabled,
               sync_interval_minutes, last_synced_at, last_error
        FROM youtube_playlist_sources
        ORDER BY id DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| map_source_row(&row)).collect())
}

pub async fn list_runs(
    pool: &SqlitePool,
    source_id: i64,
    limit: i64,
) -> anyhow::Result<Vec<YoutubeImportRun>> {
    ensure_schema(pool).await?;
    let rows = sqlx::query(
        r#"
        SELECT id, source_id, triggered_by, status, started_at, finished_at, imported_count, skipped_count, failed_count, last_error
        FROM youtube_import_runs
        WHERE source_id = ?
        ORDER BY id DESC
        LIMIT ?
        "#,
    )
    .bind(source_id)
    .bind(limit.max(1))
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| YoutubeImportRun {
            id: row.get("id"),
            source_id: row.get("source_id"),
            triggered_by: row.get("triggered_by"),
            status: row.get("status"),
            started_at: row.get("started_at"),
            finished_at: row.try_get("finished_at").ok(),
            imported_count: row.get("imported_count"),
            skipped_count: row.get("skipped_count"),
            failed_count: row.get("failed_count"),
            last_error: row.try_get("last_error").ok(),
        })
        .collect())
}

pub async fn due_source_ids(pool: &SqlitePool, default_minutes: i64) -> anyhow::Result<Vec<i64>> {
    ensure_schema(pool).await?;
    let rows = sqlx::query(
        r#"
        SELECT s.id
        FROM youtube_playlist_sources s
        LEFT JOIN youtube_import_runs r
          ON r.source_id = s.id
         AND r.id = (SELECT MAX(id) FROM youtube_import_runs WHERE source_id = s.id)
        WHERE s.enabled = 1
          AND (
            s.last_synced_at IS NULL OR
            datetime(s.last_synced_at) <= datetime('now', '-' || COALESCE(NULLIF(s.sync_interval_minutes, 0), ?) || ' minutes')
          )
          AND COALESCE(r.status, 'ok') != 'running'
        ORDER BY s.id ASC
        "#,
    )
    .bind(default_minutes.max(1))
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get("id").ok())
        .collect())
}

pub async fn run_source_import(
    pool: &SqlitePool,
    config: &AppConfig,
    source_id: i64,
    triggered_by: &str,
) -> anyhow::Result<YoutubeImportRunResult> {
    ensure_schema(pool).await?;
    let source = get_source(pool, source_id).await?;
    if !source.enabled {
        return Ok(YoutubeImportRunResult {
            run_id: 0,
            source_id,
            status: "skipped_disabled".to_string(),
            imported_count: 0,
            skipped_count: 0,
            failed_count: 0,
            last_error: None,
        });
    }

    let running_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM youtube_import_runs WHERE source_id = ? AND status = 'running'",
    )
    .bind(source_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    if running_count > 0 {
        return Ok(YoutubeImportRunResult {
            run_id: 0,
            source_id,
            status: "skipped_running".to_string(),
            imported_count: 0,
            skipped_count: 0,
            failed_count: 0,
            last_error: None,
        });
    }

    let run_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO youtube_import_runs (source_id, triggered_by, status)
        VALUES (?, ?, 'running')
        RETURNING id
        "#,
    )
    .bind(source_id)
    .bind(triggered_by)
    .fetch_one(pool)
    .await?;

    let entries = match tokio::task::spawn_blocking({
        let playlist_url = source.playlist_url.clone();
        move || youtube::fetch_playlist_entries(&playlist_url)
    })
    .await
    {
        Ok(Ok(entries)) => entries,
        Ok(Err(err)) => {
            finalize_run(
                pool,
                run_id,
                source_id,
                "failed",
                0,
                0,
                1,
                Some(err.to_string()),
            )
            .await?;
            return Ok(YoutubeImportRunResult {
                run_id,
                source_id,
                status: "failed".to_string(),
                imported_count: 0,
                skipped_count: 0,
                failed_count: 1,
                last_error: Some(err.to_string()),
            });
        }
        Err(err) => {
            let message = format!("playlist worker join failed: {err}");
            finalize_run(
                pool,
                run_id,
                source_id,
                "failed",
                0,
                0,
                1,
                Some(message.clone()),
            )
            .await?;
            return Ok(YoutubeImportRunResult {
                run_id,
                source_id,
                status: "failed".to_string(),
                imported_count: 0,
                skipped_count: 0,
                failed_count: 1,
                last_error: Some(message),
            });
        }
    };

    if entries.is_empty() {
        finalize_run(
            pool,
            run_id,
            source_id,
            "failed",
            0,
            0,
            1,
            Some("playlist contains no videos".to_string()),
        )
        .await?;
        return Ok(YoutubeImportRunResult {
            run_id,
            source_id,
            status: "failed".to_string(),
            imported_count: 0,
            skipped_count: 0,
            failed_count: 1,
            last_error: Some("playlist contains no videos".to_string()),
        });
    }

    let max_items = config.youtube_import_max_items_per_run.max(1) as usize;
    let playlist_id = ensure_playlist(pool, &source.target_playlist_name).await?;
    if source.target_playlist_id != Some(playlist_id) {
        let _ = sqlx::query(
            "UPDATE youtube_playlist_sources SET target_playlist_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(playlist_id)
        .bind(source_id)
        .execute(pool)
        .await;
    }

    let mut imported = 0_i64;
    let mut skipped = 0_i64;
    let mut failed = 0_i64;
    let mut last_error: Option<String> = None;

    for (idx, item) in entries.into_iter().take(max_items).enumerate() {
        let result = import_entry(
            pool,
            source_id,
            playlist_id,
            idx as i64 + 1,
            &item.video_id,
            &item.title,
            config.youtube_import_download_timeout_seconds.max(1) as u64,
        )
        .await;
        match result {
            Ok(import_state) => {
                match import_state {
                    EntryImportState::Imported => imported += 1,
                    EntryImportState::SkippedExisting => skipped += 1,
                }
                let _ = upsert_item_state(
                    pool,
                    source_id,
                    idx as i64 + 1,
                    &item.video_id,
                    &item.title,
                    "ok",
                    None,
                    None,
                )
                .await;
            }
            Err(err) => {
                failed += 1;
                last_error = Some(err.to_string());
                let _ = upsert_item_state(
                    pool,
                    source_id,
                    idx as i64 + 1,
                    &item.video_id,
                    &item.title,
                    "failed",
                    Some(err.to_string()),
                    None,
                )
                .await;
            }
        }
    }

    let status = if failed > 0 { "partial" } else { "ok" };
    finalize_run(
        pool,
        run_id,
        source_id,
        status,
        imported,
        skipped,
        failed,
        last_error.clone(),
    )
    .await?;
    Ok(YoutubeImportRunResult {
        run_id,
        source_id,
        status: status.to_string(),
        imported_count: imported,
        skipped_count: skipped,
        failed_count: failed,
        last_error,
    })
}

pub fn dependency_check() -> anyhow::Result<()> {
    command_exists("yt-dlp").context("yt-dlp missing from PATH")?;
    command_exists("ffmpeg").context("ffmpeg missing from PATH")?;
    Ok(())
}

enum EntryImportState {
    Imported,
    SkippedExisting,
}

async fn import_entry(
    pool: &SqlitePool,
    source_id: i64,
    playlist_id: i64,
    playlist_position_hint: i64,
    video_id: &str,
    title: &str,
    timeout_seconds: u64,
) -> anyhow::Result<EntryImportState> {
    if let Some(track_id) = track_for_video(pool, video_id).await? {
        ensure_playlist_track(pool, playlist_id, track_id, playlist_position_hint).await?;
        let _ = upsert_item_state(
            pool,
            source_id,
            playlist_position_hint,
            video_id,
            title,
            "ok",
            None,
            Some(track_id),
        )
        .await;
        return Ok(EntryImportState::SkippedExisting);
    }

    let tmp_audio = download_audio(video_id, timeout_seconds).await?;
    let meta = std::fs::metadata(&tmp_audio).context("failed to read downloaded audio metadata")?;
    let key = storage_key();
    let storage_path = storage_path_for_key(&key);
    if let Some(parent) = storage_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create storage parent directory {}",
                parent.display()
            )
        })?;
    }
    std::fs::copy(&tmp_audio, &storage_path)
        .with_context(|| format!("failed to copy audio into {}", storage_path.display()))?;
    let _ = std::fs::remove_file(&tmp_audio);

    let mut tx = pool.begin().await?;
    let blob_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO active_storage_blobs (key, filename, content_type, byte_size)
        VALUES (?, ?, 'audio/mpeg', ?)
        RETURNING id
        "#,
    )
    .bind(&key)
    .bind(format!("{video_id}.mp3"))
    .bind(meta.len() as i64)
    .fetch_one(&mut *tx)
    .await?;
    let track_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO tracks (title, artist, album, source, youtube_video_id)
        VALUES (?, 'YouTube', ?, 'youtube', ?)
        RETURNING id
        "#,
    )
    .bind(title)
    .bind(format!("YouTube Playlist {source_id}"))
    .bind(video_id)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO active_storage_attachments (name, record_type, record_id, blob_id)
        VALUES ('audio_file', 'Track', ?, ?)
        "#,
    )
    .bind(track_id)
    .bind(blob_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    ensure_playlist_track(pool, playlist_id, track_id, playlist_position_hint).await?;
    let _ = upsert_item_state(
        pool,
        source_id,
        playlist_position_hint,
        video_id,
        title,
        "ok",
        None,
        Some(track_id),
    )
    .await;
    Ok(EntryImportState::Imported)
}

async fn ensure_playlist(pool: &SqlitePool, target_name: &str) -> anyhow::Result<i64> {
    let existing = sqlx::query_scalar::<_, i64>("SELECT id FROM playlists WHERE name = ? LIMIT 1")
        .bind(target_name)
        .fetch_optional(pool)
        .await?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let id = sqlx::query_scalar::<_, i64>("INSERT INTO playlists (name) VALUES (?) RETURNING id")
        .bind(target_name)
        .fetch_one(pool)
        .await?;
    Ok(id)
}

async fn ensure_playlist_track(
    pool: &SqlitePool,
    playlist_id: i64,
    track_id: i64,
    position_hint: i64,
) -> anyhow::Result<()> {
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?",
    )
    .bind(playlist_id)
    .bind(track_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    if exists > 0 {
        return Ok(());
    }
    let next_position = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM playlist_tracks WHERE playlist_id = ?",
    )
    .bind(playlist_id)
    .fetch_one(pool)
    .await
    .unwrap_or(position_hint.max(1));
    let position = next_position.max(position_hint).max(1);
    sqlx::query("INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?, ?, ?)")
        .bind(playlist_id)
        .bind(track_id)
        .bind(position)
        .execute(pool)
        .await?;
    Ok(())
}

async fn upsert_item_state(
    pool: &SqlitePool,
    source_id: i64,
    position: i64,
    video_id: &str,
    title: &str,
    status: &str,
    error: Option<String>,
    track_id: Option<i64>,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO youtube_import_items (
          source_id, video_id, title, latest_position, import_status, last_error, imported_track_id, last_seen_at, last_imported_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CASE WHEN ? = 'ok' THEN CURRENT_TIMESTAMP ELSE NULL END, CURRENT_TIMESTAMP)
        ON CONFLICT(source_id, video_id) DO UPDATE SET
          title = excluded.title,
          latest_position = excluded.latest_position,
          import_status = excluded.import_status,
          last_error = excluded.last_error,
          imported_track_id = COALESCE(excluded.imported_track_id, youtube_import_items.imported_track_id),
          last_seen_at = CURRENT_TIMESTAMP,
          last_imported_at = CASE WHEN excluded.import_status = 'ok' THEN CURRENT_TIMESTAMP ELSE youtube_import_items.last_imported_at END,
          updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(source_id)
    .bind(video_id)
    .bind(title)
    .bind(position)
    .bind(status)
    .bind(error)
    .bind(track_id)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(())
}

async fn track_for_video(pool: &SqlitePool, video_id: &str) -> anyhow::Result<Option<i64>> {
    let row =
        sqlx::query_scalar::<_, i64>("SELECT id FROM tracks WHERE youtube_video_id = ? LIMIT 1")
            .bind(video_id)
            .fetch_optional(pool)
            .await?;
    Ok(row)
}

async fn finalize_run(
    pool: &SqlitePool,
    run_id: i64,
    source_id: i64,
    status: &str,
    imported_count: i64,
    skipped_count: i64,
    failed_count: i64,
    last_error: Option<String>,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        UPDATE youtube_import_runs
        SET status = ?, finished_at = CURRENT_TIMESTAMP, imported_count = ?, skipped_count = ?, failed_count = ?, last_error = ?
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(imported_count)
    .bind(skipped_count)
    .bind(failed_count)
    .bind(last_error.clone())
    .bind(run_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE youtube_playlist_sources
        SET last_synced_at = CURRENT_TIMESTAMP,
            last_error = ?,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(last_error)
    .bind(source_id)
    .execute(pool)
    .await?;
    Ok(())
}

fn map_source_row(row: &sqlx::sqlite::SqliteRow) -> YoutubePlaylistSource {
    YoutubePlaylistSource {
        id: row.get("id"),
        name: row.get("name"),
        playlist_url: row.get("playlist_url"),
        playlist_id: row.get("playlist_id"),
        target_playlist_name: row.get("target_playlist_name"),
        target_playlist_id: row.try_get("target_playlist_id").ok(),
        enabled: row.get::<i64, _>("enabled") != 0,
        sync_interval_minutes: row.get("sync_interval_minutes"),
        last_synced_at: row.try_get("last_synced_at").ok(),
        last_error: row.try_get("last_error").ok(),
    }
}

async fn download_audio(video_id: &str, timeout_seconds: u64) -> anyhow::Result<PathBuf> {
    let base = std::env::temp_dir().join(format!("synthwaves-youtube-import-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&base)
        .with_context(|| format!("failed to create temp directory {}", base.display()))?;
    let output_template = base.join(format!("{video_id}.%(ext)s"));
    let watch_url = format!("https://www.youtube.com/watch?v={video_id}");
    let output = tokio::task::spawn_blocking(move || {
        Command::new("yt-dlp")
            .args([
                "--no-playlist",
                "--no-warnings",
                "--socket-timeout",
                &timeout_seconds.to_string(),
                "-x",
                "--audio-format",
                "mp3",
                "-o",
                output_template.to_string_lossy().as_ref(),
                watch_url.as_str(),
            ])
            .output()
    })
    .await
    .map_err(|err| anyhow!("yt-dlp worker join failed: {err}"))?
    .context("failed to execute yt-dlp")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("yt-dlp download failed: {stderr}"));
    }

    let audio_file = first_file_in_dir(&base)
        .ok_or_else(|| anyhow!("yt-dlp did not produce an audio file in {}", base.display()))?;
    Ok(audio_file)
}

fn first_file_in_dir(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries {
        let path = entry.ok()?.path();
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn storage_key() -> String {
    Uuid::new_v4().simple().to_string()
}

fn storage_path_for_key(key: &str) -> PathBuf {
    if key.len() >= 4 {
        PathBuf::from(format!("storage/{}/{}/{}", &key[0..2], &key[2..4], key))
    } else {
        PathBuf::from(format!("storage/{key}"))
    }
}

fn command_exists(command: &str) -> anyhow::Result<()> {
    let status = Command::new(command).arg("--version").status();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(anyhow!("{command} exists but returned non-zero status")),
        Err(err) => Err(anyhow!("{command} not executable: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool should open")
    }

    #[tokio::test]
    async fn create_source_and_list_round_trip() -> anyhow::Result<()> {
        let pool = test_pool().await;
        ensure_schema(&pool).await?;
        let source = create_source(
            &pool,
            CreateYoutubeSourceInput {
                name: "Night Drive".to_string(),
                playlist_url: "https://www.youtube.com/playlist?list=PL1234567890".to_string(),
                target_playlist_name: "Night Drive Imports".to_string(),
                enabled: true,
                sync_interval_minutes: 30,
                created_by_user_id: Some("user-1".to_string()),
            },
        )
        .await?;
        assert_eq!(source.playlist_id, "PL1234567890");
        let all = list_sources(&pool).await?;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].target_playlist_name, "Night Drive Imports");
        Ok(())
    }

    #[tokio::test]
    async fn due_source_ids_respects_enabled_and_interval() -> anyhow::Result<()> {
        let pool = test_pool().await;
        ensure_schema(&pool).await?;
        let _ = create_source(
            &pool,
            CreateYoutubeSourceInput {
                name: "Enabled".to_string(),
                playlist_url: "https://www.youtube.com/playlist?list=PLENABLED".to_string(),
                target_playlist_name: "Enabled Playlist".to_string(),
                enabled: true,
                sync_interval_minutes: 60,
                created_by_user_id: None,
            },
        )
        .await?;
        let disabled = create_source(
            &pool,
            CreateYoutubeSourceInput {
                name: "Disabled".to_string(),
                playlist_url: "https://www.youtube.com/playlist?list=PLDISABLED".to_string(),
                target_playlist_name: "Disabled Playlist".to_string(),
                enabled: false,
                sync_interval_minutes: 60,
                created_by_user_id: None,
            },
        )
        .await?;
        let ids = due_source_ids(&pool, 60).await?;
        assert_eq!(ids.len(), 1);
        assert_ne!(ids[0], disabled.id);
        Ok(())
    }
}
