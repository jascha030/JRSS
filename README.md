# JRSS

Minimal SvelteKit MVP foundation for a local-first RSS reader and podcast app.

## Run With Bun

```sh
bun install
bun run dev
```

Useful scripts:

```sh
bun run check
bun run lint
bun run build
```

## Project Structure

```text
src/
  lib/
    components/   UI shell pieces like the sidebar, list view, and audio player
    services/     frontend service boundary for feeds, items, and playback persistence
    stores/       app state for selection, lists, and playback
    types/        domain types for feeds, items, enclosures, and playback
    utils/        small formatting helpers
  routes/         SPA entry layout and starter page
```

## Service Layer And Future Tauri Migration

The app keeps feed/item/playback logic behind `src/lib/services/feedService.ts` instead of building around SvelteKit server routes.

- Today, the service orchestrates browser feed fetching/parsing and stores normalized data in local storage through `src/lib/services/feedRepository.ts`.
- Stores and UI call service functions like `listFeeds`, `addFeed`, `refreshFeed`, `listItems`, `markRead`, and `savePlayback`.
- Later, the repository can move to SQLite and the fetch/parse call can move to Tauri commands without changing most UI code.

## Feed Ingestion Notes

- Adding a feed now immediately fetches and parses real RSS or Atom XML.
- Parsed feed metadata and items are mapped into the app's `Feed` and `FeedItem` types and persisted locally.
- Item updates are merged by deterministic IDs so refreshes do not create duplicates and existing read/saved/playback state is preserved.

## Browser CORS Limitations

- This MVP still runs fully in the browser, so some feeds will fail direct fetches if they do not send permissive CORS headers.
- The parser tries a direct browser fetch first, then falls back to a temporary public proxy via `allorigins.win`.
- That proxy is only a stopgap for development. It can still fail, rate limit, or be unavailable.
- A future Tauri or local backend path removes most of these browser-only CORS limits.

## Static SPA Notes

- Uses `@sveltejs/adapter-static`
- Generates SPA-friendly output with `fallback: 'index.html'`
- Disables SSR globally in `src/routes/+layout.ts`
- Stays runnable in the browser with `bun run dev`
