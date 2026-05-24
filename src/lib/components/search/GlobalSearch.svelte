<script lang="ts">
	import type { Feed } from '$lib/types/feed';
	import type { FeedListItem } from '$lib/types/item';
	import type { Station } from '$lib/types/station';
	import SearchInput from '$lib/components/ui/SearchInput.svelte';
	import {
		createGlobalSearch,
		type SearchResultEntry
	} from '$lib/services/global-search.svelte.js';
	import { createKeyboardListNavigation } from '$lib/services/keyboard-list-navigation.svelte';
	import SearchResultRow from './SearchResultRow.svelte';

	type Props = {
		feeds: Feed[];
		stations: Station[];
		onSelectResult: (item: FeedListItem) => void;
		onSelectFeedResult: (feed: Feed) => void;
		onSelectStationResult: (station: Station) => void;
		onAddFeed: (url: string) => void;
	};

	let {
		feeds,
		stations,
		onSelectResult,
		onSelectFeedResult,
		onSelectStationResult,
		onAddFeed
	}: Props = $props();

	let searchInputRef = $state<HTMLInputElement | null>(null);
	let containerRef = $state<HTMLDivElement | null>(null);
	let inputValue = $state('');

	const search = createGlobalSearch({
		getTerm: () => inputValue.trim(),
		getFeeds: () => feeds,
		getStations: () => stations
	});

	const navigation = createKeyboardListNavigation<HTMLDivElement, HTMLDivElement>({
		getItemCount: () => search.entries.length,
		onRequestClose: resetAndBlur,
		scrollPadding: 8
	});

	$effect(() => {
		if (!search.isOpen) {
			navigation.reset();
			return;
		}

		navigation.sync();
	});

	$effect(() => {
		if (!search.isOpen || search.entries.length === 0) return;

		navigation.ensureSelectedVisible();
	});

	function resetAndBlur() {
		inputValue = '';
		search.clear();
		navigation.reset();
		searchInputRef?.blur();
	}

	function closeResults() {
		search.close();
		navigation.reset();
	}

	function executeEntry(entry: SearchResultEntry) {
		if (entry.kind === 'action') {
			onAddFeed(entry.data.url);
			resetAndBlur();
			return;
		}

		if (entry.kind === 'feed') {
			onSelectFeedResult(entry.data);
			resetAndBlur();
			return;
		}

		if (entry.kind === 'station') {
			onSelectStationResult(entry.data);
			resetAndBlur();
			return;
		}

		onSelectResult(entry.data);
		resetAndBlur();
	}

	function handleInputKeydown(event: KeyboardEvent) {
		navigation.handleKeydown({
			event,
			items: search.entries,
			onExecute: executeEntry
		});
	}

	function handleInputFocus() {
		if (!inputValue.trim()) return;

		search.open();
		navigation.open();
	}

	function handleResultSelect(entry: SearchResultEntry, index: number) {
		navigation.select(index);
		executeEntry(entry);
	}

	function handleResultHover(index: number) {
		navigation.hover(index);
	}

	function handleGlobalKeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement | null;

		if (
			target instanceof HTMLInputElement ||
			target instanceof HTMLTextAreaElement ||
			target?.getAttribute('contenteditable') === 'true'
		) {
			return;
		}

		if (event.key !== '/') return;
		if (event.metaKey || event.ctrlKey || event.altKey) return;

		event.preventDefault();
		searchInputRef?.focus();
	}

	function handleWindowClick(event: MouseEvent) {
		const target = event.target;

		if (!(target instanceof Node)) return;
		if (!containerRef) return;
		if (containerRef.contains(target)) return;

		closeResults();
	}
</script>

<svelte:window onkeydown={handleGlobalKeydown} onclick={handleWindowClick} />

<div bind:this={containerRef} class="relative mr-12 flex-1">
	<SearchInput
		id="global-search"
		label="Search all feeds"
		placeholder="Search all feeds"
		bind:value={inputValue}
		bind:inputRef={searchInputRef}
		kbdShortcuts={['/']}
		isLoading={search.isLoading}
		onkeydown={handleInputKeydown}
		onfocus={handleInputFocus}
	/>

	{#if search.isOpen && search.entries.length > 0}
		<!-- svelte-ignore a11y_interactive_supports_focus -->
		<div
			use:navigation.setListRef
			class="pointer-events-auto absolute top-full right-0 left-0 z-50 mt-1 max-h-[calc(100vh-(32*var(--spacing)))] overflow-x-hidden overflow-y-auto rounded-xl border border-border bg-surface-shell-opaque shadow-xl backdrop-blur-xl"
			role="listbox"
			aria-label="Search results"
			onpointerleave={navigation.leaveList}
		>
			{#each search.entries as entry, index (entry.id)}
				<SearchResultRow
					{entry}
					{index}
					feedTitleById={search.feedTitleById}
					isHighlighted={index === navigation.highlightedIndex}
					setRef={navigation.setItemRef}
					onSelect={handleResultSelect}
					onHover={handleResultHover}
				/>
			{/each}
		</div>
	{:else if search.isOpen && !search.isLoading && inputValue.trim()}
		<div
			class="absolute top-full right-0 left-0 z-50 mt-1 rounded-xl border border-border bg-surface-glass px-4 py-6 text-center shadow-xl backdrop-blur-xl"
		>
			<p class="text-sm text-fg-muted">No results for "{inputValue.trim()}"</p>
		</div>
	{/if}
</div>
