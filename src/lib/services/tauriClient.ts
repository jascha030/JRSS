import { invoke } from '@tauri-apps/api/core';

declare global {
	interface Window {
		__TAURI_INTERNALS__?: unknown;
	}
}

export function isTauriRuntime(): boolean {
	return typeof window !== 'undefined' && typeof window.__TAURI_INTERNALS__ !== 'undefined';
}

export async function invokeCommand<T>(
	command: string,
	args?: Record<string, unknown>
): Promise<T> {
	if (!isTauriRuntime()) {
		throw new Error('Tauri backend unavailable. Use `bun run tauri:dev` to run the desktop app.');
	}

	return invoke<T>(command, args);
}
