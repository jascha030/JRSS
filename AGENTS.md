# AGENTS.md

## What is JRSS

Local-first desktop RSS reader with podcast support. Tauri v2 app: SvelteKit frontend + Rust backend with SQLite storage.

## Structure

- `src/` — SvelteKit frontend (adapter-static, Svelte 5 runes mode)
  - `lib/components/` — UI components
  - `lib/services/` — frontend service layer
  - `lib/stores/` — Svelte stores
  - `lib/types/` — TypeScript types
  - `lib/utils/` — utility functions
- `src-tauri/` — Rust backend (Tauri v2, rusqlite, feed parsing)
- `static/` — static assets

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

## Validation order

1. `bun run check` (typecheck)
2. `bun run lint` (prettier + eslint)
3. `bun run build` (vite build)

## Conventions

- **TypeScript**: strict mode, no `any`, no non-null assertions, no type assertions
- **Svelte 5**: runes mode enforced project-wide (see `svelte.config.js`)
- **Formatting**: tabs, single quotes, no trailing commas, 100 char width (Prettier)
- **Tailwind CSS v4** via Vite plugin
- **No test framework** currently configured
- **Rust edition 2021**, min rustc 1.77.2

## Handle with care

- `src-tauri/src/` — Rust commands and SQLite schema; changes affect data persistence
- `src/lib/types/` — shared type definitions between services and components
- `svelte.config.js` — runes mode logic; do not remove without migrating to Svelte 6
