<script lang="ts">
	import { isAudioLoading } from '$lib/state';
	import Icon from '@iconify/svelte';

	type Props = {
		isPlaying: boolean;
		onTogglePlayback: () => void;
		onSkip: (deltaSeconds: number) => void;
		onPreviousEpisode?: () => void;
		onNextEpisode?: () => void;
		canSkipPrevious?: boolean;
		canSkipNext?: boolean;
		skipForwardSeconds?: number;
		skipBackwardSeconds?: number;
		size?: 'sm' | 'md' | 'lg';
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
		skipForwardSeconds = 15,
		skipBackwardSeconds = 15,
		size = 'md',
		class: className = ''
	}: Props = $props();

	const sizes = $derived.by(() => {
		const map = {
			sm: { gap: 'gap-1', skip: 'size-4', playBtn: 'size-7', playIcon: 'size-7' },
			md: { gap: 'gap-2', skip: 'size-5', playBtn: 'size-9', playIcon: 'size-9' },
			lg: { gap: 'gap-2', skip: 'size-6', playBtn: 'size-10', playIcon: 'size-10' }
		};
		return map[size];
	});
</script>

<div class={`flex items-center justify-center ${sizes.gap} ${className}`}>
	{#if onPreviousEpisode}
		<button
			class="text-fg-subtle transition-colors hover:text-white disabled:opacity-30"
			type="button"
			aria-label="Previous episode"
			disabled={!canSkipPrevious}
			onclick={onPreviousEpisode}
		>
			<Icon icon="bi:skip-start-fill" class={sizes.skip} />
		</button>
	{/if}

	<button
		class="text-fg-subtle transition-colors hover:text-white"
		type="button"
		aria-label={`Back ${skipBackwardSeconds} seconds`}
		onclick={() => onSkip(-skipBackwardSeconds)}
	>
		<Icon icon="bi:rewind-fill" class={sizes.skip} />
	</button>

	<button
		class={`${sizes.playBtn} rounded-xl text-fg-subtle transition-colors hover:text-white`}
		type="button"
		onclick={onTogglePlayback}
		disabled={isAudioLoading()}
		aria-label={isPlaying ? 'Pause' : 'Play'}
		aria-pressed={isPlaying}
	>
		{#if isAudioLoading()}
			{#key isAudioLoading()}
				<Icon icon="lucide:loader-2" class={`${sizes.playIcon} animate-spin`} />
			{/key}
		{:else if isPlaying}
			<Icon icon="bi:pause-fill" class={sizes.playIcon} />
		{:else}
			<Icon icon="bi:play-fill" class={sizes.playIcon} />
		{/if}
	</button>

	<button
		class="text-fg-subtle transition-colors hover:text-white"
		type="button"
		aria-label={`Forward ${skipForwardSeconds} seconds`}
		onclick={() => onSkip(skipForwardSeconds)}
	>
		<Icon icon="bi:fast-forward-fill" class={sizes.skip} />
	</button>

	{#if onNextEpisode}
		<button
			class="text-fg-subtle transition-colors hover:text-white disabled:opacity-30"
			type="button"
			aria-label="Next episode"
			disabled={!canSkipNext}
			onclick={onNextEpisode}
		>
			<Icon icon="bi:skip-end-fill" class={sizes.skip} />
		</button>
	{/if}
</div>
