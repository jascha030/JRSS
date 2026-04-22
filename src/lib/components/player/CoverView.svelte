<script lang="ts">
	/**
	 * Cover view component - a full-screen player view showing album art prominently
	 * with the same audio controls as the compact AudioPlayer.
	 */
	import type { Feed, MediaListItem, PlaybackState } from '$lib/types/rss';
	import type { Snippet } from 'svelte';
	import { formatDuration } from '$lib/utils/format';
	import { requestSeekTo, requestTogglePlayback } from '$lib/stores/app.svelte';
	import AudioPlayerControls from './AudioPlayerControls.svelte';
	import AudioPlayerInfo from './AudioPlayerInfo.svelte';
	import AudioSeekBar from './AudioSeekBar.svelte';
	import AudioPlayerVolume from './AudioPlayerVolume.svelte';

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

	const hasAutoItems = $derived(queueItems.length > manualQueueLength);

	let feedTitleById = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const feed of feeds) {
			map[feed.id] = feed.title;
		}
		return map;
	});

	function feedTitleForItem(queueItem: MediaListItem): string | null {
		return feedTitleById[queueItem.feedId] ?? null;
	}

	function durationLabel(queueItem: MediaListItem): string | null {
		const duration = queueItem.mediaEnclosure.durationSeconds;
		if (!duration || duration <= 0) {
			return null;
		}

		const position = queueItem.playbackPositionSeconds;
		if (position > 0) {
			const remaining = Math.max(0, duration - position);
			return `${formatDuration(remaining)} left`;
		}

		return formatDuration(duration);
	}

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
	<div class="fixed inset-0 bg-black px-12 {className}">
		{#if onClose}
			<button
				type="button"
				class="absolute top-6 left-6 flex size-12 items-center justify-center rounded-full bg-white/10 text-white/80 transition-colors hover:bg-white/20 hover:text-white"
				aria-label="Close cover view"
				onclick={onClose}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="2"
					stroke="currentColor"
					class="size-6"
				>
					<path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
				</svg>
			</button>
		{/if}

		<div class="grid h-full grid-cols-4 items-center justify-center gap-8 p-8">
			<!-- Left column: Player -->
			<div class="col-span-2">
				<div class="mx-auto flex w-full max-w-6xl flex-col p-12">
					<img
						src={imageUrl}
						alt=""
						class="mx-auto w-full max-w-3xl rounded-lg object-cover shadow-sm select-none"
					/>
				</div>
				<div class="mx-auto flex w-full max-w-6xl flex-col gap-6">
					<div
						class="mx-auto grid w-full max-w-6xl grid-cols-[1fr_auto_1fr] items-center gap-6 4xl:max-w-400"
					>
						<div class="min-w-0">
							<AudioPlayerInfo {item} onNavigate={onNavigateToItem} class="" />
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

			<!-- Right column: Queue -->
			<div class="col-span-2 flex h-full max-h-[80vh] flex-col">
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
					{#if queueItems.length === 0}
						<div class="flex flex-col items-center justify-center px-6 py-16 text-center">
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="mb-3 size-10 text-white/40"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M3.75 12h16.5m-16.5 3.75h16.5M3.75 19.5h16.5M5.625 4.5h12.75a1.875 1.875 0 0 1 0 3.75H5.625a1.875 1.875 0 0 1 0-3.75Z"
								/>
							</svg>
							<p class="text-sm font-medium text-white/60">Queue is empty</p>
							<p class="mt-1 text-xs text-white/40">Press play on an episode to auto-populate</p>
						</div>
					{:else}
						<ul class="py-2">
							{#each queueItems as queueItem, index (queueItem.id)}
								{#if index === manualQueueLength && hasAutoItems}
									<li class="flex items-center gap-3 px-4 py-2" aria-hidden="true">
										<div class="h-px flex-1 bg-white/20"></div>
										<span class="text-[10px] font-medium tracking-widest text-white/50 uppercase">
											From this feed
										</span>
										<div class="h-px flex-1 bg-white/20"></div>
									</li>
								{/if}
								<li
									class="group relative flex items-start gap-3 px-4 py-3 transition-colors hover:bg-white/10"
								>
									<span
										class="mt-0.5 flex size-5 shrink-0 items-center justify-center rounded text-[10px] font-semibold text-white/60 tabular-nums"
									>
										{index + 1}
									</span>

									<div class="min-w-0 flex-1">
										<p class="truncate text-sm font-medium text-white">
											{queueItem.title}
										</p>
										{#if feedTitleForItem(queueItem)}
											<p class="mt-0.5 truncate text-xs text-white/60">
												{feedTitleForItem(queueItem)}
											</p>
										{/if}
										{#if durationLabel(queueItem)}
											<p class="mt-0.5 text-xs text-white/40 tabular-nums">
												{durationLabel(queueItem)}
											</p>
										{/if}
									</div>

									<div
										class="mt-0.5 flex shrink-0 flex-col items-center gap-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
									>
										{#if index > 0 && onMoveQueueItemUp}
											<button
												type="button"
												title="Move up"
												aria-label={`Move ${queueItem.title} up in queue`}
												class="flex size-6 items-center justify-center rounded text-white/50 hover:bg-white/20 hover:text-white"
												onclick={() => onMoveQueueItemUp(queueItem.id)}
											>
												<svg
													xmlns="http://www.w3.org/2000/svg"
													fill="none"
													viewBox="0 0 24 24"
													stroke-width="2"
													stroke="currentColor"
													class="size-3.5"
												>
													<path
														stroke-linecap="round"
														stroke-linejoin="round"
														d="m4.5 15.75 7.5-7.5 7.5 7.5"
													/>
												</svg>
											</button>
										{/if}
										{#if onRemoveQueueItem}
											<button
												type="button"
												title="Remove from queue"
												aria-label={`Remove ${queueItem.title} from queue`}
												class="flex size-6 items-center justify-center rounded text-white/50 hover:bg-white/20 hover:text-white"
												onclick={() => onRemoveQueueItem(queueItem.id)}
											>
												<svg
													xmlns="http://www.w3.org/2000/svg"
													fill="none"
													viewBox="0 0 24 24"
													stroke-width="1.5"
													stroke="currentColor"
													class="size-3.5"
												>
													<path
														stroke-linecap="round"
														stroke-linejoin="round"
														d="M6 18 18 6M6 6l12 12"
													/>
												</svg>
											</button>
										{/if}
										{#if index < queueItems.length - 1 && onMoveQueueItemDown}
											<button
												type="button"
												title="Move down"
												aria-label={`Move ${queueItem.title} down in queue`}
												class="flex size-6 items-center justify-center rounded text-white/50 hover:bg-white/20 hover:text-white"
												onclick={() => onMoveQueueItemDown(queueItem.id)}
											>
												<svg
													xmlns="http://www.w3.org/2000/svg"
													fill="none"
													viewBox="0 0 24 24"
													stroke-width="2"
													stroke="currentColor"
													class="size-3.5"
												>
													<path
														stroke-linecap="round"
														stroke-linejoin="round"
														d="m19.5 8.25-7.5 7.5-7.5-7.5"
													/>
												</svg>
											</button>
										{/if}
									</div>
								</li>
							{/each}
						</ul>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}
