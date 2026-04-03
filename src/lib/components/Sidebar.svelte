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

	const sections: Array<{ id: SidebarSection; label: string; icon: string }> = [
		{
			id: 'all',
			label: 'All feeds',
			icon: 'M3.459 1.5a.5.5 0 1 0 1 0H3.46zm4.776-1.5a.5.5 0 0 0-.5.5v6a.5.5 0 0 0 1 0v-6a.5.5 0 0 0-.5-.5zm2.961 5.5a.5.5 0 1 0-1 0v2a.5.5 0 0 0 1 0v-2zm2.05-5a.5.5 0 0 0-.5.5v6a.5.5 0 0 0 1 0v-6a.5.5 0 0 0-.5-.5zm7.003 0a.5.5 0 0 0-.5.5v6a.5.5 0 0 0 1 0v-6a.5.5 0 0 0-.5-.5z'
		},
		{
			id: 'unread',
			label: 'Unread',
			icon: 'M6.6915026,2.4744748 C6.4744532,2.2788952 6.1305528,2.2788952 5.9135034,2.4744748 L2.0151496,5.9114553 C1.8324289,6.0688169 1.8324289,6.3600423 2.0151496,6.5173722 L5.9135034,9.9967118 C6.1305528,10.1925405 6.4744532,10.1925405 6.6915026,9.9967118 L10.5898564,6.5173722 C10.7725771,6.3600423 10.7725771,6.0688169 10.5898564,5.9114553 Z M6.6915026,13.4744748 C6.4744532,13.2788952 6.1305528,13.2788952 5.9135034,13.4744748 L2.0151496,16.9114553 C1.8324289,17.0688169 1.8324289,17.3600423 2.0151496,17.5173722 L5.9135034,20.9967118 C6.1305528,21.1925405 6.4744532,21.1925405 6.6915026,20.9967118 L10.5898564,17.5173722 C10.7725771,17.3600423 10.7725771,17.0688169 10.5898564,16.9114553 Z'
		},
		{
			id: 'podcasts',
			label: 'Podcasts',
			icon: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5h3V9h4v3h3l-5 5z'
		},
		{
			id: 'settings',
			label: 'Settings',
			icon: 'M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.62l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.48.1.62l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.62l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.48-.1-.62l-2.03-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z'
		}
	];

	const getIconContent = (iconPath: string) => iconPath;
</script>

<aside
	class={`flex hidden h-screen shrink-0 flex-col overflow-y-auto border-r border-gray-200 bg-white transition-[width] duration-200 dark:border-gray-800 dark:bg-gray-950 ${collapsed ? 'w-20' : 'w-64'} md:flex`}
>
	<!-- Header -->
	<div
		class="flex h-16 shrink-0 items-center gap-2 border-b border-gray-200 px-4 dark:border-gray-800"
	>
		{#if !collapsed}
			<div class="flex flex-1 items-center gap-2">
				<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-indigo-600 text-white">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="currentColor"
						class="h-5 w-5"
					>
						<path d="M12 2L2 7v10c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V7l-10-5z" />
					</svg>
				</div>
				<h1 class="text-lg font-semibold text-gray-900 dark:text-white">Library</h1>
			</div>
		{/if}
		<button
			class="ml-auto flex h-8 w-8 items-center justify-center rounded-md text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-900 dark:hover:text-gray-300"
			type="button"
			onclick={onToggleCollapse}
			title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 20 20"
				fill="currentColor"
				class="h-5 w-5"
			>
				{#if collapsed}
					<path
						fill-rule="evenodd"
						d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
						clip-rule="evenodd"
					/>
				{:else}
					<path
						fill-rule="evenodd"
						d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"
						clip-rule="evenodd"
					/>
				{/if}
			</svg>
		</button>
	</div>

	<!-- Feed count badge -->
	{#if !collapsed}
		<div class="px-4 py-3">
			<span
				class="inline-flex items-center rounded-full bg-indigo-50 px-3 py-1 text-xs font-medium text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-400"
			>
				{feeds.length}
				{feeds.length === 1 ? 'feed' : 'feeds'}
			</span>
		</div>
	{/if}

	<!-- Main navigation -->
	<nav class="flex-1 space-y-1 px-2 py-4">
		{#each sections as section (section.id)}
			<button
				title={collapsed ? section.label : undefined}
				class={`group flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition ${
					selectedSection === section.id && selectedFeedId === null
						? 'bg-gray-100 text-gray-900 dark:bg-gray-900 dark:text-white'
						: 'text-gray-600 hover:bg-gray-50 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-900/50 dark:hover:text-white'
				}`}
				type="button"
				onclick={() => onSelectSection(section.id)}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="currentColor"
					class="h-5 w-5 shrink-0"
				>
					<path d={section.icon} />
				</svg>
				{#if !collapsed}
					<span>{section.label}</span>
				{/if}
			</button>
		{/each}
	</nav>

	<!-- Feeds section -->
	<div class="border-t border-gray-200 px-2 py-4 dark:border-gray-800">
		{#if !collapsed}
			<h3
				class="mb-3 px-3 text-xs font-semibold tracking-wider text-gray-500 uppercase dark:text-gray-400"
			>
				My Feeds
			</h3>
		{/if}

		<div class="space-y-1">
			{#if feeds.length === 0}
				{#if !collapsed}
					<p
						class="rounded-lg border-2 border-dashed border-gray-300 px-4 py-8 text-center text-sm text-gray-500 dark:border-gray-700 dark:text-gray-400"
					>
						No feeds added yet
					</p>
				{/if}
			{:else}
				{#each feeds as feed (feed.id)}
					<div class="group flex items-center gap-1">
						<button
							title={feed.title}
							class={`flex flex-1 items-center gap-3 rounded-lg px-3 py-2 text-left transition ${
								selectedFeedId === feed.id
									? 'bg-gray-100 text-gray-900 dark:bg-gray-900 dark:text-white'
									: 'text-gray-600 hover:bg-gray-50 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-gray-900/50 dark:hover:text-white'
							}`}
							type="button"
							onclick={() => onSelectFeed(feed.id)}
						>
							{#if collapsed}
								<span
									class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-indigo-400 to-indigo-600 text-xs font-semibold text-white"
								>
									{feed.title.charAt(0).toUpperCase()}
								</span>
							{:else}
								<span
									class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-indigo-400 to-indigo-600 text-xs font-semibold text-white"
								>
									{feed.title.charAt(0).toUpperCase()}
								</span>
								<div class="min-w-0 flex-1">
									<p class="truncate text-sm font-medium">{feed.title}</p>
									<p class="truncate text-xs text-gray-500 dark:text-gray-400">
										{feed.kind === 'podcast' ? 'Podcast' : 'Feed'}
										{#if refreshingFeedIds.includes(feed.id)}
											• Syncing...
										{:else if feed.lastFetchedAt}
											• Local
										{/if}
									</p>
								</div>
							{/if}
						</button>
						{#if !collapsed}
							<button
								title="Remove feed"
								class="rounded-lg px-2 py-2 text-gray-400 opacity-0 transition group-hover:opacity-100 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-900 dark:hover:text-gray-300"
								type="button"
								onclick={(event) => {
									event.stopPropagation();
									void onRemoveFeed(feed.id);
								}}
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									class="h-4 w-4"
								>
									<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
								</svg>
							</button>
						{/if}
					</div>
				{/each}
			{/if}
		</div>
	</div>
</aside>
