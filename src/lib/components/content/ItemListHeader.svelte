<script lang="ts">
	import { useMenuShortcuts } from '$lib/hooks/useMenuShortcuts.svelte';
	import { openStationEditor } from '$lib/hooks/useAppUi.svelte';
	import { formatDate } from '$lib/utils/format';
	import { openFeedContextMenu } from '$lib/utils/tauri-menu';
	import { toast } from 'svelte-sonner';
	import type { SidebarSection } from '$lib/state';
	import {
		selection,
		getEffectiveSortOrder,
		getActiveTotalCount,
		refreshExistingFeed,
		openInspector,
		setFeedSortOrder,
		playStation,
		deleteExistingStation
	} from '$lib/state';
	import {
		getSelectedFeed,
		getSelectedStation,
		getIsSelectedFeedRefreshing
	} from '$lib/state/selectors.svelte';

	import SearchBar from '$lib/components/content/SearchBar.svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';

	const sectionHeadings: Record<Exclude<SidebarSection, null>, string> = {
		home: 'Home',
		all: 'All feeds',
		unread: 'Unread',
		media: 'Media',
		settings: 'Settings'
	};

	const selectedFeed = $derived(getSelectedFeed());
	const selectedStation = $derived(getSelectedStation());
	const isRefreshing = $derived(getIsSelectedFeedRefreshing());
	const itemSortOrder = $derived(getEffectiveSortOrder());
	const selectedSection = $derived(selection.selectedSection);
	const totalCount = $derived(getActiveTotalCount());

	const pageHeading = $derived(
		selectedStation?.name ??
			selectedFeed?.title ??
			(selectedSection ? sectionHeadings[selectedSection] : 'All feeds')
	);

	let searchInputRef = $state<HTMLInputElement | null>(null);

	async function handlePlayStation() {
		if (!selectedStation) return;
		try {
			await playStation(selectedStation.id);
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to play station.');
		}
	}

	async function handleDeleteStation() {
		if (!selectedStation) return;
		try {
			await deleteExistingStation(selectedStation.id);
		} catch (error: unknown) {
			toast.error(error instanceof Error ? error.message : 'Unable to delete station.');
		}
	}

	function handleEditStation() {
		if (!selectedStation) return;
		openStationEditor({
			id: selectedStation.id
		});
	}

	function handleRefreshFeed() {
		if (selectedFeed) {
			void refreshExistingFeed(selectedFeed.id);
		}
	}

	function handleInspectFeed() {
		if (selectedFeed) {
			openInspector(selectedFeed.id);
		}
	}

	function handleSetSortOrder(order: 'newest_first' | 'oldest_first') {
		void setFeedSortOrder(order);
	}

	useMenuShortcuts([
		{
			event: 'menu-search-feed',
			handler: () => {
				searchInputRef?.focus();
			}
		},
		{
			event: 'menu-refresh-feed',
			handler: () => {
				if (selectedFeed && !isRefreshing) {
					handleRefreshFeed();
				}
			}
		}
	]);
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
					onclick={handlePlayStation}
				/>

				<IconButton
					icon="lucide:pencil"
					title="Edit station"
					label="Edit station"
					onclick={handleEditStation}
				/>

				<IconButton
					icon="lucide:trash-2"
					title="Delete station"
					label="Delete station"
					variant="error"
					onclick={handleDeleteStation}
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
							if (target instanceof HTMLSelectElement) {
								const value = target.value;
								if (value === 'newest_first' || value === 'oldest_first') {
									handleSetSortOrder(value);
								}
							}
						}}
					>
						<option value="newest_first">Newest first</option>
						<option value="oldest_first">Oldest first</option>
					</select>
				</div>
				<div class="">
					{#key isRefreshing}
						<IconButton
							icon="lucide:refresh-cw"
							title="Refresh feed"
							label="Refresh feed"
							iconClass="size-5 {isRefreshing ? 'animate-spin' : ''}"
							disabled={isRefreshing}
							onclick={handleRefreshFeed}
						/>
					{/key}

					<IconButton
						icon="lucide:scan-search"
						title="Inspect feed XML"
						label="Inspect feed XML"
						iconClass="size-5"
						onclick={handleInspectFeed}
					/>
				</div>
			{/if}
		</div>

		<SearchBar bind:inputRef={searchInputRef} />
	</div>
</div>
