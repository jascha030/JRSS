<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { MediaListItem, PlaybackState } from '$lib/types/rss';
	import { requestSeekTo, requestTogglePlayback } from '$lib/stores/app.svelte';
	import AudioSeekBar from '../player/AudioSeekBar.svelte';
	import AudioPlayerControls from '../player/AudioPlayerControls.svelte';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		feedTitle?: string;
		playbackState: PlaybackState | null;
		onClose: () => void;
	};

	let { item, imageUrl, feedTitle, playbackState }: Props = $props();

	function skip(deltaSeconds: number) {
		const current = playbackState?.positionSeconds ?? 0;
		const duration = playbackState?.durationSeconds ?? 0;
		const target = Math.max(0, Math.min(current + deltaSeconds, duration));
		requestSeekTo(target);
	}
</script>

<div class="flex h-full w-full flex-col overflow-hidden bg-surface-shell">
	<!-- Header with drag region and close button -->
	<div class="h-10 w-full px-3" data-tauri-drag-region></div>

	<!-- Main content with square album art -->
	<div class="flex flex-1 flex-col items-center justify-center px-6 pb-4">
		{#if item && playbackState}
			<!-- Square album art -->
			<div class="relative mb-4 aspect-square w-full max-w-50 overflow-hidden rounded-lg shadow-lg">
				{#if imageUrl}
					<img src={imageUrl} alt="" class="h-full w-full object-cover" draggable="false" />
				{:else}
					<div class="bg-surface-elevated flex h-full w-full items-center justify-center">
						<Icon icon="lucide:disc-3" class="size-16 text-fg-muted" />
					</div>
				{/if}
			</div>

			<!-- Track info -->
			<div class="mb-4 w-full text-center">
				<h2 class="line-clamp-1 text-lg font-semibold text-fg">{item.title}</h2>
				<p class="line-clamp-1 text-sm text-fg-muted">{feedTitle}</p>
			</div>

			<!-- Seek bar -->
			<div class="mb-4 w-full">
				<AudioSeekBar
					{playbackState}
					durationSeconds={playbackState.durationSeconds ||
						item.mediaEnclosure.durationSeconds ||
						0}
				/>
			</div>

			<!-- Controls -->
			<AudioPlayerControls
				durationSeconds={playbackState.durationSeconds || item.mediaEnclosure.durationSeconds || 0}
				isPlaying={playbackState.isPlaying}
				skipSeconds={15}
				onTogglePlayback={requestTogglePlayback}
				onSkip={skip}
			/>
		{:else}
			<div class="flex flex-1 flex-col items-center justify-center text-fg-muted">
				<Icon icon="lucide:disc-3" class="mb-4 size-16" />
				<p class="text-sm">Nothing playing</p>
			</div>
		{/if}
	</div>
</div>
