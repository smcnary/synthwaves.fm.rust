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
);

CREATE INDEX IF NOT EXISTS idx_youtube_playlist_sources_enabled
  ON youtube_playlist_sources(enabled);

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
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY(source_id) REFERENCES youtube_playlist_sources(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_youtube_import_runs_source_id
  ON youtube_import_runs(source_id);

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
  UNIQUE(source_id, video_id),
  FOREIGN KEY(source_id) REFERENCES youtube_playlist_sources(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_youtube_import_items_source_status
  ON youtube_import_items(source_id, import_status);

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
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_youtube_video_id
  ON tracks(youtube_video_id)
  WHERE youtube_video_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS playlist_tracks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  playlist_id INTEGER NOT NULL,
  track_id INTEGER NOT NULL,
  position INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(playlist_id, track_id),
  UNIQUE(playlist_id, position),
  FOREIGN KEY(playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
  FOREIGN KEY(track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS active_storage_blobs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  key TEXT NOT NULL UNIQUE,
  filename TEXT NOT NULL,
  content_type TEXT,
  byte_size INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS active_storage_attachments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  record_type TEXT NOT NULL,
  record_id INTEGER NOT NULL,
  blob_id INTEGER NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(record_type, record_id, name),
  FOREIGN KEY(blob_id) REFERENCES active_storage_blobs(id) ON DELETE CASCADE
);
