<script lang="ts">
	import type { MediaListItem } from '$lib/types/item';
	import type { Snippet } from 'svelte';

	import { playbackSettings } from '$lib/state/settings.svelte';
	import { playbackState as globalPlaybackState } from '$lib/state/playback.svelte';
	import { usePlayerControls } from '$lib/hooks/usePlayerControls.svelte';
	import { appUi } from '$lib/hooks/useAppUi.svelte';
	import { navigateToCurrentAudioItem } from '$lib/navigation/audio-nav';
	import Info from './Info.svelte';
	import Controls from './Controls.svelte';
	import Volume from './Volume.svelte';
	import SeekBar from './SeekBar.svelte';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		controls?: Snippet;
	};

	let { item, imageUrl, controls }: Props = $props();

	const player = usePlayerControls(() => item);
	const playbackState = $derived(globalPlaybackState.currentPlaybackState);

	function handleShowCover() {
		appUi.playerMode = 'cover';
	}

	let playerBarEl: HTMLDivElement | null = $state(null);
	let playerBarWidth = $state(1600);

	$effect(() => {
		if (!playerBarEl) return;
		const ro = new ResizeObserver(([entry]) => {
			playerBarWidth = entry.contentRect.width;
		});
		ro.observe(playerBarEl);
		return () => ro.disconnect();
	});

	const isCompact = $derived(playerBarWidth < 800);
	const isVeryCompact = $derived(playerBarWidth < 600);
</script>

{#if item && playbackState}
	<div class="sticky bottom-0 z-10 border-t border-border bg-surface-glass-heavy p-4 backdrop-blur">
		<div
			bind:this={playerBarEl}
			class="mx-auto flex w-full max-w-full items-center justify-between gap-4 px-4 3xl:max-w-6xl 4xl:max-w-400"
		>
			<Info
				{item}
				{imageUrl}
				onNavigate={navigateToCurrentAudioItem}
				onShowCover={handleShowCover}
				compact={isCompact}
				class="min-w-0 shrink-0 basis-35 xl:basis-56 3xl:basis-auto"
			/>

			<div class="flex min-w-0 flex-1 items-center gap-4 2xl:px-4 3xl:px-8">
				<Controls
					isPlaying={playbackState.isPlaying}
					onTogglePlayback={player.handleTogglePlayback}
					onSkip={player.handleSkip}
					onPreviousEpisode={player.previousEpisode}
					onNextEpisode={player.nextEpisode}
					canSkipPrevious={player.canSkipPrevious}
					canSkipNext={player.canSkipNext}
					skipForwardSeconds={playbackSettings.skipForwardSeconds}
					skipBackwardSeconds={playbackSettings.skipBackwardSeconds}
					size={isVeryCompact ? 'sm' : 'md'}
					class="shrink-0"
				/>

				<SeekBar
					{playbackState}
					durationSeconds={player.durationForPlayer()}
					showTimeLabels={!isCompact}
					class="min-w-0 flex-1 px-4"
				/>
			</div>

			<div class="flex min-w-0 items-center gap-4">
				<Volume volume={playbackState.volume} showRange={!isCompact} class="shrink-0" />
				{#if controls}
					{@render controls()}
				{/if}
			</div>
		</div>
	</div>
{/if}
