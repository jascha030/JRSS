# AGENTS.md

## Snapshot

- Tauri v2 desktop RSS/podcast reader. SvelteKit 2 + Svelte 5 frontend, Rust backend, SQLite stored in Tauri app data as `jrss.sqlite3`.

## Commands

- Install: `bun install`
- Full app dev: `bun run tauri:dev`
- Frontend-only dev: `bun run dev`
- Typecheck: `bun run check`
- Lint: `bun run lint` (`prettier --check . && eslint .`)
- Format: `bun run format`
- Frontend build: `bun run build`
- Native packaged build: `bun run tauri:build`
- `bun run tauri:dev` already launches `bun run dev` via `src-tauri/tauri.conf.json`; do not start a second frontend server.
- `bun run dev` serves `http://127.0.0.1:1420` only. Many features are Tauri-only; outside Tauri, `invokeCommand` throws and some services fall back to empty data.
- No test runner, CI workflow, or pre-commit config is present.
- Preferred validation order from repo instructions: `bun run check` -> `bun run lint` -> `bun run build`. Use `bun run tauri:build` when validating Rust/Tauri changes.

## Architecture

- SPA only: `src/routes/+layout.ts` sets `prerender = true` and `ssr = false`; `adapter-static` uses `build/index.html` as fallback.
- Main UI entrypoint is `src/routes/+page.svelte`; startup runs `initializeApp()` from `src/lib/stores/app.svelte`.
- Frontend state/actions live in `src/lib/state/*.svelte.ts`; `src/lib/stores/app.svelte` is the facade most components import.
- Frontend/backend path is `state/*.svelte.ts` -> `src/lib/services/feedService.ts` -> `src/lib/services/tauriClient.ts` -> Tauri commands in `src-tauri/src/commands.rs`, registered in `src-tauri/src/lib.rs`.
- `src/lib/types/rss.ts` holds frontend domain types plus raw IPC shapes. `feedService.ts` maps flat Rust payloads into discriminated unions.
- Playback and queue are backend-owned in Rust (`src-tauri/src/audio.rs`, `src-tauri/src/queue.rs`) so playback survives UI/window teardown; frontend `src/lib/state/playback.svelte.ts` mirrors that state via Tauri events/commands.

## Change Traps

- SQLite schema and migrations are all in `src-tauri/src/db.rs`. There is no external migration tool; schema changes must keep existing user databases working.
- Keep `src/lib/types/rss.ts` and `src-tauri/src/models.rs` in sync when changing IPC fields or serde casing.
- Keep Vite and Tauri config aligned: `bun run dev` serves `127.0.0.1:1420`, `src-tauri/tauri.conf.json` points `devUrl` there, and `frontendDist` expects `../build`.
- Svelte 5 runes mode is forced for project files outside `node_modules` in `svelte.config.js`; state/store modules use `.svelte.ts`.
- Formatting is enforced by Prettier: tabs, single quotes, no trailing commas, width 100. Tailwind class sorting uses `src/routes/layout.css`.
- Rust toolchain is `edition = "2024"` with `rust-version = "1.87"`.

## High-Risk Files

- `src-tauri/src/db.rs`: persistence, migrations, most queries
- `src/lib/types/rss.ts` + `src-tauri/src/models.rs`: cross-IPC contract
- `src/lib/state/playback.svelte.ts` + `src-tauri/src/audio.rs` + `src-tauri/src/queue.rs`: playback lifecycle and queue behavior
