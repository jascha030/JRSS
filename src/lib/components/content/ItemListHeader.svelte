<script lang="ts">
	import type { Feed } from '$lib/types/feed';
	import type { Station } from '$lib/types/station';
	import { formatDate } from '$lib/utils/format';
	import { openFeedContextMenu } from '$lib/utils/tauri-menu';
	import SearchBar from '$lib/components/content/SearchBar.svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';

	type Props = {
		pageHeading: string;
		totalCount: number;
		selectedFeed: Feed | null;
		selectedStation: Station | null;
		searchInputRef?: HTMLInputElement | null;
		isRefreshing: boolean;
		itemSortOrder: 'newest_first' | 'oldest_first';
		searchLabel: string | null;
		searchPlaceholder: string | null;
		searchValue: string;
		onSearchChange: (value: string) => void;
		onPlayStation: () => void;
		onEditStation: () => void;
		onDeleteStation: () => void;
		onRefreshFeed: () => void;
		onInspectFeed: () => void;
		onSetSortOrder: (order: 'newest_first' | 'oldest_first') => void;
	};

	let {
		pageHeading,
		totalCount,
		selectedFeed,
		selectedStation,
		searchInputRef = $bindable(null),
		isRefreshing,
		itemSortOrder,
		searchLabel,
		searchPlaceholder,
		searchValue,
		onSearchChange,
		onPlayStation,
		onEditStation,
		onDeleteStation,
		onRefreshFeed,
		onInspectFeed,
		onSetSortOrder
	}: Props = $props();
</script>

<div class="shrink-0 border-b border-border px-6 py-8 pb-7.75 lg:px-8">
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
			<div>
				<h2
					class="mt-2 text-2xl font-semibold tracking-tight text-fg"
					class:select-none={selectedFeed}
					oncontextmenu={(() => {
						const feed = selectedFeed;
						return feed ? (event: MouseEvent) => void openFeedContextMenu(event, feed) : undefined;
					})()}
				>
					{pageHeading}
				</h2>

				{#if selectedFeed?.lastFetchedAt}
					<p class="mt-1 text-xs text-fg-subtle">
						Last refreshed {formatDate(selectedFeed.lastFetchedAt)}
					</p>
				{/if}
			</div>
		</div>

		<p class="text-sm whitespace-nowrap text-fg-muted">{totalCount} episodes</p>

		<div class="flex flex-row flex-wrap items-center justify-end gap-3 align-bottom">
			{#if selectedStation}
				<IconButton
					icon="lucide:play"
					title="Play station"
					label="Play station"
					variant="accent"
					onclick={onPlayStation}
				/>
				<IconButton
					icon="lucide:pencil"
					title="Edit station"
					label="Edit station"
					onclick={onEditStation}
				/>
				<IconButton
					icon="lucide:trash-2"
					title="Delete station"
					label="Delete station"
					variant="error"
					onclick={onDeleteStation}
				/>
			{:else if selectedFeed}
				<div class="flex flex-col">
					<select
						id="feed-sort-order"
						class="preset-outlined-subtle select flex h-9 min-w-0 grow rounded-xl border border-border placeholder:text-fg-muted"
						aria-label="Sort order"
						value={itemSortOrder}
						onchange={(event) => {
							const target = event.currentTarget;
							if (!(target instanceof HTMLSelectElement)) return;

							const value = target.value;
							if (value === 'newest_first' || value === 'oldest_first') {
								onSetSortOrder(value);
							}
						}}
					>
						<option value="newest_first">Newest first</option>
						<option value="oldest_first">Oldest first</option>
					</select>
				</div>
				<div>
					{#key isRefreshing}
						<IconButton
							icon="lucide:refresh-cw"
							title="Refresh feed"
							label="Refresh feed"
							iconClass={`size-5 ${isRefreshing ? 'animate-spin' : ''}`}
							disabled={isRefreshing}
							onclick={onRefreshFeed}
						/>
					{/key}

					<IconButton
						icon="lucide:scan-search"
						title="Inspect feed XML"
						label="Inspect feed XML"
						iconClass="size-5"
						onclick={onInspectFeed}
					/>
				</div>
			{/if}
		</div>

		{#if searchLabel && searchPlaceholder}
			<SearchBar
				bind:inputRef={searchInputRef}
				label={searchLabel}
				placeholder={searchPlaceholder}
				value={searchValue}
				onChange={onSearchChange}
			/>
		{/if}
	</div>
</div>
