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
</script>

{#if item && playbackState}
	<div
		class="sticky bottom-0 z-10 border-t border-border bg-surface-glass-heavy px-4 py-3 backdrop-blur"
	>
		<div class="mx-auto flex max-w-6xl items-center gap-6 4xl:max-w-400">
			<Info
				{item}
				{imageUrl}
				onNavigate={navigateToCurrentAudioItem}
				onShowCover={handleShowCover}
				class="min-w-0 shrink-0 basis-56"
			/>

			<div class="flex min-w-0 flex-1 items-center gap-4">
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
					class="shrink-0"
				/>

				<SeekBar
					{playbackState}
					durationSeconds={player.durationForPlayer()}
					class="min-w-0 flex-1"
				/>

				<Volume volume={playbackState.volume} class="shrink-0" />
			</div>

			{#if controls}
				<div class="relative ml-1 shrink-0">
					{@render controls()}
				</div>
			{/if}
		</div>
	</div>
{/if}
