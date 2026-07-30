<script lang="ts">
	import { isAudioLoading } from '$lib/state';
	import Icon from '@iconify/svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';

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
		<IconButton
			icon="bi:skip-start-fill"
			variant="ghost"
			iconClass={sizes.skip}
			label="Previous episode"
			disabled={!canSkipPrevious}
			onclick={onPreviousEpisode}
			class={`${isAudioLoading() && 'invisible'}`}
		/>
	{/if}

	<IconButton
		icon="bi:rewind-fill"
		variant="ghost"
		iconClass={sizes.skip}
		label={`Back ${skipBackwardSeconds} seconds`}
		onclick={() => onSkip(-skipBackwardSeconds)}
		class={`${isAudioLoading() && 'invisible'}`}
	/>

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

	<IconButton
		icon="bi:fast-forward-fill"
		variant="ghost"
		iconClass={sizes.skip}
		label={`Forward ${skipForwardSeconds} seconds`}
		onclick={() => onSkip(skipForwardSeconds)}
		class={`${isAudioLoading() && 'invisible'}`}
	/>

	{#if onNextEpisode}
		<IconButton
			icon="bi:skip-end-fill"
			variant="ghost"
			iconClass={sizes.skip}
			label="Next episode"
			disabled={!canSkipNext}
			onclick={onNextEpisode}
			class={`${isAudioLoading() && 'invisible'}`}
		/>
	{/if}
</div>
