# JRSS | Jassie's Really Simple Syndication (Application)

JRSS local-first RSS reader with podcast support.

### This app is still under development 🚧

![podcast](https://github.com/jascha030/JRSS/blob/main/img/scr1_dark.png?raw=true)
![article](https://github.com/jascha030/JRSS/blob/main/img/scr2.png?raw=true)

## Why JRSS

- One place for articles and podcasts
- Fast, simple reading experience
- Offline-friendly by design

## Features

- Subscribe to RSS and Atom feeds
- Read full articles in a focused reader view
- Play audio for RSS podcast feeds
- Automatically resolve rss feed from Apple Podcast link using apple.itunes.com lookup endpoint
- Pick up playback where you left off

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
```

## Data

JRSS stores its data locally on your machine.
