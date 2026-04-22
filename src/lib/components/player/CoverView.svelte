<script lang="ts">
	import type { Feed, MediaListItem, PlaybackState } from '$lib/types/rss';
	import type { Snippet } from 'svelte';
	import { requestSeekTo, requestTogglePlayback } from '$lib/stores/app.svelte';
	import Icon from '@iconify/svelte';
	import AudioPlayerControls from './AudioPlayerControls.svelte';
	import AudioPlayerInfo from './AudioPlayerInfo.svelte';
	import AudioSeekBar from './AudioSeekBar.svelte';
	import AudioPlayerVolume from './AudioPlayerVolume.svelte';
	import QueueList from './QueueList.svelte';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
		onNavigateToItem?: () => void;
		onClose?: () => void;
		controls?: Snippet;
		class?: string;
		queueItems?: MediaListItem[];
		manualQueueLength?: number;
		feeds?: Feed[];
		onRemoveQueueItem?: (itemId: string) => void;
		onMoveQueueItemUp?: (itemId: string) => void;
		onMoveQueueItemDown?: (itemId: string) => void;
		onClearQueue?: () => void;
	};

	const SKIP_SECONDS = 15;

	let {
		item,
		imageUrl,
		playbackState,
		onNavigateToItem,
		onClose,
		controls,
		class: className = '',
		queueItems = [],
		manualQueueLength = 0,
		feeds = [],
		onRemoveQueueItem,
		onMoveQueueItemUp,
		onMoveQueueItemDown,
		onClearQueue
	}: Props = $props();

	function durationForPlayer(): number {
		if (playbackState && playbackState.durationSeconds > 0) {
			return playbackState.durationSeconds;
		}
		return item?.mediaEnclosure.durationSeconds ?? 0;
	}

	function skip(deltaSeconds: number) {
		const current = playbackState?.positionSeconds ?? 0;
		const dur = durationForPlayer();
		const target = Math.max(0, Math.min(current + deltaSeconds, dur));
		requestSeekTo(target);
	}

	function togglePlayback() {
		requestTogglePlayback();
	}

	$effect(() => {
		if (!item) return;

		function handleKeydown(event: KeyboardEvent) {
			if (event.code !== 'Space') return;

			const tag = event.target instanceof Element ? event.target.tagName : '';
			if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || tag === 'BUTTON') return;
			if (event.target instanceof HTMLElement && event.target.isContentEditable) return;

			event.preventDefault();
			togglePlayback();
		}

		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
	});

	$effect(() => {
		if (!item || !('mediaSession' in navigator)) {
			return;
		}

		const back = () => skip(-SKIP_SECONDS);
		const forward = () => skip(SKIP_SECONDS);

		navigator.mediaSession.setActionHandler('previoustrack', back);
		navigator.mediaSession.setActionHandler('nexttrack', forward);
		navigator.mediaSession.setActionHandler('seekbackward', back);
		navigator.mediaSession.setActionHandler('seekforward', forward);

		return () => {
			navigator.mediaSession.setActionHandler('previoustrack', null);
			navigator.mediaSession.setActionHandler('nexttrack', null);
			navigator.mediaSession.setActionHandler('seekbackward', null);
			navigator.mediaSession.setActionHandler('seekforward', null);
		};
	});
</script>

{#if item && playbackState}
	<div class={`fixed inset-0 bg-black px-12 ${className}`}>
		{#if onClose}
			<button
				type="button"
				class="absolute top-6 left-6 flex size-12 items-center justify-center rounded-full bg-white/10 text-white/80 transition-colors hover:bg-white/20 hover:text-white"
				aria-label="Close cover view"
				onclick={onClose}
			>
				<Icon icon="lucide:x" class="size-6" />
			</button>
		{/if}

		<div
			class="grid h-full grid-cols-[minmax(0,1.6fr)_minmax(20rem,0.9fr)] items-center justify-center gap-8 p-8"
		>
			<div class="min-w-0">
				<div class="mx-auto flex w-full max-w-6xl flex-col p-12">
					{#if imageUrl}
						<img
							src={imageUrl}
							alt=""
							class="mx-auto aspect-square w-full max-w-3xl rounded-lg object-cover shadow-sm select-none"
						/>
					{:else}
						<div
							class="mx-auto grid aspect-square w-full max-w-3xl place-items-center rounded-lg bg-white/5 text-white/40"
						>
							<Icon icon="lucide:disc-3" class="size-16" />
						</div>
					{/if}
				</div>

				<div class="mx-auto flex w-full max-w-6xl flex-col gap-6">
					<div
						class="mx-auto grid w-full max-w-6xl grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-6 4xl:max-w-400"
					>
						<div class="min-w-0">
							<AudioPlayerInfo {item} showCover={false} onNavigate={onNavigateToItem} />
						</div>

						<AudioPlayerControls
							durationSeconds={durationForPlayer()}
							isPlaying={playbackState.isPlaying}
							onTogglePlayback={togglePlayback}
							onSkip={skip}
							skipSeconds={SKIP_SECONDS}
						/>

						<div class="flex min-w-0 items-center justify-end gap-2">
							<AudioPlayerVolume volume={playbackState.volume} />

							{#if controls}
								<div class="relative ml-1 shrink-0">
									{@render controls()}
								</div>
							{/if}
						</div>
					</div>

					<div class="flex min-w-0 flex-row gap-4">
						<AudioSeekBar {playbackState} durationSeconds={durationForPlayer()} class="mt-1" />
					</div>
				</div>
			</div>

			<div class="flex h-full max-h-[80vh] min-w-0 flex-col">
				<div class="flex h-16 shrink-0 items-center justify-between border-b border-white/20 px-4">
					<div>
						<h2 class="text-sm font-semibold text-white">Playing next</h2>
						<p class="text-xs text-white/60">
							{queueItems.length}
							{queueItems.length === 1 ? 'episode' : 'episodes'}
						</p>
					</div>

					{#if queueItems.length > 0 && onClearQueue}
						<button
							type="button"
							class="rounded-lg px-3 py-1.5 text-xs font-medium text-white/60 transition-colors hover:bg-white/10 hover:text-white"
							onclick={onClearQueue}
						>
							Clear
						</button>
					{/if}
				</div>

				<div class="flex-1 overflow-y-auto">
					<QueueList
						{queueItems}
						{manualQueueLength}
						{feeds}
						appearance="inverse"
						rowPaddingClass="px-4"
						separatorPaddingClass="px-4"
						onRemoveItem={onRemoveQueueItem}
						onMoveItemUp={onMoveQueueItemUp}
						onMoveItemDown={onMoveQueueItemDown}
					/>
				</div>
			</div>
		</div>
	</div>
{/if}
