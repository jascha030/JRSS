# JRSS

Local-first RSS reader and podcast desktop MVP built with SvelteKit, Tauri, and SQLite.

## Current Architecture

- SvelteKit still owns the UI, routes, stores, and Tailwind styling.
- The frontend service layer in `src/lib/services/feedService.ts` is now a thin wrapper around Tauri commands.
- Native Tauri code in `src-tauri/src` handles:
  - feed fetching
  - RSS / Atom parsing
  - SQLite reads and writes
- The frontend no longer depends on browser CORS workarounds or `localStorage` persistence.

## Project Structure

```text
src/
  lib/
    components/   UI shell pieces like the sidebar, list view, and audio player
    services/     frontend service boundary and Tauri command wrapper
    stores/       app state for selection, lists, and playback
    types/        shared frontend domain types
    utils/        small formatting helpers
  routes/         SPA entry layout and page shell

src-tauri/
  src/
    commands.rs   Tauri invoke commands exposed to the frontend
    db.rs         SQLite schema and persistence helpers
    feed_ingest.rs native feed fetching and RSS/Atom parsing
    models.rs     serialized DTOs returned to the frontend
```

## SQLite

JRSS stores data in a local SQLite database named `jrss.sqlite3` inside the Tauri app data directory.

The MVP schema is created automatically on startup.

Tables:

- `feeds`
- `items`
- `playback_state`

Schema responsibilities:

- `feeds`: feed metadata and refresh timestamps
- `items`: normalized feed entries, read state, saved state, enclosure metadata
- `playback_state`: per-item playback position persisted separately from item metadata

The current MVP uses `CREATE TABLE IF NOT EXISTS` setup on open instead of a full migration framework.

## Frontend Contract

The frontend still uses the same service surface:

- `listFeeds()`
- `addFeed(url)`
- `refreshFeed(id)`
- `removeFeed(id)`
- `listItems(feedId?)`
- `markRead(itemId, read)`
- `savePlayback(itemId, positionSeconds)`

That keeps the stores and UI mostly unchanged while moving the real work native-side.

## Feed Ingestion

- Feed fetching now runs natively through Rust `reqwest`
- RSS parsing uses the `rss` crate
- Atom parsing uses the `atom_syndication` crate
- Podcast/audio detection happens from RSS enclosures and Atom enclosure links
- Refreshes upsert items without duplicating them
- Existing `read`, `saved`, and playback state are preserved across refreshes

## Run With Bun

Install JS dependencies:

```sh
bun install
```

Frontend-only browser shell:

```sh
bun run dev
```

Desktop app in development:

```sh
bun run tauri:dev
```

Useful checks:

```sh
bun run check
bun run lint
bun run build
```

Desktop build:

```sh
bun run tauri:build
```

## How Tauri Connects To The Existing Service Layer

1. Svelte components call store actions.
2. Stores call `src/lib/services/feedService.ts`.
3. That service calls `invoke(...)` through `@tauri-apps/api/core`.
4. Rust commands in `src-tauri/src/commands.rs` run native logic.
5. SQLite helpers in `src-tauri/src/db.rs` persist and query local data.

## Static Frontend Setup

- Uses `@sveltejs/adapter-static`
- Generates SPA-friendly output with `fallback: 'index.html'`
- Disables SSR globally in `src/routes/+layout.ts`
- Tauri loads the built frontend from `build/`

## Manual Setup Notes

- Rust and Cargo must be installed
- Platform build prerequisites for Tauri still apply
  - macOS: Xcode Command Line Tools
  - Linux: WebKitGTK and related native packages
  - Windows: MSVC build tools

## Remaining Rough Edges

- `bun run dev` only runs the frontend shell; native feed/database features require `bun run tauri:dev`
- The app is still single-device and offline-first only; no sync is implemented
- External article links still use the existing frontend behavior and can be refined later if needed
