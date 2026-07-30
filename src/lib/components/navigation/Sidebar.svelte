<script lang="ts">
	import { STATION_GRADIENTS } from '$lib/components/station/station-gradients';
	import type { SidebarSection } from '$lib/state';
	import { feedsState, stationsState, selection } from '$lib/state';
	import {
		appUi,
		toggleSidebar,
		openFeedEditor,
		openStationEditor
	} from '$lib/hooks/useAppUi.svelte';
	import { openFeedContextMenu, openStationContextMenu } from '$lib/utils/tauri-menu';
	import {
		navigateToHome,
		navigateToSettings,
		navigateToSection,
		navigateToFeed,
		navigateToStation,
		isAppListSection
	} from '$lib/utils/navigation/app-router';
	import Icon from '@iconify/svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import { feedImageUrls } from '$lib/state';

	const feeds = $derived(feedsState.feeds);
	const stations = $derived(stationsState.stations);
	const selectedFeedId = $derived(selection.selectedFeedId);
	const selectedStationId = $derived(selection.selectedStationId);
	const selectedSection = $derived(selection.selectedSection);
	const refreshingFeedIds = $derived(feedsState.syncingFeedIds);
	const isCollapsed = $derived(appUi.isSidebarCollapsed);

	type SidebarNavSection = Exclude<SidebarSection, null>;

	const sidebarSections: Array<{ id: SidebarNavSection; label: string; icon: string }> = [
		{ id: 'home', label: 'Home', icon: 'heroicons:home' },
		{ id: 'all', label: 'All feeds', icon: 'heroicons:squares-2x2' },
		{ id: 'unread', label: 'Unread', icon: 'heroicons:inbox' },
		{ id: 'media', label: 'Media', icon: 'heroicons:microphone' },
		{ id: 'favorites', label: 'Favorites', icon: 'heroicons:heart' },
		{ id: 'settings', label: 'Settings', icon: 'heroicons:cog-6-tooth' }
	];

	const sidebarPanelNavSections = sidebarSections.map(({ id, label }) => ({ id, label }));

	function handleSelectSection(section: SidebarSection) {
		if (section === 'home') {
			void navigateToHome();
			return;
		}
		if (section === 'settings') {
			void navigateToSettings();
			return;
		}
		if (section && isAppListSection(section)) {
			void navigateToSection(section);
		}
	}

	function handleSelectFeed(feedId: string | null) {
		if (feedId === null) {
			void navigateToSection('all');
			return;
		}
		void navigateToFeed(feedId);
	}

	function handleSelectStation(stationId: string) {
		void navigateToStation(stationId);
	}
</script>

