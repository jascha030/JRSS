# AGENTS.md

## Snapshot

- Tauri v2 desktop RSS/podcast reader. SvelteKit 2 + Svelte 5 frontend, Rust backend, local SQLite at Tauri app-data `jrss.sqlite3`.

## Commands

- Install: `bun install` (repo uses `bun.lock`; do not switch package managers).
- Full app dev: `bun run tauri:dev`.
- Frontend-only dev: `bun run dev` serves `http://127.0.0.1:1420`.
- Typecheck: `bun run check`.
- Lint: `bun run lint` (`prettier --check . && eslint .`).
- Format: `bun run format`.
- Frontend build: `bun run build`.
- Native packaged build: `bun run tauri:build`.
- Rust-only check/test: `cargo check --manifest-path src-tauri/Cargo.toml` and `cargo test --manifest-path src-tauri/Cargo.toml`.
- `bun run tauri:dev` already starts `bun run dev` via `src-tauri/tauri.conf.json`; do not start a second Vite server.
- Outside Tauri, `invokeCommand` throws and some services deliberately return empty/default data; use `bun run tauri:dev` for backend features.
- No JS test runner, CI workflow, task runner, or pre-commit config is present. Rust unit tests exist in `feed_ingest.rs` and `reader_extract.rs`.
- Validation order: `bun run check` -> `bun run lint` -> `bun run build`; use `bun run tauri:build` when Rust/Tauri packaging matters.

## Architecture

- SPA only: `src/routes/+layout.ts` sets `prerender = true` and `ssr = false`; `adapter-static` uses `build/index.html` fallback.
- Client startup is `src/hooks.client.ts`, which calls `initializeApp()` from `src/lib/state/index.ts`.
- Main UI composition is `src/routes/+page.svelte`; most components import state/actions from the `src/lib/state/index.ts` facade.
- Domain state lives in `src/lib/state/*.svelte.ts`; service boundary is `src/lib/services/feedService.ts`.
- Frontend/backend path: state/actions -> `feedService.ts` -> `src/lib/services/tauriClient.ts` -> Tauri commands in `src-tauri/src/commands.rs`, registered in `src-tauri/src/lib.rs`.
- `src/lib/types/rss.ts` holds frontend domain types plus raw IPC shapes; `feedService.ts` maps flat Rust payloads into discriminated unions.
- SQLite is under `src-tauri/src/db/`: `schema.rs` creates/migrates tables, domain files (`feeds.rs`, `items.rs`, `stations.rs`, etc.) hold queries.
- Playback and queue are backend-owned on the Rust audio thread (`src-tauri/src/audio/`, `src-tauri/src/queue.rs`) and mirrored by `src/lib/state/playback.svelte.ts` via Tauri events/commands.

## Change Traps

- Schema changes must be additive/migrating in `src-tauri/src/db/schema.rs`; no external migration tool exists and user databases must keep working.
- Keep `src/lib/types/rss.ts` and `src-tauri/src/models.rs` in sync for IPC fields, serde casing, and discriminants.
- New Tauri commands need both `src-tauri/src/commands.rs` and registration in `src-tauri/src/lib.rs`, plus frontend wrapper/types if called from Svelte.
- Keep Vite/Tauri URLs aligned: `bun run dev` serves `127.0.0.1:1420`; `src-tauri/tauri.conf.json` points `devUrl` there and `frontendDist` to `../build`.
- Svelte 5 runes mode is forced for project files outside `node_modules` in `svelte.config.js`; state/store modules use `.svelte.ts`.
- Repo-local OpenCode config loads `@sveltejs/opencode`; use Svelte MCP/tools before editing or reviewing `.svelte` / `.svelte.ts` files.
- Prettier config: tabs, single quotes, no trailing commas, width 100, Svelte + Tailwind plugins, Tailwind stylesheet `src/routes/layout.css`.
- Rust toolchain is `edition = "2024"` with `rust-version = "1.87"`.

## High-Risk Files

- `src-tauri/src/db/schema.rs`: persistence schema/migrations.
- `src-tauri/src/db/items.rs`, `feeds.rs`, `stations.rs`: high-volume SQLite queries.
- `src/lib/types/rss.ts` + `src-tauri/src/models.rs`: cross-IPC contract.
- `src/lib/state/playback.svelte.ts` + `src-tauri/src/audio/` + `src-tauri/src/queue.rs`: playback lifecycle, Tauri events, queue persistence.

## Adding a New Setting

Adding a setting requires changes across the frontend type system, Rust backend, and UI configuration.

1. Add the field to `AppSettings` in `src/lib/types/rss.ts`
2. Add the field to `AppSettingsRecord` in `src-tauri/src/models.rs`
3. Add a migration in `src-tauri/src/db/schema.rs`
4. Add a `SettingEntry` to `APP_SETTINGS` in `src/lib/config/settings.ts` (TypeScript enforces key ↔ kind alignment)

### Adding a New Setting Kind

To add a new input type (e.g., a new control beyond toggle/number/select/segmented/color):

1. Define a new `*Def` interface in `src/lib/types/settings.ts` (must include `kind: '<name>'`)
2. Add the corresponding `*Entry` type
3. Add it to the `SettingEntry` union
4. Create the matching input component in `src/lib/components/settings/inputs/`
5. Add the dispatch branch in `src/lib/components/settings/SettingRow.svelte`

## Response Style

- Zero hedging. Do not start sentences or thoughts with "Wait", "Actually", "But wait", "Maybe", "Perhaps", "Hmm", "Hold on", "Let me think", "I think", "I believe", "It seems", "Probably", "Likely", or "Presumably". These words do not exist in your vocabulary.
- No self-correction loops. One conclusion, one action. If you started down a path, commit to it. Do not backtrack mid-sentence.
- No meta-commentary about your own process. Do not narrate what you are about to do, what you just did, or what you are considering. Just act.
- State the final answer directly. If uncertain, pick the most likely path and execute without disclaimers.
- If genuinely uncertain (not just hedging), ask the user for clarification before acting.
- Never say "I'll do that now", "Let me check", or any variation. Just do it.
- No filler sentences. No confirmations. No summarizing what was just done unless the result is ambiguous.
- Brevity reduces output token cost. Every wasted word is wasted money.
