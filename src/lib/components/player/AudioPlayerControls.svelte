<script lang="ts">
	import { isAudioLoading } from '$lib/stores/app.svelte';
	import Icon from '@iconify/svelte';

	type Props = {
		durationSeconds: number;
		isPlaying: boolean;
		onTogglePlayback: () => void;
		onSkip: (deltaSeconds: number) => void;
		onPreviousEpisode?: () => void;
		onNextEpisode?: () => void;
		canSkipPrevious?: boolean;
		canSkipNext?: boolean;
		skipSeconds?: number;
		class?: string;
	};

	let {
		isPlaying,
		onTogglePlayback,
		onSkip,
		onPreviousEpisode,
		onNextEpisode,
		canSkipPrevious = false,
		canSkipNext = false,
		skipSeconds = 15,
		class: className = ''
	}: Props = $props();
</script>

<div class={`flex items-center justify-center gap-2 ${className}`}>
	{#if onPreviousEpisode}
		<button
			class="text-fg-subtle hover:text-white disabled:opacity-30"
			type="button"
			aria-label="Previous episode"
			disabled={!canSkipPrevious}
			onclick={onPreviousEpisode}
		>
			<Icon icon="bi:skip-start-fill" class="size-5" />
		</button>
	{/if}

	<button
		class="text-fg-subtle hover:text-white"
		type="button"
		aria-label={`Back ${skipSeconds} seconds`}
		onclick={() => onSkip(-skipSeconds)}
	>
		<Icon icon="bi:rewind-fill" class="size-5" />
	</button>

	<button
		class="preset-filled-accent btn-icon size-5 rounded-xl"
		type="button"
		onclick={onTogglePlayback}
		disabled={isAudioLoading()}
		aria-label={isPlaying ? 'Pause' : 'Play'}
		aria-pressed={isPlaying}
	>
		{#if isAudioLoading()}
			{#key isAudioLoading()}
				<Icon icon="lucide:loader-2" class="size-5 animate-spin" />
			{/key}
		{:else if isPlaying}
			<Icon icon="bi:pause-fill" class="size-5" />
		{:else}
			<Icon icon="bi:play-fill" class="size-5" />
		{/if}
	</button>

	<button
		class="text-fg-subtle hover:text-white"
		type="button"
		aria-label={`Forward ${skipSeconds} seconds`}
		onclick={() => onSkip(skipSeconds)}
	>
		<Icon icon="bi:fast-forward-fill" class="size-5" />
	</button>

	{#if onNextEpisode}
		<button
			class="text-fg-subtle hover:text-white disabled:opacity-30"
			type="button"
			aria-label="Next episode"
			disabled={!canSkipNext}
			onclick={onNextEpisode}
		>
			<Icon icon="bi:skip-end-fill" class="size-5" />
		</button>
	{/if}
</div>
