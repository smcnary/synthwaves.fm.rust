# synthwaves.fm (Rust)

Rust implementation of synthwaves.fm using an Axum workspace.

## Workspace Layout

- `rust/axum-app` - HTTP server and route handlers
- `rust/domain` - domain models and shared types
- `rust/infra` - configuration, DB access, and integration helpers
- `rust/jobs` - recurring and background job workers

## Local Development

Requirements:

- Rust (stable toolchain)
- SQLite3
- ffmpeg (for media workflows)

Run the app:

```bash
cd rust
cargo run -p axum-app
```

Run checks:

```bash
cd rust
cargo test
cargo check
```

## Rewrite Docs

Historical rewrite planning and parity notes are kept in `docs/rust-rewrite/`.

## Backend Deployment (Railway)

Railway builds the Rust service from [`Dockerfile.rust`](Dockerfile.rust) (see
[`railway.json`](railway.json)). The root [`Dockerfile`](Dockerfile) is the
legacy Rails image and must not be used for the Axum app.

Backend deploys are handled by GitHub Actions in
`.github/workflows/deploy-backend-railway.yml`.

Trigger behavior:

- Push to `main` runs checks, builds `axum-app`, then deploys to Railway.
- Manual deploys are available via `workflow_dispatch`.

Required GitHub repository secrets:

- `RAILWAY_TOKEN` - Railway API token used by Railway CLI.
- `RAILWAY_PROJECT_ID` - target Railway project id.

Optional GitHub repository secrets:

- `RAILWAY_ENVIRONMENT` - Railway environment selector for deploy target.
- `RAILWAY_SERVICE` - Railway service selector within the project.

Runtime configuration is managed in Railway service variables and should align
with app config env names (uppercase underscore format), including:

- `HOST`
- `PORT`
- `DATABASE_URL`
- `JWT_SECRET`
- `LIQUIDSOAP_API_TOKEN`
- `RAILS_HOST`
- `RAILS_PROTOCOL`
- `ICECAST_PROTOCOL`
- `ICECAST_HOST`
- `ICECAST_PORT`
- `ICECAST_ADMIN_USERNAME`
- `ICECAST_ADMIN_PASSWORD`
- `ICECAST_PUBLIC_BASE_URL` (optional) — public base URL for browser `<audio>` mounts, e.g. `https://stream.example.com` when the site is HTTPS or when `ICECAST_HOST` is an internal Docker hostname. If unset, listeners use `ICECAST_PROTOCOL` + `ICECAST_HOST` + `ICECAST_PORT`.

Post-deploy health check endpoint: `/up`.

## Radio: Icecast + Liquidsoap (local)

1. Run the Axum app on the host (`cargo run -p axum-app`). Set `DATABASE_URL`, `LIQUIDSOAP_API_TOKEN`, and Icecast-related env vars so they match the compose stack (defaults use `hackme` for source/admin passwords).
2. Start Icecast and Liquidsoap (does not start the legacy `web` service unless you run compose without a profile filter):

   ```bash
   docker compose --profile radio up -d icecast liquidsoap
   ```

3. Generate Liquidsoap script from the database and restart the encoder:

   ```bash
   export LIQUIDSOAP_API_TOKEN=dev-liquidsoap-token   # match your app config
   curl -fsS -H "Authorization: Bearer $LIQUIDSOAP_API_TOKEN" \
     "http://127.0.0.1:4000/api/internal/liquidsoap_config" -o docker/radio/radio.liq
   docker compose --profile radio restart liquidsoap
   ```

4. Open `/radio` in the browser. Stream URLs target your Icecast mounts (default public base `http://localhost:8000` when `ICECAST_PUBLIC_BASE_URL` is unset).

Internal API additions:

- `GET /api/internal/liquidsoap_config` — Bearer `LIQUIDSOAP_API_TOKEN`; returns `text/plain` Liquidsoap script.
- `GET /api/internal/radio_stations/active` — same auth; JSON list of stations with `stream_url`, `mount_point`, etc.

