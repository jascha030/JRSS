# AGENTS.md

## Snapshot

- Tauri 2 desktop RSS/podcast reader. Frontend is SvelteKit 2 + Svelte 5 in SPA mode (`src/routes/+layout.ts`: `prerender = true`, `ssr = false`); backend is Rust; persistent data lives in Tauri app-data `jrss.sqlite3`.
- Use Bun only. Lockfile is `bun.lock`.

## Commands

- Install: `bun install`
- Full app dev: `bun run tauri:dev`
- Frontend-only dev: `bun run dev` on `http://127.0.0.1:1420`
- Frontend validation order: `bun run check && bun run lint && bun run build`
- Frontend tests: `bun run test`
- Single frontend test: `bun run test -- src/lib/services/feed.test.ts`
- Rust validation: `cargo check --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml`
- Single Rust test: `cargo test --manifest-path src-tauri/Cargo.toml <filter>`
- Format: `bun run format`
- `bun run tauri:dev` already starts Vite via `src-tauri/tauri.conf.json`; do not run a second dev server.
- Outside Tauri, `src/lib/services/tauri.ts` throws on `invokeCommand`; desktop-backed services either fail or return empty/default data. Use `bun run tauri:dev` for backend behavior.
- No CI workflows or pre-commit config are in this repo; local verification is the source of truth.

## Architecture

- Client boot is `src/hooks.client.ts` -> `initializeApp()` in `src/lib/state/index.ts`.
- `src/routes/+page.svelte` swaps between the main app and the mini-player window via `?window=mini`.
- Frontend state/actions live in `src/lib/state/*.svelte.ts`; import through `$lib/state` unless a domain file is required.
- Frontend services are split by domain in `src/lib/services/{feed,item,station,settings,tauri}.ts`; `tauri.ts` is the runtime gate.
- Backend entry is `src-tauri/src/lib.rs`; Tauri commands live in `src-tauri/src/commands.rs`; SQLite code lives in `src-tauri/src/db/`; playback and queue logic live in `src-tauri/src/audio/` and `src-tauri/src/queue.rs`.
- Frontend types are split across `src/lib/types/*.ts`, not a single shared `rss.ts`.

## Change Traps

- New Tauri command: add it in `src-tauri/src/commands.rs`, register it in `src-tauri/src/lib.rs`, then add the frontend service wrapper and any type updates.
- Schema changes must be additive in `src-tauri/src/db/schema.rs`; existing user databases must keep opening cleanly.
- Keep Vite/Tauri wiring aligned: Vite serves `127.0.0.1:1420`, `src-tauri/tauri.conf.json` points `devUrl` there, and packaged builds use `../build`.
- Keep frontend types, Rust models, and persisted fields aligned across `src/lib/types/*.ts`, `src-tauri/src/models.rs`, and `src-tauri/src/db/schema.rs`.
- Settings changes touch `src/lib/types/settings.ts`, `src/lib/constants/settings.ts`, `src-tauri/src/models.rs`, and `src-tauri/src/db/schema.rs`.
- New settings UI kinds also need a component in `src/lib/components/settings/inputs/` and a dispatch branch in `src/lib/components/settings/SettingRow.svelte`.
- Svelte runes mode is forced for project files in `svelte.config.js`; state modules use `.svelte.ts`.
- Tests run in Vitest + happy-dom (`vite.config.ts`, `vitest.setup.ts`); tests needing Tauri IPC should use `@tauri-apps/api/mocks` per test/file.
- Prettier uses tabs, single quotes, no trailing commas, `prettier-plugin-svelte`, and `prettier-plugin-tailwindcss` with `src/routes/layout.css`.
- Repo-local OpenCode config loads `@sveltejs/opencode` and `.opencode/instructions/*.md`; follow `.opencode/instructions/comment-policy.md` for comment/suppression rules and Rust `SAFETY:` comments.
- PR flow from `CONTRIBUTING.md`: push from a fork, not directly to this repo; maintainer approval required.
