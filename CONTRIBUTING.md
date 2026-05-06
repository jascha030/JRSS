# Contributing to JRSS

## Development Setup

### Prerequisites

- [Bun](https://bun.sh/)
- Rust 1.87+ and Cargo
- Tauri system dependencies for your OS: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Install

```sh
bun install
```

### Run Locally

```sh
bun run tauri:dev
```

Use `bun run dev` only for frontend-only work. Tauri-backed features are not available there.

## Validation

Frontend changes:

```sh
bun run check
bun run lint
bun run build
```

Rust/backend changes:

```sh
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Recommended order:

```sh
bun run check && bun run lint && bun run build
```

## Repository Notes

- Use Bun. This repo is locked with `bun.lock`.
- `bun run tauri:dev` already starts the frontend dev server. Do not start a second Vite server.
- Schema changes in `src-tauri/src/db/schema.rs` must be additive and migration-safe.
- Keep `src/lib/types/rss.ts` and `src-tauri/src/models.rs` in sync when changing IPC payloads.
- New Tauri commands need both Rust registration and frontend wrappers.

## Pull Requests

- Do not push branches to this repository directly. Fork the repo and open a PR from your fork.
- All PRs require approval from the maintainer (@jascha030) before merging.
- Keep changes focused and minimal.
- Explain user-visible behavior changes clearly.
- Run the relevant validation commands before opening a PR.
- For larger changes, open an issue first so the approach can be discussed.
