<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import type { Feed, Station } from '$lib/types/rss';
	import type { SidebarSection } from '$lib/stores/app.svelte';
	import { openFeedContextMenu } from '$lib/utils/tauri-menu';
	import Icon from '@iconify/svelte';

	type Props = {
		feeds: Feed[];
		stations: Station[];
		selectedFeedId: string | null;
		selectedStationId: string | null;
		selectedSection: SidebarSection;
		onSelectFeed: (feedId: string | null) => void;
		onSelectSection: (section: SidebarSection) => void;
		onSelectStation: (stationId: string) => void;
		onToggleCollapse: () => void;
		onCreateStation: () => void;
		onAddFeed: () => void;
		refreshingFeedIds: string[];
		isCollapsed: boolean;
	};

	let {
		feeds,
		stations,
		selectedFeedId,
		selectedStationId,
		selectedSection,
		onSelectFeed,
		onSelectSection,
		onSelectStation,
		onToggleCollapse,
		onCreateStation,
		onAddFeed,
		refreshingFeedIds,
		isCollapsed
	}: Props = $props();

	onMount(() => {
		let unlistenSettings: UnlistenFn | undefined;
		let unlistenNewStation: UnlistenFn | undefined;

		const setupListeners = async () => {
			unlistenSettings = await listen('menu-settings', () => {
				onSelectSection('settings');
			});
			unlistenNewStation = await listen('menu-new-station', () => {
				onCreateStation();
			});
		};

		void setupListeners();

		return () => {
			if (unlistenSettings) {
				unlistenSettings();
			}
			if (unlistenNewStation) {
				unlistenNewStation();
			}
		};
	});
</script>

