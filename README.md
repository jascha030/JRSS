# JRSS | Jassie's Really Simple Syndication

JRSS is a local-first desktop RSS reader with podcast support.

This app is still under development.

![podcast](https://github.com/jascha030/JRSS/blob/main/img/main.png?raw=true)

## Why JRSS

- One place for articles and podcasts
- Fast, simple reading experience
- Offline-friendly by design

## Features

- Subscribe to RSS and Atom feeds
- Read full articles in a focused reader view
- Play audio for RSS podcast feeds
- Resolve RSS feeds from Apple Podcasts links or IDs
- Create podcast stations from selected feeds
- Queue episodes with play next, add to queue, and persistent playback state
- Pick up playback where you left off

## Tech Stack

- Tauri v2
- SvelteKit 2 + Svelte 5
- Rust backend
- Local SQLite database

## Run locally

### Requirements

- [Bun](https://bun.sh/)
- Rust/Cargo

### Install

```sh
bun install
```

### Start the app

```sh
bun run tauri:dev
```

### Optional commands

```sh
bun run dev
bun run check
bun run lint
bun run build
bun run tauri:build
cargo test --manifest-path src-tauri/Cargo.toml
```

## Data

JRSS stores feeds, articles, playback progress, and settings locally in a SQLite database in the Tauri app data directory.
