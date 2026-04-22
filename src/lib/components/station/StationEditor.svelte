<script lang="ts">
	import type {
		CreateStationInput,
		Feed,
		Station,
		StationEpisodeFilter,
		ItemSortOrder
	} from '$lib/types/rss';
	import { SvelteSet } from 'svelte/reactivity';
	import Icon from '@iconify/svelte';

	type Props = {
		open: boolean;
		/** Pass an existing station to edit, or null to create a new one. */
		station: Station | null;
		/** Only media feeds are relevant for stations. */
		feeds: Feed[];
		onSave: (input: CreateStationInput) => void;
		onClose: () => void;
	};

	let { open, station, feeds, onSave, onClose }: Props = $props();

	const mediaFeeds = $derived(feeds.filter((feed) => feed.kind === 'media'));

	let name = $state('');
	let episodeFilter = $state<StationEpisodeFilter>('all');
	let sortOrder = $state<ItemSortOrder>('newest_first');
	let selectedFeedIds = $state(new SvelteSet<string>());

	// Reset form when dialog opens or station changes
	$effect(() => {
		if (open) {
			name = station?.name ?? '';
			episodeFilter = station?.episodeFilter ?? 'all';
			sortOrder = station?.sortOrder ?? 'newest_first';
			selectedFeedIds = new SvelteSet(station?.feedIds ?? []);
		}
	});

	function toggleFeed(feedId: string): void {
		if (selectedFeedIds.has(feedId)) {
			selectedFeedIds.delete(feedId);
		} else {
			selectedFeedIds.add(feedId);
		}
	}

	function handleSubmit(): void {
		const trimmedName = name.trim();

		if (!trimmedName || selectedFeedIds.size === 0) {
			return;
		}

		onSave({
			name: trimmedName,
			episodeFilter,
			sortOrder,
			feedIds: [...selectedFeedIds]
		});
	}

	const isValid = $derived(name.trim().length > 0 && selectedFeedIds.size > 0);
</script>

{#if open}
	<!-- Backdrop -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
		onkeydown={(event) => {
			if (event.key === 'Escape') {
				onClose();
			}
		}}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="absolute inset-0" onclick={onClose}></div>

		<div
			class="relative z-10 w-full max-w-lg rounded-3xl border border-border bg-surface p-6 shadow-xl"
		>
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-fg">
					{station ? 'Edit station' : 'New station'}
				</h2>
				<button
					type="button"
					class="flex size-8 items-center justify-center rounded-lg text-fg-muted transition-colors hover:bg-surface-hover hover:text-fg"
					onclick={onClose}
				>
					<Icon icon="lucide:x" class="size-4" />
				</button>
			</div>

			<form
				class="mt-6 space-y-5"
				onsubmit={(event) => {
					event.preventDefault();
					handleSubmit();
				}}
			>
				<!-- Name -->
				<div>
					<label for="station-name" class="block text-sm font-medium text-fg-secondary">
						Name
					</label>
					<input
						id="station-name"
						type="text"
						bind:value={name}
						placeholder="My station"
						class="mt-1.5 w-full rounded-xl border border-border bg-surface px-4 py-2.5 text-sm text-fg transition outline-none focus:border-border-hover focus:ring-2 focus:ring-ring"
					/>
				</div>

				<!-- Episode filter -->
				<div>
					<label for="station-filter" class="block text-sm font-medium text-fg-secondary">
						Episodes
					</label>
					<select
						id="station-filter"
						bind:value={episodeFilter}
						class="mt-1.5 w-full rounded-xl border border-border bg-surface px-3 py-2.5 text-sm text-fg transition outline-none focus:border-border-hover focus:ring-2 focus:ring-ring"
					>
						<option value="all">All episodes</option>
						<option value="unplayed">Unplayed only</option>
					</select>
				</div>

				<!-- Sort order -->
				<div>
					<label for="station-sort" class="block text-sm font-medium text-fg-secondary">
						Sort order
					</label>
					<select
						id="station-sort"
						bind:value={sortOrder}
						class="mt-1.5 w-full rounded-xl border border-border bg-surface px-3 py-2.5 text-sm text-fg transition outline-none focus:border-border-hover focus:ring-2 focus:ring-ring"
					>
						<option value="newest_first">Newest first</option>
						<option value="oldest_first">Oldest first</option>
					</select>
				</div>

				<!-- Feed picker -->
				<div>
					<p class="text-sm font-medium text-fg-secondary">Podcasts</p>

					{#if mediaFeeds.length === 0}
						<p class="mt-2 text-sm text-fg-muted">
							No podcast feeds added yet. Add a podcast feed first.
						</p>
					{:else}
						<div
							class="mt-2 max-h-48 space-y-1 overflow-y-auto rounded-xl border border-border p-2"
						>
							{#each mediaFeeds as feed (feed.id)}
								<label
									class="flex cursor-pointer items-center gap-3 rounded-xl px-3 py-2 transition-colors hover:bg-surface-hover"
								>
									<input
										type="checkbox"
										checked={selectedFeedIds.has(feed.id)}
										onchange={() => toggleFeed(feed.id)}
										class="size-4 rounded border-border text-accent focus:ring-ring"
									/>
									<span class="min-w-0 flex-1">
										<span class="block truncate text-sm text-fg">{feed.title}</span>
									</span>
								</label>
							{/each}
						</div>
					{/if}
				</div>

				<div class="flex items-center justify-end gap-3 pt-2">
					<button type="button" class="btn-secondary btn" onclick={onClose}>Cancel</button>
					<button type="submit" class="btn-primary btn" disabled={!isValid}>
						{station ? 'Save changes' : 'Create station'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
