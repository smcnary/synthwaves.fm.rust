# Rust Workspace

This directory contains the Axum rewrite implementation scaffold.

## Crates

- `axum-app` - web/API server
- `domain` - domain models and auth errors
- `infra` - config, db, auth helpers, Liquidsoap config generation
- `jobs` - recurring scheduler and worker jobs

## Run

```bash
cd rust
cargo run -p axum-app
```

Default address: `127.0.0.1:4000`
