# Big-Bang Cutover Checklist

## Pre-cutover

- [ ] Run Axum service against production-like SQLite snapshot.
- [ ] Validate `/api/v1/auth/token` and `/api/v1/me` JWT auth flow.
- [ ] Validate internal radio endpoints with `LIQUIDSOAP_API_TOKEN`.
- [ ] Validate Subsonic ping for both `/rest/ping` and `/api/rest/ping`.
- [ ] Verify generated Liquidsoap config and mount formatting.
- [ ] Verify ffmpeg conversion jobs from `jobs::workers`.

## Cutover window

- [ ] Freeze uploads and background processing on Rails.
- [ ] Snapshot SQLite databases and storage volume.
- [ ] Deploy Axum image with mirrored environment values.
- [ ] Smoke-test `/up`, web root, auth endpoints, internal radio endpoints.
- [ ] Resume queues/scheduler on Axum side.

## Rollback

- [ ] Keep last-known-good Rails image available.
- [ ] Keep DB snapshot and pre-cutover Liquidsoap config archive.
- [ ] If rollback triggered, restore snapshot and restart Rails/Liquidsoap.
