<script lang="ts">
	import { Navigation } from '@skeletonlabs/skeleton-svelte';
	import type { Feed, Station } from '$lib/types/rss';
	import type { SidebarSection } from '$lib/stores/app.svelte';
	import { openFeedContextMenu } from '$lib/utils/tauri-menu';

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

	const sections = [
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
	] as const;

	function isActiveSection(sectionId: string): boolean {
		return selectedSection === sectionId && selectedFeedId === null;
	}
</script>

<div class="absolute inset-y-0 left-0 z-20 hidden md:block">
	<Navigation
		layout={isCollapsed ? 'rail' : 'sidebar'}
		class="h-full border-r border-border bg-surface"
	>
		<Navigation.Content class="flex h-full flex-col overflow-y-auto">
			<Navigation.Header class="sticky top-0 z-10 border-b border-border bg-surface p-4">
				<button
					type="button"
					onclick={onToggleCollapse}
					title={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
					aria-label={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
					class={`btn preset-filled ${
						isCollapsed ? 'aspect-square w-full justify-center px-0' : 'w-full justify-start gap-2'
					}`}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
						class={`size-5 transition-transform duration-300 ${isCollapsed ? 'rotate-180' : 'rotate-0'}`}
					>
						<path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5" />
					</svg>

					{#if !isCollapsed}
						<span>Library</span>
					{/if}
				</button>
			</Navigation.Header>

			<Navigation.Menu class="flex-1 space-y-6 p-2">
				<div class="space-y-1">
					{#each sections as section (section.id)}
						<button
							type="button"
							onclick={() => onSelectSection(section.id as SidebarSection)}
							title={section.label}
							class={`btn ${
								isCollapsed
									? 'aspect-square w-full justify-center px-0'
									: 'w-full justify-start gap-3'
							} ${
								isActiveSection(section.id)
									? 'preset-tonal text-fg'
									: 'hover:preset-tonal text-fg-muted hover:text-fg'
							}`}
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="size-5 shrink-0"
							>
								{#each section.paths as path (path)}
									<path stroke-linecap="round" stroke-linejoin="round" d={path} />
								{/each}
							</svg>

							{#if !isCollapsed}
								<span>{section.label}</span>
							{/if}
						</button>
					{/each}
				</div>

				<div class="border-t border-border pt-4">
					{#if !isCollapsed}
						<h2 class="mb-3 px-3 text-xs font-semibold tracking-[0.18em] text-fg-muted uppercase">
							My feeds
						</h2>
					{/if}

					<div class="space-y-1">
						{#if feeds.length === 0 && !isCollapsed}
							<div class="px-3 py-3 text-sm text-fg-muted">No feeds added yet</div>
						{/if}

						{#each feeds as feed (feed.id)}
							{@const isSelected = selectedFeedId === feed.id}
							{@const isRefreshing = refreshingFeedIds.includes(feed.id)}

							<div class={`group flex items-center ${isCollapsed ? 'justify-center' : ''}`}>
								<button
									type="button"
									onclick={() => onSelectFeed(feed.id)}
									oncontextmenu={(e) => void openFeedContextMenu(e, feed)}
									title={feed.title}
									class={`btn min-w-0 ${
										isCollapsed
											? 'size-12 overflow-hidden rounded-2xl p-0'
											: 'w-full justify-start gap-3'
									} ${
										isSelected
											? 'preset-tonal text-fg'
											: 'hover:preset-tonal text-fg-muted hover:text-fg'
									}`}
								>
									{#if isCollapsed}
										{#if feed.imageUrl}
											<img src={feed.imageUrl} alt={feed.title} class="size-full object-cover" />
										{:else}
											<span
												class="flex size-full items-center justify-center bg-accent text-sm font-semibold text-fg-inverse"
											>
												{(feed.title?.trim()?.[0] ?? '?').toUpperCase()}
											</span>
										{/if}
									{:else}
										{#if feed.imageUrl}
											<img
												src={feed.imageUrl}
												alt={feed.title}
												class="size-8 shrink-0 rounded-lg object-cover"
											/>
										{:else}
											<span
												class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-accent text-xs font-semibold text-fg-inverse"
											>
												{(feed.title?.trim()?.[0] ?? '?').toUpperCase()}
											</span>
										{/if}

										<span class="min-w-0 flex-1 text-left">
											<span class="block truncate text-sm font-medium">{feed.title}</span>
											<span class="block truncate text-xs text-fg-muted">
												{feed.kind === 'media' ? 'Podcast' : 'Feed'}
												{#if isRefreshing}
													• Syncing…
												{:else if feed.lastFetchedAt}
													• Local
												{/if}
											</span>
										</span>
									{/if}
								</button>

								{#if !isCollapsed}
									<button
										type="button"
										title="Open feed context menu"
										aria-label={`Open ${feed.title} context menu`}
										class="btn-icon hover:preset-tonal ml-1 shrink-0 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
										onclick={(e) => void openFeedContextMenu(e, feed)}
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
								{/if}
							</div>
						{/each}
					</div>
				</div>

				<div class="border-t border-border pt-4">
					<div
						class={`mb-3 flex items-center ${isCollapsed ? 'justify-center' : 'justify-between px-3'}`}
					>
						{#if !isCollapsed}
							<h2 class="text-xs font-semibold tracking-[0.18em] text-fg-muted uppercase">
								Stations
							</h2>
						{/if}

						<button
							type="button"
							title="New station"
							aria-label="Create new station"
							class="btn-icon hover:preset-tonal"
							onclick={onCreateStation}
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="2"
								stroke="currentColor"
								class="size-4"
							>
								<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
							</svg>
						</button>
					</div>

					<div class="space-y-1">
						{#if stations.length === 0 && !isCollapsed}
							<div class="px-3 py-2 text-sm text-fg-muted">No stations yet</div>
						{/if}

						{#each stations as station (station.id)}
							{@const isSelected = selectedStationId === station.id}

							<button
								type="button"
								onclick={() => onSelectStation(station.id)}
								title={station.name}
								class={`btn min-w-0 ${
									isCollapsed
										? 'size-12 justify-center rounded-2xl px-0'
										: 'w-full justify-start gap-3'
								} ${
									isSelected
										? 'preset-tonal text-fg'
										: 'hover:preset-tonal text-fg-muted hover:text-fg'
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

								{#if !isCollapsed}
									<span class="min-w-0 flex-1 text-left">
										<span class="block truncate text-sm font-medium">{station.name}</span>
										<span class="block truncate text-xs text-fg-muted">
											{station.feedIds.length}
											{station.feedIds.length === 1 ? 'podcast' : 'podcasts'}
										</span>
									</span>
								{/if}
							</button>
						{/each}
					</div>
				</div>
			</Navigation.Menu>
		</Navigation.Content>
	</Navigation>
</div>
