<script lang="ts">
	import type { Feed } from '$lib/types/feed';
	import type { Station } from '$lib/types/station';
	import { formatDate } from '$lib/utils/format';
	import { openFeedContextMenu, openStationContextMenu } from '$lib/utils/tauri-menu';
	import { feedImageUrls } from '$lib/state';
	import SearchBar from '$lib/components/content/SearchBar.svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import {
		Combobox,
		Portal,
		type ComboboxRootProps,
		useListCollection
	} from '@skeletonlabs/skeleton-svelte';

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
		onExportFeed: () => void;
		isExporting: boolean;
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
		onExportFeed,
		isExporting,
		onSetSortOrder
	}: Props = $props();

	const sortOptions = [
		{ label: 'Newest first', value: 'newest_first' },
		{ label: 'Oldest first', value: 'oldest_first' }
	];

	const collection = $derived(
		useListCollection({
			items: sortOptions,
			itemToString: (item) => item.label,
			itemToValue: (item) => item.value
		})
	);

	const onValueChange: ComboboxRootProps['onValueChange'] = (event) => {
		const value = event.value[0] ?? '';
		if (value === 'newest_first' || value === 'oldest_first') {
			onSetSortOrder(value);
		}
	};

	const feedImageFallback = $derived((selectedFeed?.title.trim().slice(0, 1) ?? '?').toUpperCase());
</script>

<div class="shrink-0 border-b border-border bg-surface px-6 py-8 pb-7.75 backdrop-blur-md lg:px-8">
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
			<div>
				<div class="mt-2 flex items-center gap-3">
					{#if selectedFeed}
						<div
							class="flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-surface-elevated text-xs font-semibold text-fg-muted"
						>
							{#if selectedFeed.imageUrl}
								<img
									src={feedImageUrls[selectedFeed.id] || selectedFeed.imageUrl}
									alt={selectedFeed.title}
									class="size-full object-cover"
								/>
							{:else}
								<span>{feedImageFallback}</span>
							{/if}
						</div>
					{/if}

					<h2
						class="text-2xl font-semibold tracking-tight text-fg"
						class:select-none={selectedFeed || selectedStation}
						oncontextmenu={selectedFeed
							? (event: MouseEvent) => void openFeedContextMenu(event, selectedFeed)
							: selectedStation
								? (event: MouseEvent) => void openStationContextMenu(event, selectedStation)
								: undefined}
					>
						{pageHeading}
					</h2>
				</div>

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
					<Combobox
						class="w-full min-w-0"
						{collection}
						value={[itemSortOrder]}
						{onValueChange}
						openOnClick
					>
						<Combobox.Label class="sr-only">Sort order</Combobox.Label>
						<Combobox.Control class="flex h-9 items-center gap-2">
							<Combobox.Input class="h-full min-w-0 flex-1" />
							<Combobox.Trigger />
						</Combobox.Control>
						<Portal>
							<Combobox.Positioner>
								<Combobox.Content class="z-50 max-h-64 overflow-y-auto">
									{#each sortOptions as item (item.value)}
										<Combobox.Item {item}>
											<Combobox.ItemText>{item.label}</Combobox.ItemText>
											<Combobox.ItemIndicator />
										</Combobox.Item>
									{/each}
								</Combobox.Content>
							</Combobox.Positioner>
						</Portal>
					</Combobox>
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
					<IconButton
						icon="lucide:download"
						title="Export feed"
						label="Export feed"
						iconClass={`size-5 ${isExporting ? 'animate-pulse' : ''}`}
						disabled={isExporting}
						onclick={onExportFeed}
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
