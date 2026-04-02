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

Post-deploy health check endpoint: `/up`.
