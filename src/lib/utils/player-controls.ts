import {
	requestNextEpisode,
	requestPreviousEpisode,
	requestSeekTo,
	requestSetVolume,
	requestTogglePlayback
} from '$lib/stores/app.svelte';
import type { PlaybackState } from '$lib/types/rss';

export const DEFAULT_SKIP_SECONDS = 15;
export const VOLUME_STEP = 0.1;

export function skip(
	playbackState: PlaybackState | null,
	itemDurationSeconds: number | undefined,
	deltaSeconds: number
) {
	const current = playbackState?.positionSeconds ?? 0;
	const dur = playbackState?.durationSeconds ?? itemDurationSeconds ?? 0;
	const target = Math.max(0, Math.min(current + deltaSeconds, dur));
	requestSeekTo(target);
}

export function togglePlayback() {
	requestTogglePlayback();
}

export function adjustVolume(playbackState: PlaybackState | null, delta: number) {
	if (!playbackState) return;
	const newVolume = Math.max(0, Math.min(1, playbackState.volume + delta));
	requestSetVolume(newVolume);
}

export function nextEpisode() {
	requestNextEpisode();
}

export function previousEpisode() {
	requestPreviousEpisode();
}