YouTube-backed `next_track` URLs require `yt-dlp` wherever Liquidsoap runs; playlist/library tracks use your app’s `/tracks/:id/stream` URLs and work without it.

## Radio: Production Preflight and Test Runbook

Use this runbook before testing playlist import and live Icecast playback in production.

### 1) Runtime preflight checklist

- Axum runtime image includes `yt-dlp` and `ffmpeg` (required for playlist import and stream URL resolution).
- Railway service variables include:
  - `LIQUIDSOAP_API_TOKEN`
  - `RAILS_HOST`
  - `RAILS_PROTOCOL`
  - `ICECAST_PROTOCOL`
  - `ICECAST_HOST`
  - `ICECAST_PORT`
  - `ICECAST_ADMIN_USERNAME`
  - `ICECAST_ADMIN_PASSWORD`
  - `ICECAST_PUBLIC_BASE_URL` (required if browser-facing stream host differs from internal Icecast host)
- Icecast and Liquidsoap are running and Liquidsoap uses the same `LIQUIDSOAP_API_TOKEN`.
- At least one station exists in DB (`playlists` + `radio_stations`). A bootstrap script is provided at `ops/sql/bootstrap_radio_station.sql`.

### 2) Bootstrap a station (if needed)

Use the SQL script in `ops/sql/bootstrap_radio_station.sql` to create a playlist and station row, then note the created station id and mount.

If `DATABASE_URL` is SQLite (for example `sqlite://storage/development.sqlite3`), you can run:

```bash
sqlite3 "${DATABASE_URL#sqlite://}" < ops/sql/bootstrap_radio_station.sql
```

For Railway production, run the same SQL statements in your Railway database console/shell against the deployed database.

### 3) Import a YouTube playlist into a station

```bash
export API_BASE_URL="https://<your-production-host>"
export LIQUIDSOAP_API_TOKEN="<token>"
export STATION_ID="<station-id>"
export PLAYLIST_URL="https://www.youtube.com/watch?v=6aouLxiL4Cw&list=PLfAwSvgqO_M_aT7SOI4jdCCpJbZvDvOT-"

curl -fsS -X POST \
  -H "Authorization: Bearer $LIQUIDSOAP_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"url\":\"$PLAYLIST_URL\"}" \
  "$API_BASE_URL/api/internal/radio_stations/$STATION_ID/import_youtube"
```

Expected response shape:

```json
{"station_id":1,"playlist_id":"...","imported":42}
```

### 4) Regenerate Liquidsoap config from production DB

```bash
curl -fsS \
  -H "Authorization: Bearer $LIQUIDSOAP_API_TOKEN" \
  "$API_BASE_URL/api/internal/liquidsoap_config" \
  -o radio.liq
```

Deploy `radio.liq` to Liquidsoap and restart/reload Liquidsoap so it picks up the latest station list and mount settings.

### 5) Verify active stations and streaming

```bash
curl -fsS \
  -H "Authorization: Bearer $LIQUIDSOAP_API_TOKEN" \
  "$API_BASE_URL/api/internal/radio_stations/active"
```

Then verify:

- `/radio` and `/radio/<id>` load in browser and play the Icecast mount.
- `/up` returns healthy.
- `/api/internal/radio_stations/<id>/stats` shows listener data over time.

### 6) Failure triage

- `yt-dlp failed while fetching playlist` or `yt-dlp failed to resolve audio stream URL`:
  - confirm `yt-dlp` is installed in the Axum runtime
  - confirm outbound network access to YouTube
  - confirm playlist URL contains `list=...`
- Browser audio fails on HTTPS site:
  - set `ICECAST_PUBLIC_BASE_URL` to an HTTPS stream endpoint to avoid mixed content
- Unauthorized (`401`) from internal endpoints:
  - ensure Bearer token exactly matches `LIQUIDSOAP_API_TOKEN`
