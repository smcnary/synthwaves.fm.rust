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
