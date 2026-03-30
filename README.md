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

- Today, the service uses mock seed data plus browser local storage persistence.
- Stores and UI call service functions like `listFeeds`, `listItems`, `markRead`, and `savePlayback`.
- Later, that same surface can be swapped for Tauri commands or another local backend with minimal UI changes.

## Static SPA Notes

- Uses `@sveltejs/adapter-static`
- Generates SPA-friendly output with `fallback: 'index.html'`
- Disables SSR globally in `src/routes/+layout.ts`
- Stays runnable in the browser with `bun run dev`
