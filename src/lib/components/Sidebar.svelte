<script lang="ts">
	import type { SidebarSection } from '$lib/stores/app';
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

	const sections: Array<{ id: SidebarSection; label: string }> = [
		{ id: 'all', label: 'All feeds' },
		{ id: 'unread', label: 'Unread' },
		{ id: 'podcasts', label: 'Podcasts' },
		{ id: 'settings', label: 'Settings' }
	];
</script>

<aside
	class={`flex h-full shrink-0 flex-col overflow-y-auto border-r border-slate-200 bg-white/85 p-4 backdrop-blur transition-[width,padding] duration-200 dark:border-slate-800 dark:bg-slate-950/85 ${collapsed ? 'w-20 px-3' : 'w-80 p-6'}`}
>
	<div class={`flex items-center gap-3 ${collapsed ? 'justify-center' : 'justify-between'}`}>
		{#if !collapsed}
			<h2 class="mt-1 text-xl font-semibold text-slate-950 dark:text-white">Library</h2>
		{/if}
		<button
			class="flex h-10 w-10 items-center justify-center rounded-2xl border border-slate-200 text-xs font-medium text-slate-500 transition hover:border-slate-300 hover:text-slate-900 dark:border-slate-800 dark:text-slate-400 dark:hover:border-slate-700 dark:hover:text-white"
			type="button"
			onclick={onToggleCollapse}
		>
			{collapsed ? '>>' : '<<'}
		</button>
	</div>

	{#if !collapsed}
		<span
			class="mt-4 inline-flex rounded-full bg-slate-100 px-3 py-1 text-xs font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-300"
		>
			{feeds.length} feeds
		</span>
	{/if}

	<nav class="mt-6 grid gap-2">
		{#each sections as section (section.id)}
			<button
				title={collapsed ? section.label : undefined}
				class={`flex items-center justify-between rounded-2xl px-4 py-3 text-left text-sm font-medium transition ${selectedSection === section.id && selectedFeedId === null ? 'bg-slate-200 text-slate-950 dark:bg-slate-800 dark:text-white' : 'text-slate-600 hover:bg-slate-100 hover:text-slate-950 dark:text-slate-300 dark:hover:bg-slate-900 dark:hover:text-white'}`}
				type="button"
				onclick={() => onSelectSection(section.id)}
			>
				<span class={`${collapsed ? 'mx-auto' : ''}`}>
					{collapsed ? section.label.charAt(0) : section.label}
				</span>
				{#if section.id === 'all' && !collapsed}
					<span class="text-xs text-slate-400 dark:text-slate-500">Home</span>
				{/if}
			</button>
		{/each}
	</nav>

	<section class="mt-8">
		<div class={`mb-3 flex items-center gap-2 ${collapsed ? 'justify-center' : 'justify-between'}`}>
			<h3
				class="text-xs font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400"
			>
				{collapsed ? 'Feeds' : 'My feeds'}
			</h3>
		</div>

		<div class="grid gap-2">
			{#if feeds.length === 0}
				<p
					class="rounded-2xl border border-dashed border-slate-300 px-4 py-5 text-sm text-slate-500 dark:border-slate-700 dark:text-slate-400"
				>
					No feeds added yet.
				</p>
			{:else}
				{#each feeds as feed (feed.id)}
					<div
						class={`group flex items-center gap-2 rounded-2xl border border-transparent p-1 transition hover:border-slate-200 dark:hover:border-slate-800 ${collapsed ? 'justify-center' : ''}`}
					>
						<button
							title={feed.title}
							class={`min-w-0 flex-1 rounded-[1rem] px-3 py-3 text-left transition ${selectedFeedId === feed.id ? 'bg-slate-200 text-slate-950 dark:bg-slate-800 dark:text-white' : 'hover:bg-slate-100 dark:hover:bg-slate-900'}`}
							type="button"
							onclick={() => onSelectFeed(feed.id)}
						>
							{#if collapsed}
								<div class="flex justify-center">
									<span
										class="flex h-9 w-9 items-center justify-center rounded-xl bg-slate-100 text-sm font-semibold text-slate-700 dark:bg-slate-800 dark:text-slate-200"
									>
										{feed.title.charAt(0).toUpperCase()}
									</span>
								</div>
							{:else}
								<p class="truncate text-sm font-medium text-slate-900 dark:text-slate-100">
									{feed.title}
								</p>
								<p class="mt-1 truncate text-xs text-slate-500 dark:text-slate-400">
									{feed.kind === 'podcast' ? 'Podcast' : 'Feed'}
									{#if refreshingFeedIds.includes(feed.id)}
										• Syncing...
									{:else if feed.lastFetchedAt}
										• Local
									{/if}
								</p>
							{/if}
						</button>
						{#if !collapsed}
							<button
								title="Remove feed"
								class="rounded-xl px-3 py-2 text-xs font-medium text-slate-400 transition hover:bg-slate-100 hover:text-slate-900 dark:hover:bg-slate-900 dark:hover:text-white"
								type="button"
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
									class="size-6"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										d="m9.75 9.75 4.5 4.5m0-4.5-4.5 4.5M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
									/>
								</svg>
							</button>
						{/if}
					</div>
				{/each}
			{/if}
		</div>
	</section>
</aside>
