<script lang="ts">
	import Icon from '@iconify/svelte';
	import {
		selectSection,
		selectFeed,
		selectStation,
		closeInspector,
		playbackState,
		requestTogglePlayback,
		requestNextEpisode,
		requestPreviousEpisode,
		requestSeekTo
	} from '$lib/state';
	import type { Feed } from '$lib/types/feed';
	import type { Station } from '$lib/types/station';

	type CategoryLabel = 'Navigation' | 'Actions' | 'Playback' | 'View';

	interface StaticCommand {
		kind: 'command';
		id: string;
		title: string;
		icon: string;
		category: CategoryLabel;
		keywords: string[];
		action: () => void;
	}

	type PaletteItem =
		| StaticCommand
		| { kind: 'feed'; data: Feed; action: () => void }
		| { kind: 'station'; data: Station; action: () => void };

	type Props = {
		open: boolean;
		feeds: Feed[];
		stations: Station[];
		isPlaying: boolean;
		onClose: () => void;
		onToggleCover: () => void;
		onToggleMiniPlayer: () => void;
		onToggleSidebar: () => void;
		onAddFeed: () => void;
		onAddStation: () => void;
	};

	let {
		open,
		feeds,
		stations,
		isPlaying,
		onClose,
		onToggleCover,
		onToggleMiniPlayer,
		onToggleSidebar,
		onAddFeed,
		onAddStation
	}: Props = $props();

	let inputValue = $state('');
	let highlightedIndex = $state(-1);
	let inputRef: HTMLInputElement | undefined = $state();

	const allCommands = $derived.by(() => {
		const commands: StaticCommand[] = [
			{
				kind: 'command',
				id: 'nav-home',
				title: 'Go to Home',
				icon: 'heroicons:home',
				category: 'Navigation',
				keywords: ['home', 'dashboard'],
				action: () => {
					closeInspector();
					selectSection('home');
					onClose();
				}
			},
			{
				kind: 'command',
				id: 'nav-all',
				title: 'Go to All Feeds',
				icon: 'heroicons:squares-2x2',
				category: 'Navigation',
				keywords: ['all', 'feeds', 'everything'],
				action: () => {
					closeInspector();
					selectSection('all');
					onClose();
				}
			},
			{
				kind: 'command',
				id: 'nav-unread',
				title: 'Go to Unread',
				icon: 'heroicons:inbox',
				category: 'Navigation',
				keywords: ['unread', 'inbox', 'new'],
				action: () => {
					closeInspector();
					selectSection('unread');
					onClose();
				}
			},
			{
				kind: 'command',
				id: 'nav-media',
				title: 'Go to Media',
				icon: 'heroicons:microphone',
				category: 'Navigation',
				keywords: ['media', 'podcasts', 'audio'],
				action: () => {
					closeInspector();
					selectSection('media');
					onClose();
				}
			},
			{
				kind: 'command',
				id: 'nav-settings',
				title: 'Go to Settings',
				icon: 'heroicons:cog-6-tooth',
				category: 'Navigation',
				keywords: ['settings', 'preferences', 'config'],
				action: () => {
					closeInspector();
					selectSection('settings');
					onClose();
				}
			},
			{
				kind: 'command',
				id: 'add-feed',
				title: 'Add Feed',
				icon: 'lucide:plus',
				category: 'Actions',
				keywords: ['add', 'feed', 'subscribe', 'url'],
				action: () => {
					onAddFeed();
					onClose();
				}
			},
			{
				kind: 'command',
				id: 'add-station',
				title: 'Add Station',
				icon: 'lucide:radio',
				category: 'Actions',
				keywords: ['add', 'station', 'playlist', 'create'],
				action: () => {
					onAddStation();
					onClose();
				}
			}
		];

		if (isPlaying) {
			commands.push(
				{
					kind: 'command',
					id: 'play-pause',
					title: 'Pause',
					icon: 'lucide:pause',
					category: 'Playback',
					keywords: ['play', 'pause', 'toggle'],
					action: () => {
						requestTogglePlayback();
						onClose();
					}
				},
				{
					kind: 'command',
					id: 'next-episode',
					title: 'Next Episode',
					icon: 'lucide:skip-forward',
					category: 'Playback',
					keywords: ['next', 'skip', 'forward', 'episode'],
					action: () => {
						requestNextEpisode();
						onClose();
					}
				},
				{
					kind: 'command',
					id: 'prev-episode',
					title: 'Previous Episode',
					icon: 'lucide:skip-back',
					category: 'Playback',
					keywords: ['previous', 'prev', 'back', 'episode'],
					action: () => {
						requestPreviousEpisode();
						onClose();
					}
				},
				{
					kind: 'command',
					id: 'skip-forward',
					title: 'Skip Forward 15s',
					icon: 'lucide:forward',
					category: 'Playback',
					keywords: ['skip', 'forward', 'jump', 'seek', 'ahead'],
					action: () => {
						const pos = playbackState.currentPlaybackState?.positionSeconds ?? 0;
						requestSeekTo(pos + 15);
						onClose();
					}
				},
				{
					kind: 'command',
					id: 'skip-back',
					title: 'Skip Back 15s',
					icon: 'lucide:rewind',
					category: 'Playback',
					keywords: ['skip', 'back', 'rewind', 'jump', 'seek'],
					action: () => {
						const pos = playbackState.currentPlaybackState?.positionSeconds ?? 0;
						requestSeekTo(Math.max(0, pos - 15));
						onClose();
					}
				}
			);
		}

		commands.push(
			{
				kind: 'command',
				id: 'toggle-cover',
				title: 'Toggle Cover View',
				icon: 'heroicons:arrows-pointing-out',
				category: 'View',
				keywords: ['cover', 'fullscreen', 'artwork'],
				action: () => {
					onToggleCover();
					onClose();
				}
			},
			{
				kind: 'command',
				id: 'toggle-mini',
				title: 'Toggle Mini Player',
				icon: 'heroicons:window',
				category: 'View',
				keywords: ['mini', 'player', 'popout', 'pip'],
				action: () => {
					onToggleMiniPlayer();
					onClose();
				}
			},
			{
				kind: 'command',
				id: 'toggle-sidebar',
				title: 'Toggle Sidebar',
				icon: 'lucide:panel-left',
				category: 'View',
				keywords: ['sidebar', 'toggle', 'collapse', 'expand'],
				action: () => {
					onToggleSidebar();
					onClose();
				}
			}
		);

		return commands;
	});

	function matchesStatic(term: string, cmd: StaticCommand): boolean {
		if (!term) return true;
		return (
			cmd.title.toLowerCase().includes(term) ||
			cmd.keywords.some((k) => k.toLowerCase().includes(term))
		);
	}

	function feedsMatchScore(feed: Feed, term: string): number {
		const lower = feed.title.toLowerCase();
		if (lower.startsWith(term)) return 0;
		if (lower.includes(term)) return 1;
		return -1;
	}

	function stationsMatchScore(station: Station, term: string): number {
		const lower = station.name.toLowerCase();
		if (lower.startsWith(term)) return 0;
		if (lower.includes(term)) return 1;
		return -1;
	}

	const categoryOrder: Record<string, number> = {
		Navigation: 0,
		Actions: 1,
		Playback: 2,
		View: 3,
		Feeds: 4,
		Stations: 5
	};

	const flatResults = $derived.by(() => {
		const term = inputValue.trim().toLowerCase();

		const filteredCommands = allCommands.filter((c) => matchesStatic(term, c));

		const results: PaletteItem[] = [...filteredCommands];

		if (term) {
			const matchingFeeds = feeds
				.map((f) => ({ feed: f, score: feedsMatchScore(f, term) }))
				.filter((f) => f.score >= 0)
				.sort((a, b) => a.score - b.score)
				.slice(0, 5)
				.map(({ feed }) => ({
					kind: 'feed' as const,
					data: feed,
					action: () => {
						closeInspector();
						selectFeed(feed.id);
						onClose();
					}
				}));

			const matchingStations = stations
				.map((s) => ({ station: s, score: stationsMatchScore(s, term) }))
				.filter((s) => s.score >= 0)
				.sort((a, b) => a.score - b.score)
				.slice(0, 5)
				.map(({ station }) => ({
					kind: 'station' as const,
					data: station,
					action: () => {
						closeInspector();
						selectStation(station.id);
						onClose();
					}
				}));

			results.push(...matchingFeeds, ...matchingStations);
		}

		return results.sort((a, b) => {
			const catA = categoryOrder[getItemCategory(a)] ?? 99;
			const catB = categoryOrder[getItemCategory(b)] ?? 99;
			return catA - catB;
		});
	});

	function getItemCategory(item: PaletteItem): string {
		if (item.kind === 'command') return item.category;
		if (item.kind === 'feed') return 'Feeds';
		return 'Stations';
	}

	function getItemKey(item: PaletteItem): string {
		if (item.kind === 'command') return item.id;
		if (item.kind === 'feed') return `feed:${item.data.id}`;
		return `station:${item.data.id}`;
	}

	const isEmpty = $derived(flatResults.length === 0 && inputValue.trim() !== '');

	$effect(() => {
		if (open) {
			requestAnimationFrame(() => {
				inputRef?.focus();
				inputRef?.select();
			});
		} else {
			inputValue = '';
			highlightedIndex = -1;
		}
	});

	$effect(() => {
		if (open) {
			highlightedIndex = flatResults.length > 0 ? 0 : -1;
		}
	});

	function handleKeydown(event: KeyboardEvent) {
		switch (event.key) {
			case 'ArrowDown':
			case 'Tab': {
				event.preventDefault();
				const step = event.key === 'Tab' && event.shiftKey ? -1 : 1;
				highlightedIndex =
					flatResults.length === 0
						? -1
						: (highlightedIndex + step + flatResults.length) % flatResults.length;
				break;
			}
			case 'ArrowUp': {
				event.preventDefault();
				highlightedIndex =
					flatResults.length === 0
						? -1
						: (highlightedIndex - 1 + flatResults.length) % flatResults.length;
				break;
			}
			case 'Enter':
				event.preventDefault();
				if (highlightedIndex >= 0 && highlightedIndex < flatResults.length) {
					flatResults[highlightedIndex].action();
				}
				break;
			case 'Escape':
				event.preventDefault();
				onClose();
				break;
		}
	}

	function handleItemClick(item: PaletteItem) {
		item.action();
	}

	function handleBackdropClick() {
		onClose();
	}

	function handlePanelClick(event: MouseEvent) {
		event.stopPropagation();
	}

	function handleItemMouseEnter(index: number) {
		highlightedIndex = index;
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-start justify-center bg-black/50 pt-[15vh] backdrop-blur-sm"
		tabindex="-1"
		onclick={handleBackdropClick}
		onkeydown={(event) => {
			if (event.key === 'Escape') {
				event.preventDefault();
				onClose();
			}
		}}
		role="dialog"
		aria-label="Command palette"
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="w-full max-w-lg rounded-xl border border-border bg-surface-shell-opaque shadow-2xl"
			onclick={handlePanelClick}
		>
			<div class="flex items-center gap-3 border-b border-border px-4 py-3">
				<Icon icon="lucide:search" class="size-4 shrink-0 text-fg-muted" />
				<input
					bind:this={inputRef}
					bind:value={inputValue}
					type="text"
					class="w-full bg-transparent text-sm text-fg placeholder:text-fg-muted focus:outline-none"
					placeholder="Type a command..."
					onkeydown={handleKeydown}
				/>
				{#if inputValue}
					<button
						type="button"
						class="cursor-pointer text-xs text-fg-muted hover:text-fg"
						onclick={() => {
							inputValue = '';
							highlightedIndex = -1;
							inputRef?.focus();
						}}
						aria-label="Clear search"
					>
						<Icon icon="lucide:x" class="size-3" />
					</button>
				{/if}
			</div>

			<div class="max-h-80 overflow-y-auto p-2">
				{#if isEmpty}
					<div class="px-4 py-8 text-center text-sm text-fg-muted">No results found</div>
				{:else if flatResults.length > 0}
					{#each flatResults as item, i (getItemKey(item))}
						{@const showHeader =
							i === 0 || getItemCategory(flatResults[i - 1]) !== getItemCategory(item)}
						{#if showHeader}
							<div
								class="px-3 pt-2 pb-1 text-[10px] font-semibold tracking-wider text-fg-muted uppercase"
							>
								{getItemCategory(item)}
							</div>
						{/if}
						<button
							type="button"
							class="flex w-full cursor-pointer items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors"
							class:bg-surface-raised={i === highlightedIndex}
							class:text-fg={i === highlightedIndex}
							class:text-fg-muted={i !== highlightedIndex}
							onclick={() => handleItemClick(item)}
							onmouseenter={() => handleItemMouseEnter(i)}
						>
							<Icon
								icon={item.kind === 'command'
									? item.icon
									: item.kind === 'feed'
										? 'lucide:rss'
										: 'lucide:radio'}
								class="size-4 shrink-0"
							/>
							<span class="truncate">
								{item.kind === 'command'
									? item.title
									: item.kind === 'feed'
										? item.data.title
										: item.data.name}
							</span>
							{#if item.kind === 'feed'}
								<span
									class="ml-auto shrink-0 text-[10px] font-medium tracking-wider text-fg-muted uppercase"
									>Feed</span
								>
							{/if}
							{#if item.kind === 'station'}
								<span
									class="ml-auto shrink-0 text-[10px] font-medium tracking-wider text-fg-muted uppercase"
									>Station</span
								>
							{/if}
						</button>
					{/each}
				{/if}
			</div>

			<div class="border-t border-border px-4 py-2">
				<div class="flex items-center gap-4 text-[10px] text-fg-muted">
					<span class="flex items-center gap-1">
						<span class="bg-surface-raised rounded px-1 py-0.5 font-mono text-[10px]">↑↓</span>
						<span class="bg-surface-raised rounded px-1 py-0.5 font-mono text-[10px]">⇥</span>
						Navigate
					</span>
					<span class="flex items-center gap-1">
						<span class="bg-surface-raised rounded px-1 py-0.5 font-mono text-[10px]">↵</span>
						Execute
					</span>
					<span class="flex items-center gap-1">
						<span class="bg-surface-raised rounded px-1 py-0.5 font-mono text-[10px]">Esc</span>
						Close
					</span>
				</div>
			</div>
		</div>
	</div>
{/if}
