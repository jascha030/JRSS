<script lang="ts">
	import { requestSeekTo } from '$lib/stores/app.svelte';
	import type { PlaybackState } from '$lib/types/rss';
	import { formatDuration } from '$lib/utils/format';
	import RangeInput from '../ui/RangeInput.svelte';

	type Props = {
		playbackState: PlaybackState;
		durationSeconds: number;
		class?: string;
	};

	let { playbackState, durationSeconds, class: className = '' }: Props = $props();

	let isSeeking = $state(false);
	let seekPosition = $state(0);

	let displayPosition = $derived(isSeeking ? seekPosition : playbackState.positionSeconds);

	function handleSeekInput(event: Event & { currentTarget: HTMLInputElement }) {
		isSeeking = true;
		seekPosition = Number(event.currentTarget.value);
	}

	function handleSeekChange(event: Event & { currentTarget: HTMLInputElement }) {
		const position = Number(event.currentTarget.value);
		requestSeekTo(position);
		isSeeking = false;
	}
</script>

<div class={`flex min-w-0 flex-1 items-center gap-3 ${className}`}>
	<span class="shrink-0 text-xs text-fg-muted tabular-nums">
		{formatDuration(displayPosition)}
	</span>

	<RangeInput
		class="min-w-0 flex-1"
		value={displayPosition}
		max={durationSeconds}
		step={1}
		ariaLabel="Seek position"
		oninput={handleSeekInput}
		onchange={handleSeekChange}
	/>

	<span class="shrink-0 text-xs text-fg-muted tabular-nums">
		{formatDuration(durationSeconds)}
	</span>
</div>
