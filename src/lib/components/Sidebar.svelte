<script lang="ts">
	import type { SidebarSection } from '$lib/stores/app';
	import type { Feed } from '$lib/types/rss';

	type Props = {
		feeds: Feed[];
		refreshingFeedIds: string[];
		selectedFeedId: string | null;
		selectedSection: SidebarSection;
		onRemoveFeed: (feedId: string) => Promise<void>;
		onSelectFeed: (feedId: string | null) => void;
		onSelectSection: (section: SidebarSection) => void;
	};

	let {
		feeds,
		refreshingFeedIds,
		selectedFeedId,
		selectedSection,
		onRemoveFeed,
		onSelectFeed,
		onSelectSection
	}: Props = $props();

	const sections: Array<{ id: SidebarSection; label: string }> = [
		{ id: 'all', label: 'All feeds' },
		{ id: 'unread', label: 'Unread' },
		{ id: 'podcasts', label: 'Podcasts' },
		{ id: 'saved', label: 'Saved' },
		{ id: 'settings', label: 'Settings' }
	];
</script>

<aside
	class="w-full border-b border-slate-200 bg-white/85 p-4 backdrop-blur lg:w-80 lg:border-r lg:border-b-0 lg:p-6 dark:border-slate-800 dark:bg-slate-950/85"
>
	<div class="flex items-center justify-between gap-3">
		<div>
			<p class="text-xs font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400">
				Workspace
			</p>
			<h2 class="mt-1 text-xl font-semibold text-slate-950 dark:text-white">Library</h2>
		</div>
		<span
			class="rounded-full bg-slate-100 px-3 py-1 text-xs font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-300"
		>
			{feeds.length} feeds
		</span>
	</div>

	<nav class="mt-6 grid gap-2">
		{#each sections as section (section.id)}
			<button
				class={`flex items-center justify-between rounded-2xl px-4 py-3 text-left text-sm font-medium transition ${selectedSection === section.id && selectedFeedId === null ? 'bg-slate-200 text-slate-950 dark:bg-slate-800 dark:text-white' : 'text-slate-600 hover:bg-slate-100 hover:text-slate-950 dark:text-slate-300 dark:hover:bg-slate-900 dark:hover:text-white'}`}
				type="button"
				onclick={() => onSelectSection(section.id)}
			>
				<span>{section.label}</span>
				{#if section.id === 'all'}
					<span class="text-xs text-slate-400 dark:text-slate-500">Home</span>
				{/if}
			</button>
		{/each}
	</nav>

	<section class="mt-8">
		<div class="mb-3 flex items-center justify-between gap-2">
			<h3
				class="text-xs font-medium tracking-[0.18em] text-slate-500 uppercase dark:text-slate-400"
			>
				My feeds
			</h3>
			{#if selectedFeedId}
				<button
					class="text-xs font-medium text-slate-500 transition hover:text-slate-900 dark:text-slate-400 dark:hover:text-white"
					type="button"
					onclick={() => onSelectFeed(null)}
				>
					Clear
				</button>
			{/if}
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
						class="group flex items-center gap-2 rounded-2xl border border-transparent p-1 transition hover:border-slate-200 dark:hover:border-slate-800"
					>
						<button
							class={`min-w-0 flex-1 rounded-[1rem] px-3 py-3 text-left transition ${selectedFeedId === feed.id ? 'bg-slate-200 text-slate-950 dark:bg-slate-800 dark:text-white' : 'hover:bg-slate-100 dark:hover:bg-slate-900'}`}
							type="button"
							onclick={() => onSelectFeed(feed.id)}
						>
							<p class="truncate text-sm font-medium text-slate-900 dark:text-slate-100">
								{feed.title}
							</p>
							<p class="mt-1 truncate text-xs text-slate-500 dark:text-slate-400">
								{feed.kind === 'podcast' ? 'Podcast' : 'Feed'}
								{#if refreshingFeedIds.includes(feed.id)}
									• Syncing...
								{/if}
							</p>
						</button>
						<button
							class="rounded-xl px-3 py-2 text-xs font-medium text-slate-400 transition hover:bg-slate-100 hover:text-slate-900 dark:hover:bg-slate-900 dark:hover:text-white"
							type="button"
							onclick={(event) => {
								event.stopPropagation();
								void onRemoveFeed(feed.id);
							}}
						>
							Remove
						</button>
					</div>
				{/each}
			{/if}
		</div>
	</section>
</aside>
