import {
	playbackState,
	requestNextEpisode,
	requestPreviousEpisode,
	requestSeekTo,
	requestSetVolume,
	requestTogglePlayback
} from '$lib/state/playback.svelte';
import type { MediaListItem } from '$lib/types/item';

export function usePlayerControls(getItem: () => MediaListItem | null) {
	const canSkipPrevious = $derived(playbackState.playbackHistory.length > 0);
	const canSkipNext = $derived(
		playbackState.manualQueue.length > 0 || playbackState.autoQueue.length > 0
	);

	function durationForPlayer(): number {
		const state = playbackState.currentPlaybackState;
		if (state && state.durationSeconds > 0) {
			return state.durationSeconds;
		}
		return getItem()?.mediaEnclosure.durationSeconds ?? 0;
	}

	function handleSkip(deltaSeconds: number) {
		const state = playbackState.currentPlaybackState;
		const item = getItem();
		const dur =
			state?.fileDurationSeconds ??
			state?.durationSeconds ??
			item?.mediaEnclosure.durationSeconds ??
			0;
		const current = state?.positionSeconds ?? 0;
		const target = Math.max(0, Math.min(current + deltaSeconds, dur));
		requestSeekTo(target);
	}

	function handleTogglePlayback() {
		requestTogglePlayback();
	}

	function handleAdjustVolume(delta: number) {
		const state = playbackState.currentPlaybackState;
		if (!state) return;
		const newVolume = Math.max(0, Math.min(1, state.volume + delta));
		requestSetVolume(newVolume);
	}

	function nextEpisode() {
		requestNextEpisode();
	}

	function previousEpisode() {
		requestPreviousEpisode();
	}

	return {
		get canSkipPrevious() {
			return canSkipPrevious;
		},
		get canSkipNext() {
			return canSkipNext;
		},
		durationForPlayer,
		handleSkip,
		handleTogglePlayback,
		handleAdjustVolume,
		nextEpisode,
		previousEpisode
	};
}
