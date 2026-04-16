# AGENTS.md

## What is JRSS

Local-first desktop RSS reader with podcast support. Tauri v2 app: SvelteKit frontend + Rust backend with SQLite storage.

## Commands

| Task          | Command               |
| ------------- | --------------------- |
| Install       | `bun install`         |
| Dev           | `bun run tauri:dev`   |
| Build         | `bun run tauri:build` |
| Typecheck     | `bun run check`       |
| Lint          | `bun run lint`        |
| Format        | `bun run format`      |
| Frontend only | `bun run dev`         |

**Validation order:** `bun run check` -> `bun run lint` -> `bun run build`

`bun run dev` runs the SvelteKit frontend only. Most features require the Rust backend; use `bun run tauri:dev` for full-stack dev. No test framework is configured.

## Architecture

SPA mode: `ssr = false` and `prerender = true` in `src/routes/+layout.ts`. All routing is client-side with `adapter-static` producing `build/index.html` as fallback.

**Frontend-backend boundary:** all data access flows through `src/lib/services/tauriClient.ts` (`invokeCommand` wrapper around Tauri `invoke`) to Rust commands registered in `src-tauri/src/lib.rs` and defined in `src-tauri/src/commands.rs`.

**Rust modules** (`src-tauri/src/`):

- `commands.rs` — Tauri command handlers (the IPC surface)
- `db.rs` — SQLite connection, schema migrations, all queries (~930 lines, the core of persistence)
- `models.rs` — Rust structs/enums serialized to/from the frontend
- `feed_ingest.rs` — RSS/Atom parsing and feed fetching
- `reader_extract.rs` — article readability extraction

**SQLite migrations** are embedded in `db.rs` (no external migration tool). Schema changes require updating the migration logic there.

**Type sync is manual.** `src/lib/types/rss.ts` and `src-tauri/src/models.rs` must mirror each other. When changing a Rust model's fields or serde attributes, update the TS types to match, and vice versa.

**Stores** use Svelte 5 runes (`.svelte.ts` files, e.g. `src/lib/stores/app.svelte.ts`).

## Conventions

- **TypeScript**: strict mode, no `any`, no non-null assertions, no type assertions
- **Svelte 5**: runes mode enforced project-wide (see `svelte.config.js`); do not remove runes config without migrating to Svelte 6
- **Formatting**: tabs, single quotes, no trailing commas, 100 char width (Prettier)
- **Tailwind CSS v4** via Vite plugin; Prettier class sorting uses `src/routes/layout.css` as the stylesheet reference
- **Rust edition 2021**, min rustc 1.77.2

## Handle with care

- `src-tauri/src/db.rs` — schema migrations and all queries; changes affect data persistence
- `src/lib/types/rss.ts` + `src-tauri/src/models.rs` — must stay in sync across the IPC boundary
- `src-tauri/gen/` — Tauri-generated code; do not edit manually
- `svelte.config.js` — runes mode logic; do not remove without migrating to Svelte 6
