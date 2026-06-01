<script lang="ts">
	import { requestSeekTo } from '$lib/state';
	import type { PlaybackState } from '$lib/types/playback';
	import { formatDuration } from '$lib/utils/format';
	import RangeInput from '../ui/RangeInput.svelte';

	type Props = {
		playbackState: PlaybackState;
		durationSeconds: number;
		showTimeLabels?: boolean;
		class?: string;
	};

	let {
		playbackState,
		durationSeconds,
		showTimeLabels = true,
		class: className = ''
	}: Props = $props();

	let isSeeking = $state(false);
	let seekPosition = $state(0);
	let hoverTime = $state(0);
	let isHovering = $state(false);
	let cursorPercent = $state(0);

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

	function handleMouseMove(event: MouseEvent & { currentTarget: HTMLElement }) {
		const rect = event.currentTarget.getBoundingClientRect();
		const ratio = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width));
		hoverTime = ratio * durationSeconds;
		cursorPercent = ratio * 100;
	}
</script>

<div class={`flex min-w-0 flex-1 items-center gap-3 ${className}`}>
	{#if showTimeLabels}
		<span class="shrink-0 text-xs text-fg-muted tabular-nums">
			{formatDuration(displayPosition)}
		</span>
	{/if}

	<div
		class="relative min-w-0 flex-1"
		onmousemove={handleMouseMove}
		onmouseenter={() => {
			isHovering = true;
		}}
		onmouseleave={() => {
			isHovering = false;
		}}
		role="presentation"
	>
		{#if isHovering}
			<div
				class="pointer-events-none absolute -top-1 z-50 -translate-x-1/2 -translate-y-full card preset-filled-surface-950-50 p-1 text-xs tabular-nums"
				style="left: {cursorPercent}%"
			>
				{formatDuration(hoverTime)}
			</div>
		{/if}
		<RangeInput
			class="min-w-0 flex-1"
			value={displayPosition}
			max={durationSeconds}
			step={1}
			ariaLabel="Seek position"
			oninput={handleSeekInput}
			onchange={handleSeekChange}
		/>
	</div>

	{#if showTimeLabels}
		<span class="shrink-0 text-xs text-fg-muted tabular-nums">
			{formatDuration(durationSeconds)}
		</span>
	{/if}
</div>
