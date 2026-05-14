<script lang="ts">
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	import { queryItems } from '$lib/services/item';
	import type { Feed } from '$lib/types/feed';
	import type { FeedListItem } from '$lib/types/item';
	import { isMediaItem } from '$lib/types/item';
	import { formatDate } from '$lib/utils/format';
	import SearchInput from '$lib/components/ui/SearchInput.svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';

	type Props = {
		onOpenDialog: () => void;
		feeds: Feed[];
		onSelectResult: (item: FeedListItem) => void;
		onSelectFeedResult: (feed: Feed) => void;
		onAddFeed: (url: string) => void;
	};

	type ActionResult = {
		title: string;
		description?: string;
		handleSelectAction: () => void;
	};

	type CombinedResult =
		| { kind: 'action'; data: ActionResult }
		| { kind: 'feed'; data: Feed }
		| { kind: 'item'; data: FeedListItem };

	let { onOpenDialog, feeds, onSelectResult, onSelectFeedResult, onAddFeed }: Props = $props();

	let searchInputRef = $state<HTMLInputElement | null>(null);
	let containerRef = $state<HTMLDivElement | null>(null);
	let inputValue = $state('');
	let feedResults = $state<Feed[]>([]);
	let results = $state<FeedListItem[]>([]);
	let isLoading = $state(false);
	let isOpen = $state(false);
	let highlightedIndex = $state(-1);
	let resultRefs = $state<(HTMLDivElement | null)[]>([]);

	const isUrl = (input: string): boolean => URL.canParse(input);

	const actionResults = $derived.by<ActionResult[]>(() => {
		const term = inputValue.trim();

		if (!term || !isUrl(term)) {
			return [];
		}

		return [
			{
				title: 'Add feed',
				description: `Add ${term} as a new feed`,
				handleSelectAction: () => {
					onAddFeed(term);
					inputValue = '';
					isOpen = false;
					highlightedIndex = -1;
					searchInputRef?.blur();
				}
			}
		];
	});

	const feedTitleById = $derived.by(() => new Map(feeds.map((f) => [f.id, f.title])));

	const combinedResults = $derived.by<CombinedResult[]>(() => [
		...actionResults.map((a) => ({ kind: 'action' as const, data: a })),
		...feedResults.map((f) => ({ kind: 'feed' as const, data: f })),
		...results.map((i) => ({ kind: 'item' as const, data: i }))
	]);

	$effect(() => {
		if (highlightedIndex >= 0 && resultRefs[highlightedIndex]) {
			resultRefs[highlightedIndex]?.scrollIntoView({
				block: 'nearest',
				behavior: 'smooth'
			});
		}
	});

	$effect(() => {
		const term = inputValue.trim();

		if (!term) {
			feedResults = [];
			results = [];
			isLoading = false;
			isOpen = false;
			highlightedIndex = -1;
			return;
		}

		const lowerTerm = term.toLowerCase();
		const newFeedResults = feeds
			.filter(
				(f) => f.title.toLowerCase().includes(lowerTerm) || f.url.toLowerCase().includes(lowerTerm)
			)
			.slice(0, 3);

		feedResults = newFeedResults;
		results = [];
		highlightedIndex = -1;
		isLoading = true;
		isOpen = true;

		const hasFeedResults = newFeedResults.length > 0;
		const hasActionResults = actionResults.length > 0;

		let cancelled = false;

		const timer = setTimeout(() => {
			void queryItems({
				section: 'all',
				offset: 0,
				limit: 8,
				search: term,
				sortOrder: 'newest_first'
			})
				.then((page) => {
					if (cancelled) return;

					const nextResults = page.items;
					results = nextResults;

					if (nextResults.length > 0 || hasFeedResults || hasActionResults) {
						isOpen = true;
					}
				})
				.catch(() => {
					if (cancelled) return;
					results = [];
				})
				.finally(() => {
					if (cancelled) return;
					isLoading = false;
				});
		}, 220);

		return () => {
			cancelled = true;
			clearTimeout(timer);
		};
	});

	$effect(() => {
		let unlistenAddFeed: UnlistenFn | undefined;

		void listen('menu-add-feed', () => {
			onOpenDialog();
		}).then((unlisten) => {
			unlistenAddFeed = unlisten;
		});

		return () => {
			unlistenAddFeed?.();
		};
	});

	function handleSelect(item: FeedListItem): void {
		onSelectResult(item);
		inputValue = '';
		isOpen = false;
		highlightedIndex = -1;
		searchInputRef?.blur();
	}

	function handleSelectFeed(feed: Feed): void {
		onSelectFeedResult(feed);
		inputValue = '';
		isOpen = false;
		highlightedIndex = -1;
		searchInputRef?.blur();
	}

	function handleInputKeydown(event: KeyboardEvent): void {
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			highlightedIndex = Math.min(highlightedIndex + 1, combinedResults.length - 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			highlightedIndex = Math.max(highlightedIndex - 1, -1);
		} else if (event.key === 'Enter') {
			event.preventDefault();

			const entry = highlightedIndex >= 0 ? combinedResults[highlightedIndex] : combinedResults[0];
			if (!entry) return;

			if (entry.kind === 'action') {
				entry.data.handleSelectAction();
			} else if (entry.kind === 'feed') {
				handleSelectFeed(entry.data);
			} else {
				handleSelect(entry.data);
			}
		} else if (event.key === 'Escape') {
			inputValue = '';
			isOpen = false;
			highlightedIndex = -1;
			searchInputRef?.blur();
		}
	}

	function handleGlobalKeydown(event: KeyboardEvent): void {
		const target = event.target as HTMLElement;

		if (
			target instanceof HTMLInputElement ||
			target instanceof HTMLTextAreaElement ||
			target.getAttribute('contenteditable') === 'true'
		) {
			return;
		}

		if (event.key === '/') {
			event.preventDefault();
			searchInputRef?.focus();
		}
	}

	function handleWindowClick(event: MouseEvent): void {
		if (containerRef && !containerRef.contains(event.target as Node)) {
			isOpen = false;
			highlightedIndex = -1;
		}
	}
