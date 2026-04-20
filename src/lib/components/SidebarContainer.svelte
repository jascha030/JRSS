<script lang="ts">
	import type { Feed, Station } from '$lib/types/rss';
	import type { SidebarSection } from '$lib/stores/app.svelte';

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
		refreshingFeedIds,
		isCollapsed
	}: Props = $props();
</script>

<div class="absolute inset-y-0 left-0 z-20 hidden md:block">
	<aside
		class="relative hidden h-full w-72 shrink-0 overflow-hidden border-r border-border bg-surface md:block"
	>
		<!-- single scroll container for both rail and panel -->
		<div class="flex h-full overflow-y-auto">
			<!-- rail -->
			<div class="flex w-24 shrink-0 flex-col bg-surface">
				<div
					class="sticky top-0 z-10 flex h-20 shrink-0 items-center justify-center border-b border-border bg-surface"
				>
					<button
						type="button"
						onclick={onToggleCollapse}
						title={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
						aria-label={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
						class="flex size-12 items-center justify-center rounded-2xl bg-accent text-fg-inverse transition-colors hover:bg-accent-hover"
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

				<div class="flex-1 py-3 border-r border-border">
					<div class="space-y-1 px-2">
						{@render sidebarRailSections()}
					</div>

					<div class="mt-6 border-t border-border px-2 pt-4">
						<div class="mt-6.5 space-y-2">
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
				class={`w-48 flex-1 shrink-0 transform-gpu bg-surface transition-[transform,opacity] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] will-change-transform ${
					isCollapsed
						? 'pointer-events-none -translate-x-full opacity-0'
						: 'translate-x-0 opacity-100'
				}`}
			>
				<div
					class="sticky top-0 z-10 flex h-20 shrink-0 items-center border-b border-border bg-surface px-4"
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
						<h2 class="mb-3 px-3 text-xs font-semibold tracking-[0.18em] text-fg-muted uppercase">
							My feeds
						</h2>

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
		{
			id: 'all',
			label: 'All feeds',
			paths: [
				'm2.25 12 8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25'
			]
		},
		{
			id: 'unread',
			label: 'Unread',
			paths: [
				'M2.25 13.5h3.86a2.25 2.25 0 0 1 2.012 1.244l.256.512a2.25 2.25 0 0 0 2.013 1.244h3.218a2.25 2.25 0 0 0 2.013-1.244l.256-.512a2.25 2.25 0 0 1 2.013-1.244h3.859m-19.5.338V18a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18v-4.162c0-.224-.034-.447-.1-.661L19.24 5.338a2.25 2.25 0 0 0-2.15-1.588H6.911a2.25 2.25 0 0 0-2.15 1.588L2.35 13.177a2.25 2.25 0 0 0-.1.661Z'
			]
		},
		{
			id: 'media',
			label: 'Media',
			paths: [
				'M12 18.75a6 6 0 0 0 6-6v-1.5m-6 7.5a6 6 0 0 1-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 0 1-3-3V4.5a3 3 0 1 1 6 0v8.25a3 3 0 0 1-3 3Z'
			]
		},
		{
			id: 'settings',
			label: 'Settings',
			paths: [
				'M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z',
				'M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z'
			]
		}
	]}

	{#each sections as section (section.id)}
		{@const isActive = selectedSection === section.id && selectedFeedId === null}
		<button
			type="button"
			onclick={() => onSelectSection(section.id as SidebarSection)}
			title={section.label}
			class={`mx-auto flex h-12 w-14 items-center justify-center rounded-2xl transition-colors ${
				isActive
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
				class="size-5"
			>
				{#each section.paths as path (path)}
					<path stroke-linecap="round" stroke-linejoin="round" d={path} />
				{/each}
			</svg>
		</button>
	{/each}
{/snippet}

{#snippet sidebarFeedButtons()}
	{#each feeds as feed (feed.id)}
		<button
			type="button"
			onclick={() => onSelectFeed(feed.id)}
			title={feed.title}
			class={`mx-auto flex size-12 items-center justify-center overflow-hidden rounded-2xl text-sm font-semibold shadow-sm transition-transform hover:scale-[1.02] ${
				selectedFeedId === feed.id ? 'ring-2 ring-accent ring-offset-2 ring-offset-surface' : ''
			}`}
		>
			{#if feed.imageUrl}
				<img src={feed.imageUrl} alt={feed.title} class="size-full object-cover" />
			{:else}
				<span
					class={`flex size-full items-center justify-center text-fg-inverse ${
						selectedFeedId === feed.id
							? 'bg-linear-to-br from-indigo-500 to-indigo-700'
							: 'bg-linear-to-br from-indigo-400 to-indigo-600'
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
			class={`mx-auto flex size-12 items-center justify-center overflow-hidden rounded-2xl text-sm font-semibold shadow-sm transition-transform hover:scale-[1.02] ${
				selectedStationId === station.id
					? 'ring-2 ring-accent ring-offset-2 ring-offset-surface'
					: ''
			}`}
		>
			<span
				class={`flex size-full items-center justify-center text-fg-inverse ${
					selectedStationId === station.id
						? 'bg-linear-to-br from-emerald-500 to-emerald-700'
						: 'bg-linear-to-br from-emerald-400 to-emerald-600'
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
			class={`flex h-12 w-full items-center rounded-2xl px-3 text-sm font-medium transition-colors ${
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
				class={`flex h-12 min-w-0 flex-1 items-center rounded-2xl px-3 py-2.5 text-left transition-colors ${
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
			class={`flex h-12 w-full items-center gap-2.5 rounded-2xl px-3 py-2.5 text-left transition-colors ${
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
				class="size-4 shrink-0 text-emerald-500"
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