<div class="h-full w-full">
	<aside class="relative h-full w-full overflow-hidden border-r border-border bg-surface-sidebar">
		<div class="scrollbar-none flex h-full overflow-y-auto">
			<div class="flex w-16 shrink-0 flex-col bg-surface-sidebar">
				<div
					class="sticky top-0 z-10 flex h-16 shrink-0 items-center justify-center border-r border-b border-border bg-surface-shell-opaque"
				>
					<button
						type="button"
						onclick={toggleSidebar}
						oncontextmenu={(e) => e.preventDefault()}
						title={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
						aria-label={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
						class="flex size-9 items-center justify-center rounded-xl bg-accent text-fg-inverse transition-colors hover:bg-accent-hover"
					>
						<Icon
							icon="lucide:chevron-left"
							class={`size-5 transition-transform duration-300 ${isCollapsed ? 'rotate-180' : 'rotate-0'}`}
						/>
					</button>
				</div>

				<div class="flex-1 border-r border-border py-3">
					<div class="space-y-1 px-2">
						{@render sidebarRailSections()}
					</div>

					<div class="mt-6 border-t border-border px-2 pt-4">
						<div class="mt-9 space-y-2">
							{@render sidebarFeedButtons()}
						</div>
					</div>

					{#if stations.length > 0}
						<div class="mt-6 border-t border-border px-2 pt-4">
							<div class="mt-8.75 space-y-2">
								{@render sidebarStationButtons()}
							</div>
						</div>
					{/if}
				</div>
			</div>

			<div
				class={`w-44 shrink-0 transform-gpu bg-surface-sidebar transition-[transform,opacity] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] will-change-transform ${
					isCollapsed
						? 'pointer-events-none -translate-x-full opacity-0'
						: 'translate-x-0 opacity-100'
				}`}
			>
				<div
					class="sticky top-0 z-10 flex h-16 shrink-0 items-center border-b border-border bg-surface-shell-opaque px-3"
				>
					<div class="min-w-0">
						<h1 class="truncate text-base font-semibold text-fg">Library</h1>
						<p class="mt-1 text-xs text-fg-muted">
							{feeds.length}
							{feeds.length === 1 ? 'feed' : 'feeds'}
						</p>
					</div>
				</div>

				<div class="flex-1 py-3">
					<div class="space-y-1 px-2">
						{@render sidebarPanelSections()}
					</div>

					<div class="mt-6 border-t border-border px-2 pt-4">
						<div class="mb-3 flex items-center justify-between px-3">
							<h2 class="text-xs font-semibold tracking-[0.18em] text-fg-muted uppercase">Feeds</h2>
							<IconButton
								icon="lucide:plus"
								variant="ghost"
								class="size-6 rounded-md text-fg-muted hover:bg-surface-hover hover:text-fg"
								iconClass="size-3.5"
								title="Add feed"
								label="Add new feed"
								onclick={openFeedEditor}
							/>
						</div>

						<div class="space-y-1">
							{#if feeds.length === 0}
								<div class="px-3 py-3 text-sm text-fg-muted">No feeds added yet</div>
							{:else}
								{@render feedList()}
							{/if}
						</div>
					</div>

					<div class="mt-6 border-t border-border px-2 pt-4">
						<div class="mb-3 flex items-center justify-between px-3">
							<h2 class="text-xs font-semibold tracking-[0.18em] text-fg-muted uppercase">
								Stations
							</h2>
							<IconButton
								icon="lucide:plus"
								variant="ghost"
								class="size-6 rounded-md text-fg-muted hover:bg-surface-hover hover:text-fg"
								iconClass="size-3.5"
								title="New station"
								label="Create new station"
								onclick={() => openStationEditor()}
							/>
						</div>

						<div class="space-y-1">
							{#if stations.length === 0}
								<div class="px-3 py-2 text-sm text-fg-muted">No stations yet</div>
							{:else}
								{@render stationList()}
							{/if}
						</div>
					</div>
				</div>
			</div>
		</div>
	</aside>
</div>

{#snippet sidebarRailSections()}
	{#each sidebarSections as section (section.id)}
		{@const isActive = selectedSection === section.id && selectedFeedId === null}
		<button
			type="button"
			onclick={() => handleSelectSection(section.id)}
			oncontextmenu={(e) => e.preventDefault()}
			title={section.label}
			aria-label={section.label}
			class={`mx-auto flex h-10 w-11 items-center justify-center rounded-xl transition-colors ${
				isActive
					? 'bg-surface-sidebar-active text-fg'
					: 'text-fg-muted hover:bg-surface-sidebar-hover hover:text-fg'
			}`}
		>
			<Icon icon={section.icon} class="size-4" />
		</button>
	{/each}
{/snippet}

{#snippet sidebarFeedButtons()}
	{#each feeds as feed (feed.id)}
		<button
			type="button"
			onclick={() => handleSelectFeed(feed.id)}
			oncontextmenu={(e) => void openFeedContextMenu(e, feed)}
			title={feed.title}
			aria-label={feed.title}
			class={`mx-auto flex size-10 items-center justify-center overflow-hidden rounded-xl text-xs font-semibold shadow-sm transition-transform hover:scale-[1.02] ${
				selectedFeedId === feed.id ? 'ring-2 ring-accent ring-offset-2 ring-offset-transparent' : ''
			}`}
		>
			{#if feed.imageUrl}
				<img
					src={feedImageUrls[feed.id] || feed.imageUrl}
					alt={feed.title}
					class="size-full object-cover"
					loading="lazy"
				/>
			{:else}
				<span
					class={`flex size-full items-center justify-center text-fg-inverse ${
						selectedFeedId === feed.id
							? 'bg-linear-to-br from-primary-500 to-primary-700'
							: 'bg-linear-to-br from-primary-400 to-primary-600'
					}`}
				>
					{(feed.title?.trim()?.[0] ?? '?').toUpperCase()}
				</span>
			{/if}
		</button>
	{/each}
{/snippet}

{#snippet sidebarStationButtons()}
	{#each stations as station (station.id)}
		<button
			type="button"
			onclick={() => handleSelectStation(station.id)}
			oncontextmenu={(e) => void openStationContextMenu(e, station)}
			title={station.name}
			aria-label={station.name}
			class={`mx-auto flex size-10 items-center justify-center overflow-hidden rounded-xl text-xs font-semibold shadow-sm transition-transform hover:scale-[1.02] ${
				selectedStationId === station.id
					? 'ring-2 ring-accent ring-offset-2 ring-offset-transparent'
					: ''
			}`}
		>
			<span
				class={`flex size-full items-center justify-center bg-linear-to-br text-fg-inverse ${selectedStationId === station.id ? `${STATION_GRADIENTS[station.gradient].activeFrom} ${STATION_GRADIENTS[station.gradient].activeTo}` : `${STATION_GRADIENTS[station.gradient].from} ${STATION_GRADIENTS[station.gradient].to}`}`}
			>
				<Icon icon="lucide:mic" class="size-5" />
			</span>
		</button>
	{/each}
{/snippet}

{#snippet sidebarPanelSections()}
	{#each sidebarPanelNavSections as section (section.id)}
		{@const isActive = selectedSection === section.id && selectedFeedId === null}
		<button
			type="button"
			onclick={() => handleSelectSection(section.id)}
			oncontextmenu={(e) => e.preventDefault()}
			aria-label={section.label}
			class={`flex h-10 w-full items-center rounded-xl px-3 text-sm font-medium transition-colors ${
				isActive
					? 'bg-surface-sidebar-active text-fg'
					: 'text-fg-muted hover:bg-surface-sidebar-hover hover:text-fg'
			}`}
		>
			{section.label}
		</button>
	{/each}
{/snippet}

{#snippet feedList()}
	{#each feeds as feed (feed.id)}
		{@const isRefreshing = refreshingFeedIds.includes(feed.id)}
		<div
			class={`group mb-2 flex items-center rounded-xl transition-colors ${
				selectedFeedId === feed.id
					? 'bg-surface-sidebar-active text-fg'
					: 'text-fg-muted hover:bg-surface-sidebar-hover hover:text-fg'
			}`}
		>
			<button
				type="button"
				onclick={() => handleSelectFeed(feed.id)}
				oncontextmenu={(e) => void openFeedContextMenu(e, feed)}
				aria-label={feed.title}
				class="flex h-10 min-w-0 flex-1 items-center px-3 py-2 text-left"
			>
				<span class="min-w-0 flex-1">
					<span class="block truncate text-sm font-medium">{feed.title}</span>
					<span class="block truncate text-xs text-fg-muted">
						{feed.kind === 'media' ? 'Podcast' : 'Feed'}
						{#if isRefreshing}
							• Syncing...
						{:else if feed.lastFetchedAt}
							• Local
						{/if}
					</span>
				</span>
			</button>

			<button
				type="button"
				title="Open feed context menu"
				onclick={(e) => void openFeedContextMenu(e, feed)}
				aria-label={`Open ${feed.title} context menu`}
				class="ml-1 flex size-9 shrink-0 items-center justify-center rounded-lg text-fg-subtle transition-[opacity,background-color,color] duration-150 group-hover:text-fg-secondary"
			>
				<Icon icon="heroicons:ellipsis-vertical" class="size-4" />
			</button>
		</div>
	{/each}
{/snippet}

{#snippet stationList()}
	{#each stations as station (station.id)}
		<div
			class={`group mb-2 flex items-center rounded-xl transition-colors ${
				selectedStationId === station.id
					? 'bg-surface-sidebar-active text-fg'
					: 'text-fg-muted hover:bg-surface-sidebar-hover hover:text-fg'
			}`}
		>
			<button
				type="button"
				onclick={() => handleSelectStation(station.id)}
				oncontextmenu={(e) => void openStationContextMenu(e, station)}
				aria-label={station.name}
				class="flex h-10 min-w-0 flex-1 items-center gap-2.5 px-3 py-2 text-left"
			>
				<Icon icon="heroicons:microphone" class="size-4 shrink-0 text-success-600" />

				<span class="min-w-0 flex-1">
					<span class="block truncate text-sm font-medium">{station.name}</span>
					<span class="block truncate text-xs text-fg-muted">
						{station.feedIds.length}
						{station.feedIds.length === 1 ? 'podcast' : 'podcasts'}
					</span>
				</span>
			</button>

			<button
				type="button"
				title="Open station context menu"
				onclick={(e) => void openStationContextMenu(e, station)}
				aria-label={`Open ${station.name} context menu`}
				class="ml-1 flex size-9 shrink-0 items-center justify-center rounded-lg text-fg-subtle transition-[opacity,background-color,color] duration-150 group-hover:text-fg-secondary"
			>
				<Icon icon="heroicons:ellipsis-vertical" class="size-4" />
			</button>
		</div>
	{/each}
{/snippet}
