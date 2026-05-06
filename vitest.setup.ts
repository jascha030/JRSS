import '@testing-library/jest-dom/vitest';

// No global Tauri mocks needed:
// - `invokeCommand` (tauriClient.ts) throws "Tauri backend unavailable" when
//   `isTauriRuntime()` is false, which is always the case in happy-dom unless
//   `mockIPC` has been called for that test.
// - `listen` (@tauri-apps/api/event) is never called at module-init time in
//   this codebase (only inside `initAudioEventListeners`), so no safety-net
//   module mock is required.
//
// Tests that need Tauri IPC or events import `mockIPC` / `clearMocks` from
// `@tauri-apps/api/mocks` directly and set up per-test or per-file fixtures.
