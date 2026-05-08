import { invokeCommand, isTauriRuntime } from '$lib/services/tauri';
import type { PlaybackSession } from '$lib/types/playback';

export async function savePlaybackSession(session: PlaybackSession): Promise<void> {
	await invokeCommand('save_playback_session', { session });
}

export async function loadPlaybackSession(): Promise<PlaybackSession | null> {
	if (!isTauriRuntime()) {
		return null;
	}

	return invokeCommand<PlaybackSession | null>('load_playback_session');
}

export async function clearPlaybackSession(): Promise<void> {
	await invokeCommand('clear_playback_session');
}

export async function savePlaybackContext(
	context: { contextType: 'feed' | 'station'; id: string } | null
): Promise<void> {
	await invokeCommand('save_playback_context', { context });
}

export async function loadPlaybackContext(): Promise<{
	contextType: 'feed' | 'station';
	id: string;
} | null> {
	if (!isTauriRuntime()) {
		return null;
	}
	return invokeCommand<{ contextType: 'feed' | 'station'; id: string } | null>(
		'load_playback_context'
	);
}
