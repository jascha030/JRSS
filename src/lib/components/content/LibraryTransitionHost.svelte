<script lang="ts">
	import { toPng } from 'html-to-image';
	import LibraryPage from '$lib/components/content/LibraryPage.svelte';

	let {
		transitionKey,
		onReadyStateChange
	}: {
		transitionKey: string;
		onReadyStateChange?: (ready: boolean) => void;
	} = $props();

	let containerRef = $state<HTMLDivElement | null>(null);
	let snapshotUrl = $state<string | null>(null);
	let phase = $state<'idle' | 'capturing' | 'overlay' | 'fading'>('idle');
	let lastKey = $state('');
	let isIncomingReady = $state(false);
	let captureGeneration = $state(0);

	$effect.pre(() => {
		if (lastKey === '') {
			lastKey = transitionKey;
			return;
		}

		if (transitionKey === lastKey) return;

		if (containerRef) {
			captureSnapshot();
		}

		lastKey = transitionKey;
		isIncomingReady = false;
	});

	async function captureSnapshot() {
		if (!containerRef) return;

		const gen = ++captureGeneration;
		phase = 'capturing';

		try {
			const url = await toPng(containerRef, {
				pixelRatio: window.devicePixelRatio,
				cacheBust: true
			});
			if (gen !== captureGeneration) return;
			snapshotUrl = url;
			phase = 'overlay';
		} catch {
			if (gen !== captureGeneration) return;
			snapshotUrl = null;
			phase = 'idle';
		}
	}

	function handleReady(ready: boolean) {
		onReadyStateChange?.(ready);
		if (ready) {
			isIncomingReady = true;
		}
	}

	$effect(() => {
		if (phase === 'overlay' && isIncomingReady) {
			phase = 'fading';
			window.setTimeout(() => {
				snapshotUrl = null;
				phase = 'idle';
			}, 280);
		}
	});

	const isSnapshotVisible = $derived(
		snapshotUrl !== null && (phase === 'overlay' || phase === 'fading')
	);
</script>

<div bind:this={containerRef} class="relative flex min-h-0 flex-1 overflow-hidden">
	<LibraryPage onReadyStateChange={handleReady} />

	{#if isSnapshotVisible}
		<img
			src={snapshotUrl}
			alt=""
			aria-hidden="true"
			class="pointer-events-none absolute inset-0 z-50 h-full w-full object-fill transition-opacity duration-280 ease-out motion-reduce:transition-none"
			class:opacity-0={phase === 'fading'}
		/>
	{/if}
</div>