<div class="absolute inset-y-0 left-0 z-20 hidden md:block">
	<aside
		class="relative hidden h-full w-60 shrink-0 overflow-hidden border-r border-border bg-surface md:block"
	>
		<!-- single scroll container for both rail and panel -->
		<div class="flex h-full overflow-y-auto">
			<!-- rail -->
			<div class="flex w-16 shrink-0 flex-col bg-surface">
				<div
					class="sticky top-0 z-10 flex h-16 shrink-0 items-center justify-center border-r border-b border-border bg-surface"
				>
					<button
						type="button"
						onclick={onToggleCollapse}
						title={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
						aria-label={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
						class="flex size-9 items-center justify-center rounded-xl bg-accent text-fg-inverse transition-colors hover:bg-accent-hover"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.8"
							class={`size-5 transition-transform duration-300 ${isCollapsed ? 'rotate-180' : 'rotate-0'}`}
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M15.75 19.5 8.25 12l7.5-7.5"
							/>
						</svg>
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

			<!-- sliding panel -->
			<div
				class={`w-44 flex-1 shrink-0 transform-gpu bg-surface transition-[transform,opacity] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] will-change-transform ${
					isCollapsed
						? 'pointer-events-none -translate-x-full opacity-0'
						: 'translate-x-0 opacity-100'
				}`}
			>
				<div
					class="sticky top-0 z-10 flex h-16 shrink-0 items-center border-b border-border bg-surface px-3"
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
							<h2 class="text-xs font-semibold tracking-[0.18em] text-fg-muted uppercase">
								My feeds
							</h2>
							<button
								type="button"
								title="Add feed"
								aria-label="Add new feed"
								class="flex size-6 items-center justify-center rounded-md text-fg-muted transition-colors hover:bg-surface-hover hover:text-fg"
								onclick={onAddFeed}
							>
								<Icon icon="lucide:plus" class="size-3.5" />
							</button>
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
							<button
								type="button"
								title="New station"
								aria-label="Create new station"
								class="flex size-6 items-center justify-center rounded-md text-fg-muted transition-colors hover:bg-surface-hover hover:text-fg"
								onclick={onCreateStation}
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									fill="none"
									viewBox="0 0 24 24"
									stroke-width="2"
									stroke="currentColor"
									class="size-3.5"
								>
									<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
								</svg>
							</button>
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
	{@const sections = [
		{ id: 'all', label: 'All feeds', icon: 'heroicons:home' },
		{ id: 'unread', label: 'Unread', icon: 'heroicons:inbox' },
		{ id: 'media', label: 'Media', icon: 'heroicons:microphone' },
		{ id: 'settings', label: 'Settings', icon: 'heroicons:cog-6-tooth' }
	]}

	{#each sections as section (section.id)}
		{@const isActive = selectedSection === section.id && selectedFeedId === null}
		<button
			type="button"
			onclick={() => onSelectSection(section.id as SidebarSection)}
			title={section.label}
			class={`mx-auto flex h-10 w-11 items-center justify-center rounded-xl transition-colors ${
				isActive
					? 'bg-surface-active text-fg'
					: 'text-fg-muted hover:bg-surface-hover hover:text-fg'
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
			onclick={() => onSelectFeed(feed.id)}
			oncontextmenu={(e) => void openFeedContextMenu(e, feed)}
			title={feed.title}
			class={`mx-auto flex size-10 items-center justify-center overflow-hidden rounded-xl text-xs font-semibold shadow-sm transition-transform hover:scale-[1.02] ${
				selectedFeedId === feed.id ? 'ring-2 ring-accent ring-offset-2 ring-offset-surface' : ''
			}`}
		>
			{#if feed.imageUrl}
				<img src={feed.imageUrl} alt={feed.title} class="size-full object-cover" />
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
			onclick={() => onSelectStation(station.id)}
			title={station.name}
			class={`mx-auto flex size-10 items-center justify-center overflow-hidden rounded-xl text-xs font-semibold shadow-sm transition-transform hover:scale-[1.02] ${
				selectedStationId === station.id
					? 'ring-2 ring-accent ring-offset-2 ring-offset-surface'
					: ''
			}`}
		>
			<span
				class={`flex size-full items-center justify-center text-fg-inverse ${
					selectedStationId === station.id
						? 'bg-linear-to-br from-success-500 to-success-700'
						: 'bg-linear-to-br from-success-400 to-success-600'
				}`}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
					class="size-5"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M12 18.75a6 6 0 0 0 6-6v-1.5m-6 7.5a6 6 0 0 1-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 0 1-3-3V4.5a3 3 0 1 1 6 0v8.25a3 3 0 0 1-3 3Z"
					/>
				</svg>
			</span>
		</button>
	{/each}
{/snippet}

{#snippet sidebarPanelSections()}
	{@const sections = [
		{ id: 'all', label: 'All feeds' },
		{ id: 'unread', label: 'Unread' },
		{ id: 'media', label: 'Media' },
		{ id: 'settings', label: 'Settings' }
	]}

	{#each sections as section (section.id)}
		{@const isActive = selectedSection === section.id && selectedFeedId === null}
		<button
			type="button"
			onclick={() => onSelectSection(section.id as SidebarSection)}
			class={`flex h-10 w-full items-center rounded-xl px-3 text-sm font-medium transition-colors ${
				isActive
					? 'bg-surface-active text-fg'
					: 'text-fg-muted hover:bg-surface-hover hover:text-fg'
			}`}
		>
			{section.label}
		</button>
	{/each}
{/snippet}

{#snippet feedList()}
	{#each feeds as feed (feed.id)}
		{@const isRefreshing = refreshingFeedIds.includes(feed.id)}
		<div class="group mb-2 flex items-center">
			<button
				type="button"
				onclick={() => onSelectFeed(feed.id)}
				oncontextmenu={(e) => void openFeedContextMenu(e, feed)}
				class={`flex h-10 min-w-0 flex-1 items-center rounded-xl px-3 py-2 text-left transition-colors ${
					selectedFeedId === feed.id
						? 'bg-surface-active text-fg'
						: 'text-fg-muted hover:bg-surface-hover hover:text-fg'
				}`}
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
				class="ml-1 flex size-9 shrink-0 items-center justify-center rounded-lg text-fg-subtle opacity-0 transition-[opacity,background-color,color] duration-150 group-hover:opacity-100 hover:bg-surface-hover hover:text-fg-secondary"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
					class="size-4"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M12 6.75a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5ZM12 12.75a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5ZM12 18.75a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5Z"
					/>
				</svg>
			</button>
		</div>
	{/each}
{/snippet}

{#snippet stationList()}
	{#each stations as station (station.id)}
		<button
			type="button"
			onclick={() => onSelectStation(station.id)}
			class={`flex h-10 w-full items-center gap-2.5 rounded-xl px-3 py-2 text-left transition-colors ${
				selectedStationId === station.id
					? 'bg-surface-active text-fg'
					: 'text-fg-muted hover:bg-surface-hover hover:text-fg'
			}`}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke-width="1.5"
				stroke="currentColor"
				class="size-4 shrink-0 text-success-600"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M12 18.75a6 6 0 0 0 6-6v-1.5m-6 7.5a6 6 0 0 1-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 0 1-3-3V4.5a3 3 0 1 1 6 0v8.25a3 3 0 0 1-3 3Z"
				/>
			</svg>
			<span class="min-w-0 flex-1">
				<span class="block truncate text-sm font-medium">{station.name}</span>
				<span class="block truncate text-xs text-fg-muted">
					{station.feedIds.length}
					{station.feedIds.length === 1 ? 'podcast' : 'podcasts'}
				</span>
			</span>
		</button>
	{/each}
{/snippet}
