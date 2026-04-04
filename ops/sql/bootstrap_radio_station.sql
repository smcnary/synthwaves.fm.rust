-- Bootstrap a production radio station (SQLite).
-- Update placeholder values before running.
--
-- Run example:
--   sqlite3 /path/to/development.sqlite3 < ops/sql/bootstrap_radio_station.sql
--
-- Required existing data:
--   - users.id value for OWNER_USER_ID

-- 1) Create playlist row for the station.
INSERT INTO playlists (name)
VALUES ('Synthwave FM Playlist');

-- 2) Create radio station row linked to the playlist.
-- Replace OWNER_USER_ID with a real users.id UUID from your database.
INSERT INTO radio_stations (
  user_id,
  playlist_id,
  mount_point,
  status,
  bitrate,
  crossfade,
  crossfade_duration
)
VALUES (
  'OWNER_USER_ID',
  (SELECT id FROM playlists ORDER BY id DESC LIMIT 1),
  '/radio/1.mp3',
  'live',
  192,
  1,
  6
);

-- 3) Verify bootstrap result.
SELECT
  rs.id AS station_id,
  p.name AS playlist_name,
  rs.mount_point,
  rs.status,
  rs.bitrate
FROM radio_stations rs
JOIN playlists p ON p.id = rs.playlist_id
ORDER BY rs.id DESC
LIMIT 5;
