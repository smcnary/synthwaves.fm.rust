# Contributing

Thanks for your interest in contributing to synthwaves.fm!

## Setup

**Requirements:** Rust (stable), SQLite3, ffmpeg

```bash
cd rust
cargo check
```

This verifies the workspace compiles and fetches dependencies.

## Development

```bash
cd rust
cargo run -p axum-app  # Start dev server
cargo test             # Run tests
cargo check            # Compile checks
```

## Submitting Changes

1. Open an issue first for large changes so we can discuss the approach
2. Fork the repo and create a branch from `main`
3. Include tests for new behavior
4. Make sure `cargo test` and `cargo check` pass
5. Open a pull request

## Reporting Bugs

Open an issue with steps to reproduce, expected behavior, and actual behavior.
