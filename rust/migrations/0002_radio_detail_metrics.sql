ALTER TABLE radio_stations ADD COLUMN listener_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE radio_stations ADD COLUMN last_stream_started_at TEXT;

CREATE TABLE IF NOT EXISTS radio_station_runtime_state (
  station_id INTEGER PRIMARY KEY,
  current_video_id TEXT,
  current_title TEXT,
  current_source TEXT DEFAULT 'YouTube',
  current_duration_seconds INTEGER,
  last_track_started_at TEXT,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS radio_station_recent_tracks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  station_id INTEGER NOT NULL,
  title TEXT NOT NULL,
  source TEXT NOT NULL DEFAULT 'YouTube',
  duration_seconds INTEGER,
  played_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS radio_station_listener_stats (
  station_id INTEGER PRIMARY KEY,
  listener_count INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS radio_station_queue_state (
  station_id INTEGER PRIMARY KEY,
  current_position INTEGER NOT NULL DEFAULT 1,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
