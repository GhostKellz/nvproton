# Contributing to nvproton

Thanks for your interest in improving nvproton. This document covers the
development setup, coding standards, and how to submit changes.

> nvproton is experimental and under active development. Expect APIs and
> behavior to change.

## Prerequisites

- Rust 1.90+ (the project targets the **2024** edition)
- An NVIDIA GPU and driver for runtime testing (535+ works; 595+ recommended
  for DX12 descriptor-heap features)
- Optional, for full integration testing: Steam, Heroic, Lutris, MangoHud,
  Feral GameMode, and the `nv*` libraries (`nvshader`, `nvlatency`, `nvsync`)

## Building and Testing

```bash
# Build the CLI and library
cargo build

# Run the test suite
cargo test

# Lint with Clippy
cargo clippy --all-targets

# Format (required before submitting)
cargo fmt
```

## Coding Standards

- **Formatting:** code must be `cargo fmt`-clean.
- **Linting:** new code should not introduce `cargo clippy` warnings.
- **Edition:** Rust 2024. Use idiomatic, safe Rust; isolate `unsafe` to the FFI
  layer (`src/ffi/`) and document the safety invariants.
- **Errors:** use `anyhow` for application paths and `thiserror` for typed
  library errors. Avoid `unwrap()`/`expect()` outside of tests.
- **Comments:** explain *why*, not *what*. Do not embed version numbers in code
  or comments.
- **Scope:** keep changes minimal and focused. Avoid unrelated refactors in the
  same PR.

## Commit Messages

Follow Conventional Commits, matching the existing history:

```
feat: add VRR auto-detection for Wayland sessions
fix: handle missing Steam library folder gracefully
docs: document profile inheritance
chore: bump rusqlite to 0.40
```

## Pull Requests

1. Fork the repository and create a topic branch.
2. Make your change with accompanying tests where practical.
3. Ensure `cargo fmt`, `cargo clippy`, and `cargo test` all pass.
4. Open a PR describing the change and the motivation behind it.

## Documentation

User-facing documentation lives in [`docs/`](docs/README.md). Keep it accurate
and organized:

- One index only: `docs/README.md`.
- Group new pages under `getting-started/`, `reference/`, `guides/`, or
  `internals/`.
- Use lowercase, descriptive, kebab-case filenames (e.g. `game-detection.md`).
- Record notable changes in [`CHANGELOG.md`](CHANGELOG.md).

## Reporting Bugs and Requesting Features

Open an issue with clear reproduction steps, your environment (GPU, driver,
distro, Proton version), and relevant output from `nvproton status --verbose`.
For security issues, follow [SECURITY.md](SECURITY.md) instead.
