import { invokeCommand, isTauriRuntime } from '$lib/services/tauri';

export async function loadPlaybackSession(): Promise<{
	currentItemId?: string;
	positionSeconds: number;
	durationSeconds: number;
	historyQueue: string[];
	manualQueue: string[];
	autoQueue: string[];
	playbackContext?: { contextType: 'feed' | 'station'; id: string };
} | null> {
	if (!isTauriRuntime()) {
		return null;
	}

	return invokeCommand('load_playback_session');
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
