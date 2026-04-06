<script lang="ts">
	import type { SidebarSection } from '$lib/stores/app.svelte';
	import type { Feed } from '$lib/types/rss';

	type Props = {
		collapsed: boolean;
		feeds: Feed[];
		refreshingFeedIds: string[];
		selectedFeedId: string | null;
		selectedSection: SidebarSection;
		onRemoveFeed: (feedId: string) => Promise<void>;
		onSelectFeed: (feedId: string | null) => void;
		onSelectSection: (section: SidebarSection) => void;
		onToggleCollapse: () => void;
	};

	let {
		collapsed,
		feeds,
		refreshingFeedIds,
		selectedFeedId,
		selectedSection,
		onRemoveFeed,
		onSelectFeed,
		onSelectSection,
		onToggleCollapse
	}: Props = $props();

	const sections: Array<{
		id: SidebarSection;
		label: string;
		paths: string[];
	}> = [
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
			id: 'podcasts',
			label: 'Podcasts',
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
	];

	function isSectionActive(sectionId: SidebarSection) {
		return selectedSection === sectionId && selectedFeedId === null;
	}

	function isRefreshing(feedId: string) {
		return refreshingFeedIds.includes(feedId);
	}

	function feedInitial(title: string) {
		return (title?.trim()?.[0] ?? '?').toUpperCase();
	}
</script>

<aside
	class="relative hidden h-full w-72 shrink-0 overflow-hidden border-r border-zinc-200 bg-white md:block dark:border-zinc-800 dark:bg-zinc-950"
>
	<!-- rail -->
	<div
		class="absolute inset-y-0 left-0 z-20 flex w-24 flex-col border-r border-zinc-200 bg-white dark:border-zinc-800 dark:bg-zinc-950"
	>
		<div
			class="flex h-20 shrink-0 items-center justify-center border-b border-zinc-200 dark:border-zinc-800"
		>
			<button
				type="button"
				onclick={onToggleCollapse}
				title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
				aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
				class="flex size-12 items-center justify-center rounded-2xl bg-indigo-600 text-white transition-colors hover:bg-indigo-500"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					class={`size-5 transition-transform duration-300 ${collapsed ? 'rotate-180' : 'rotate-0'}`}
				>
					<path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5 8.25 12l7.5-7.5" />
				</svg>
			</button>
		</div>

		<div class="flex-1 overflow-y-auto py-3">
			<div class="space-y-1 px-2">
				{#each sections as section (section.id)}
					<button
						type="button"
						onclick={() => onSelectSection(section.id)}
						title={section.label}
						class={`mx-auto flex h-12 w-14 items-center justify-center rounded-2xl transition-colors ${
							isSectionActive(section.id)
								? 'bg-zinc-100 text-zinc-900 dark:bg-zinc-800 dark:text-white'
								: 'text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-900 dark:hover:text-white'
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
			</div>

			<div class="mt-6 border-t border-zinc-200 px-2 pt-4 dark:border-zinc-800">
				<div class="space-y-2 mt-7">
					{#each feeds as feed (feed.id)}
						<button
							type="button"
							onclick={() => onSelectFeed(feed.id)}
							title={feed.title}
							class={`mx-auto flex size-12 items-center justify-center rounded-2xl text-sm font-semibold text-white shadow-sm transition-transform hover:scale-[1.02] ${
								selectedFeedId === feed.id
									? 'bg-linear-to-br from-indigo-500 to-indigo-700'
									: 'bg-linear-to-br from-indigo-400 to-indigo-600'
							}`}
						>
							{feedInitial(feed.title)}
						</button>
					{/each}
				</div>
			</div>
		</div>
	</div>

	<!-- sliding panel -->
	<div
		class={`absolute inset-y-0 left-24 z-10 w-48 transform-gpu bg-white transition-[transform,opacity] duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] will-change-transform dark:bg-zinc-950 ${
			collapsed ? 'pointer-events-none -translate-x-full opacity-0' : 'translate-x-0 opacity-100'
		}`}
	>
		<div class="flex h-20 shrink-0 items-center border-b border-zinc-200 px-4 dark:border-zinc-800">
			<div class="min-w-0">
				<h1 class="truncate text-base font-semibold text-zinc-900 dark:text-white">Library</h1>
				<p class="mt-1 text-xs text-zinc-500 dark:text-zinc-400">
					{feeds.length}
					{feeds.length === 1 ? 'feed' : 'feeds'}
				</p>
			</div>
		</div>

		<div class="flex-1 overflow-y-auto py-3">
			<div class="space-y-1 px-2">
				{#each sections as section (section.id)}
					<button
						type="button"
						onclick={() => onSelectSection(section.id)}
						class={`flex h-12 w-full items-center rounded-2xl px-3 text-sm font-medium transition-colors ${
							isSectionActive(section.id)
								? 'bg-zinc-100 text-zinc-900 dark:bg-zinc-800 dark:text-white'
								: 'text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-900 dark:hover:text-white'
						}`}
					>
						{section.label}
					</button>
				{/each}
			</div>

			<div class="mt-6 border-t border-zinc-200 px-2 pt-4 dark:border-zinc-800">
				<h2
					class="mb-3 px-3 text-xs font-semibold tracking-[0.18em] text-zinc-500 uppercase dark:text-zinc-400"
				>
					My feeds
				</h2>

				<div class="space-y-1">
					{#if feeds.length === 0}
						<div class="px-3 py-3 text-sm text-zinc-500 dark:text-zinc-400">No feeds added yet</div>
					{:else}
						{#each feeds as feed (feed.id)}
							<div class="group flex items-center">
								<button
									type="button"
									onclick={() => onSelectFeed(feed.id)}
									class={`flex min-w-0 flex-1 items-center rounded-2xl px-3 py-2.5 text-left transition-colors ${
										selectedFeedId === feed.id
											? 'bg-zinc-100 text-zinc-900 dark:bg-zinc-800 dark:text-white'
											: 'text-zinc-600 hover:bg-zinc-100 hover:text-zinc-900 dark:text-zinc-400 dark:hover:bg-zinc-900 dark:hover:text-white'
									}`}
								>
									<span class="min-w-0 flex-1">
										<span class="block truncate text-sm font-medium">{feed.title}</span>
										<span class="block truncate text-xs text-zinc-500 dark:text-zinc-400">
											{feed.kind === 'podcast' ? 'Podcast' : 'Feed'}
											{#if isRefreshing(feed.id)}
												• Syncing...
											{:else if feed.lastFetchedAt}
												• Local
											{/if}
										</span>
									</span>
								</button>

								<button
									type="button"
									title="Remove feed"
									aria-label={`Remove ${feed.title}`}
									class="ml-1 flex size-9 shrink-0 items-center justify-center rounded-lg text-zinc-400 opacity-0 transition-[opacity,background-color,color] duration-150 group-hover:opacity-100 hover:bg-zinc-100 hover:text-zinc-700 dark:hover:bg-zinc-900 dark:hover:text-zinc-200"
									onclick={(event) => {
										event.stopPropagation();
										void onRemoveFeed(feed.id);
									}}
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
											d="m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0"
										/>
									</svg>
								</button>
							</div>
						{/each}
					{/if}
				</div>
			</div>
		</div>
	</div>
</aside>