</script>

<svelte:window onkeydown={handleGlobalKeydown} onclick={handleWindowClick} />

<div class="pointer-events-none flex w-full items-center gap-4 px-2">
	<div bind:this={containerRef} class="relative mr-12 flex-1">
		<SearchInput
			id="global-search"
			label="Search all feeds"
			placeholder="Search all feeds"
			bind:value={inputValue}
			bind:inputRef={searchInputRef}
			kbdShortcuts={['/']}
			{isLoading}
			bgClass="bg-surface-shell"
			onkeydown={handleInputKeydown}
			onfocus={() => {
				if (feedResults.length > 0 || results.length > 0) isOpen = true;
			}}
		/>

		{#if isOpen && combinedResults.length > 0}
			<div
				class="pointer-events-auto absolute top-full right-0 left-0 z-50 mt-1 max-h-[calc(100vh-(32*var(--spacing)))] overflow-x-hidden rounded-xl border border-border bg-surface-shell-opaque shadow-xl backdrop-blur-xl"
				role="listbox"
				aria-label="Search results"
			>
				{#each actionResults as action, i (i)}
					{@const globalIndex = i}
					<div
						bind:this={resultRefs[globalIndex]}
						class={`flex cursor-pointer flex-col gap-0.5 px-4 py-3 transition-colors ${
							i > 0 ? 'border-t border-border' : ''
						} ${globalIndex === highlightedIndex ? 'bg-surface-sidebar-active-opaque' : 'hover:bg-surface-sidebar-hover-opaque'}`}
						role="option"
						tabindex="-1"
						aria-selected={globalIndex === highlightedIndex}
						onmousedown={(e) => {
							e.preventDefault();
							action.handleSelectAction();
						}}
						onmouseenter={() => (highlightedIndex = globalIndex)}
					>
						<div class="flex items-center gap-2">
							<span class="truncate text-xs font-medium tracking-widest text-fg-muted uppercase"
								>{action.title}</span
							>
						</div>
						<p class="truncate text-sm font-medium text-fg">{action.description}</p>
					</div>
				{/each}
				{#each feedResults as feed, i (feed.id)}
					{@const globalIndex = actionResults.length + i}
					<div
						bind:this={resultRefs[globalIndex]}
						class={`flex cursor-pointer flex-col gap-0.5 px-4 py-3 transition-colors ${
							i > 0 ? 'border-t border-border' : ''
						} ${globalIndex === highlightedIndex ? 'bg-surface-sidebar-active-opaque' : 'hover:bg-surface-sidebar-hover-opaque'}`}
						role="option"
						tabindex="-1"
						aria-selected={globalIndex === highlightedIndex}
						onmousedown={(e) => {
							e.preventDefault();
							handleSelectFeed(feed);
						}}
						onmouseenter={() => (highlightedIndex = globalIndex)}
					>
						<div class="flex items-center gap-2">
							<span class="truncate text-xs font-medium tracking-widest text-fg-muted uppercase"
								>Feed</span
							>
							{#if feed.kind === 'media'}
								<span class="badge shrink-0 preset-tonal-surface text-xs">Podcast</span>
							{/if}
						</div>
						<p class="truncate text-sm font-medium text-fg">{feed.title}</p>
						<p class="truncate text-xs text-fg-secondary">{feed.url}</p>
					</div>
				{/each}
				{#each results as item, i (item.id)}
					{@const globalIndex = actionResults.length + feedResults.length + i}
					<div
						bind:this={resultRefs[globalIndex]}
						class={`flex cursor-pointer flex-col gap-0.5 px-4 py-3 transition-colors ${
							globalIndex > 0 ? 'border-t border-border' : ''
						} ${globalIndex === highlightedIndex ? 'bg-surface-sidebar-active-opaque' : 'hover:bg-surface-sidebar-hover-opaque'}`}
						role="option"
						tabindex="-1"
						aria-selected={globalIndex === highlightedIndex}
						onmousedown={(e) => {
							e.preventDefault();
							handleSelect(item);
						}}
						onmouseenter={() => (highlightedIndex = globalIndex)}
					>
						<div class="flex items-center gap-2">
							<span class="truncate text-xs font-medium tracking-widest text-fg-muted uppercase">
								{feedTitleById.get(item.feedId) ?? 'Unknown feed'}
							</span>
							{#if isMediaItem(item)}
								<span class="badge shrink-0 preset-tonal-surface text-xs">Podcast</span>
							{/if}
							<span class="ml-auto shrink-0 text-xs text-fg-subtle">
								{formatDate(item.publishedAt)}
							</span>
						</div>
						<p class="truncate text-sm font-medium text-fg">{item.title}</p>
						{#if item.previewText}
							<p class="truncate text-xs text-fg-secondary">{item.previewText}</p>
						{/if}
					</div>
				{/each}
			</div>
		{:else if isOpen && !isLoading && inputValue.trim()}
			<div
				class="absolute top-full right-0 left-0 z-50 mt-1 rounded-xl border border-border bg-surface-glass px-4 py-6 text-center shadow-xl backdrop-blur-xl"
			>
				<p class="text-sm text-fg-muted">No results for "{inputValue.trim()}"</p>
			</div>
		{/if}
	</div>

	<IconButton icon="lucide:plus" title="Add feed" variant="accent" onclick={onOpenDialog} />
</div>
