import { invokeCommand } from '$lib/services/tauri';
import type { BackendPlaybackState } from '$lib/types/playback';

export async function audioToggle(): Promise<void> {
	await invokeCommand('audio_toggle');
}

export async function audioStop(): Promise<void> {
	await invokeCommand('audio_stop');
}

export async function audioSeek(positionSeconds: number): Promise<void> {
	await invokeCommand('audio_seek', { positionSeconds });
}

export async function audioSetVolume(volume: number): Promise<void> {
	await invokeCommand('audio_set_volume', { volume });
}

export async function audioGetState(): Promise<BackendPlaybackState | null> {
	return invokeCommand<BackendPlaybackState | null>('audio_get_state');
}
