# JRSS | Jassie's Really Simple Syndication (Application)

JRSS local-first RSS reader with podcast support.

### This app is still under development 🚧

![podcast](https://github.com/jascha030/JRSS/blob/main/img/main.png?raw=true)

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

## TODO’s

- [x] Sidebar
- [x] Add feed
  - [x] RSS articles
  - [x] ATOM feed
  - [x] Podcast RSS feed
  - [x] Podcast RSS resolved from Apple Podcasts url
- [x] Virtualize stacked feed list
- [x] Feed Favicons
- [x] Podcast episode image
- [x] Media controls (Podcast)
- [x] Playing next
- [x] Ordering Playing next
- [x] Play Next + Add to queue on right mouse menu
- [x] Autoplay and populate queue
- [x] Play queue persistence
- [x] Podcast playing order newest/oldest → oldest/newest
- [x] Refactor feeditem type discrimination
- [x] Play controls in feed/item header
- [x] Toaster notifications
  - [x] Add feed notices/errors
  - [x] Reader errors
- [x] Scrolling episode titles in player
- [x] Non-media context menu
  - [x] Open in reader
  - [x] Open feed (if currently in all feeds/unread/podcasts etc.)
  - [x] Mark read
  - [x] Mark unread
  - [x] Copy url
- [x] Media Item context menu
  - [x] Play/pausePause (if playing)
  - [x] Play now (if not playing) (clears current queueu and rebuilds queue from episode)
  - [x] Play from start (if has progress and isn’t playing) (clears queue etc.)
  - [x] Open feed (if currently in all feeds/unread/podcasts etc.)
  - [x] Mark played (same as read but different name in front end)
  - [x] Mark unplayed (Same as unread different name in front end)
  - [x] Stop playback (if playing)
  - [x] Play next
  - [x] Add to queue
  - [x] Copy url
- [x] Feed context menu
  - [x] Open feed (if currently in all feeds/unread/podcasts etc.)
  - [x] Copy feed url
  - [x] Delete feed
- [ ] Global search
  - [ ] Algolia style pop-out grouped results etc.
  - [ ] Feeds
  - [ ] Items
- [ ] Article search (search content inside article when viewing article)
- [ ] Keyboard control
  - [x] Spacebar for play/pause
  - [ ] Add Feed: ⌘+K?
  - [ ] Global search: ⌘+⇧+f
  - [x] Local/Feed search: ⌘+f
  - [ ] Article search: /
- [x] Playlist/Station creation
  - [x] Add podcasts
  - [x] Select all/unplayed episodes
  - [x] Order oldest>newest/newest>oldest
  - [x] Station feed
- [ ] Pop out inputs
  - [ ] Search feed
  - [ ] Global search
  - [ ] Add feed
- [ ] Support Video
  - [ ] Player
- [ ] Cover view/Mini Player
- [ ] Responsive Pane UI (when window too small for reader and list view together)
