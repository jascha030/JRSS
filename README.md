# JRSS | Jassie's Really Simple Syndication

[![Last commit](https://img.shields.io/github/last-commit/jascha030/JRSS)](https://github.com/jascha030/JRSS/commits)
[![License](https://img.shields.io/github/license/jascha030/JRSS)](LICENSE)

JRSS is a local-first desktop RSS reader and podcast player built with Tauri, SvelteKit, and Rust. It pulls RSS and Atom feeds into a local SQLite database, gives articles a focused reader mode, and handles podcast playback with queueing, history, and session restore.

![JRSS screenshot](img/main.png)

## Why JRSS

JRSS combines feed reading and podcast listening in one desktop app.

- Subscribe using a feed URL, Apple Podcasts URL, or Apple Podcasts ID
- Sync RSS and Atom feeds into a local SQLite database
- Read feed content directly or open extracted reader-mode article content
- Play podcast episodes with queue controls, history, and resume support
- Organize podcasts into stations for grouped playback
- Run background auto-refresh and manage local audio cache settings

## Why JRSS Is Useful

- Local-first: feeds, items, playback progress, and settings are stored on your machine
- One workflow for articles and podcasts instead of separate apps
- Reader mode improves long-form reading by extracting clean article content
- Podcast playback is stateful, with persistent queue and playback session restore
- Desktop-first UX includes a mini-player window and keyboard shortcuts

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/)
- Rust 1.87+ and Cargo
- Tauri system dependencies for your OS: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

JRSS uses `bun.lock`; use Bun for dependency installation and scripts.

### Install

```sh
bun install
```

### Run The Desktop App

```sh
bun run tauri:dev
```

This is the main development flow. Tauri starts the frontend for you at `http://127.0.0.1:1420`.

### Run Frontend-Only

```sh
bun run dev
```

Use this only for UI work. Backend-backed features are unavailable outside Tauri, and some services intentionally return empty/default data in that mode.

### Validate Changes

```sh
bun run check
bun run lint
bun run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Typical frontend validation order in this repo is:

```sh
bun run check && bun run lint && bun run build
```

### Example Workflow

1. Start JRSS with `bun run tauri:dev`.
2. Add a feed using a direct RSS/Atom URL or an Apple Podcasts link/ID.
3. Open an item in feed view or reader view.
4. Queue a podcast episode or create a station from selected feeds.

## Project Layout

- `src/`: SvelteKit frontend, UI components, state modules, and Tauri service wrappers
- `src-tauri/`: Rust backend, SQLite access layer, feed ingestion, reader extraction, and audio playback
- `img/`: screenshots and project imagery
- `.github/CODEOWNERS`: repository ownership

### Adding Settings

Adding a new user setting requires coordination across TypeScript types, Rust models, and the database schema:

1. Add the field to `AppSettings` in `src/lib/types/rss.ts`
2. Add the field to `AppSettingsRecord` in `src-tauri/src/models.rs`
3. Add a migration in `src-tauri/src/db/schema.rs`
4. Add a `SettingEntry` to `APP_SETTINGS` in `src/lib/config/settings.ts`

See [AGENTS.md](AGENTS.md#adding-a-new-setting) for detailed instructions on adding new setting kinds (custom input types).

High-level data flow:

```text
Svelte state/actions -> feedService.ts -> tauriClient.ts -> Tauri commands -> SQLite/audio services
```

## Data Storage

JRSS stores feeds, items, stations, playback state, playback session, and app settings in a local SQLite database named `jrss.sqlite3` in the Tauri app-data directory.

## Keyboard Shortcuts

The app defines desktop shortcuts for common actions, including:

- `CmdOrCtrl+N`: add feed
- `CmdOrCtrl+Shift+N`: create station
- `CmdOrCtrl+R`: refresh feed
- `CmdOrCtrl+F`: search feed
- `CmdOrCtrl+L`: go to feed
- `Space`: play/pause
- `CmdOrCtrl+Shift+Left` / `CmdOrCtrl+Shift+Right`: skip backward/forward

## Getting Help

- Open a bug report or feature request: <https://github.com/jascha030/JRSS/issues>
- Read contribution notes: [CONTRIBUTING.md](CONTRIBUTING.md)
- Tauri docs: <https://v2.tauri.app/>
- SvelteKit docs: <https://svelte.dev/docs/kit>

## Maintainers And Contributing

JRSS is maintained by Jascha van Aalst. Repository ownership is currently defined in [.github/CODEOWNERS](.github/CODEOWNERS) as `@jascha030`.

Contributions are welcome. For setup, validation, and change expectations, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

JRSS is published under the MIT license.
