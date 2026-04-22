<script lang="ts">
	import type {
		CreateStationInput,
		Feed,
		Station,
		StationEpisodeFilter,
		ItemSortOrder
	} from '$lib/types/rss';
	import Icon from '@iconify/svelte';
	import {
		Combobox,
		Portal,
		SegmentedControl,
		type ComboboxRootProps,
		useListCollection
	} from '@skeletonlabs/skeleton-svelte';

	type Props = {
		open: boolean;
		station: Station | null;
		feeds: Feed[];
		onSave: (input: CreateStationInput) => void;
		onClose: () => void;
	};

	type FeedOption = {
		label: string;
		value: string;
	};

	let { open, station, feeds, onSave, onClose }: Props = $props();

	const mediaFeeds = $derived(feeds.filter((feed) => feed.kind === 'media'));

	const allPodcastOptions = $derived.by<FeedOption[]>(() =>
		mediaFeeds.map((feed) => ({
			label: feed.title,
			value: feed.id
		}))
	);

	let name = $state('');
	let episodeFilter = $state<StationEpisodeFilter>('all');
	let sortOrder = $state<ItemSortOrder>('newest_first');
	let selectedFeedValues = $state<string[]>([]);
	let filteredPodcastOptions = $state<FeedOption[]>([]);

	const collection = $derived(
		useListCollection({
			items: filteredPodcastOptions,
			itemToString: (item) => item.label,
			itemToValue: (item) => item.value
		})
	);

	const selectedFeedTitleById = $derived.by(() => {
		const map: Record<string, string> = {};
		for (const option of allPodcastOptions) {
			map[option.value] = option.label;
		}
		return map;
	});

	const selectedFeedOptions = $derived.by(() =>
		selectedFeedValues
			.map((id) => {
				const label = selectedFeedTitleById[id];
				return label ? { value: id, label } : null;
			})
			.filter((item): item is FeedOption => item !== null)
	);

	const isValid = $derived(name.trim().length > 0 && selectedFeedValues.length > 0);

	$effect(() => {
		filteredPodcastOptions = allPodcastOptions;
	});

	$effect(() => {
		if (open) {
			name = station?.name ?? '';
			episodeFilter = station?.episodeFilter ?? 'all';
			sortOrder = station?.sortOrder ?? 'newest_first';
			selectedFeedValues = station?.feedIds ?? [];
			filteredPodcastOptions = allPodcastOptions;
		}
	});

	const onComboboxOpenChange = () => {
		filteredPodcastOptions = allPodcastOptions;
	};

	const onComboboxInputValueChange: ComboboxRootProps['onInputValueChange'] = (event) => {
		const query = event.inputValue.toLowerCase().trim();

		if (!query) {
			filteredPodcastOptions = allPodcastOptions;
			return;
		}

		filteredPodcastOptions = allPodcastOptions.filter((item) =>
			item.label.toLowerCase().includes(query)
		);
	};

	const onComboboxValueChange: ComboboxRootProps['onValueChange'] = (event) => {
		selectedFeedValues = [...event.value];
	};

	function removeSelectedFeed(feedId: string) {
		selectedFeedValues = selectedFeedValues.filter((id) => id !== feedId);
	}

	function handleSubmit(): void {
		const trimmedName = name.trim();

		if (!trimmedName || selectedFeedValues.length === 0) {
			return;
		}

		onSave({
			name: trimmedName,
			episodeFilter,
			sortOrder,
			feedIds: selectedFeedValues
		});
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
		onkeydown={(event) => {
			if (event.key === 'Escape') {
				onClose();
			}
		}}
	>
		<button
			type="button"
			class="absolute inset-0"
			aria-label="Close station editor"
			onclick={onClose}
		></button>

		<div
			class="relative z-10 w-full max-w-lg rounded-2xl border border-border bg-surface p-6 shadow-xl"
		>
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-fg">
					{station ? 'Edit station' : 'New station'}
				</h2>

				<button
					type="button"
					class="btn-icon rounded-xl text-fg-muted hover:preset-tonal hover:text-fg"
					aria-label="Close station editor"
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
				<div>
					<label for="station-name" class="block text-sm font-medium text-fg-secondary">
						Name
					</label>

					<input
						id="station-name"
						type="text"
						bind:value={name}
						placeholder="My station"
						class="mt-1.5 w-full rounded-xl border border-border bg-surface px-4 py-2.5 text-sm text-fg transition outline-none placeholder:text-fg-muted focus:border-border-hover focus:ring-2 focus:ring-ring"
					/>
				</div>

				<div>
					<p class="mb-2 text-sm font-medium text-fg-secondary">Episodes</p>

					<SegmentedControl
						value={episodeFilter}
						onValueChange={(details) => {
							const value = details.value;
							if (value === 'all' || value === 'unplayed') {
								episodeFilter = value;
							}
						}}
					>
						<SegmentedControl.Label class="sr-only">Episode filter</SegmentedControl.Label>

						<SegmentedControl.Control class="rounded-xl">
							<SegmentedControl.Indicator />

							<SegmentedControl.Item value="all">
								<SegmentedControl.ItemText>All episodes</SegmentedControl.ItemText>
								<SegmentedControl.ItemHiddenInput />
							</SegmentedControl.Item>

							<SegmentedControl.Item value="unplayed">
								<SegmentedControl.ItemText>Unplayed only</SegmentedControl.ItemText>
								<SegmentedControl.ItemHiddenInput />
							</SegmentedControl.Item>
						</SegmentedControl.Control>
					</SegmentedControl>
				</div>

				<div>
					<p class="mb-2 text-sm font-medium text-fg-secondary">Sort order</p>

					<SegmentedControl
						value={sortOrder}
						onValueChange={(details) => {
							const value = details.value;
							if (value === 'newest_first' || value === 'oldest_first') {
								sortOrder = value;
							}
						}}
					>
						<SegmentedControl.Label class="sr-only">Station sort order</SegmentedControl.Label>

						<SegmentedControl.Control class="rounded-xl">
							<SegmentedControl.Indicator />

							<SegmentedControl.Item value="newest_first">
								<SegmentedControl.ItemText>Newest first</SegmentedControl.ItemText>
								<SegmentedControl.ItemHiddenInput />
							</SegmentedControl.Item>

							<SegmentedControl.Item value="oldest_first">
								<SegmentedControl.ItemText>Oldest first</SegmentedControl.ItemText>
								<SegmentedControl.ItemHiddenInput />
							</SegmentedControl.Item>
						</SegmentedControl.Control>
					</SegmentedControl>
				</div>

				<div>
					<div class="mb-2 flex items-center justify-between gap-3">
						<p class="text-sm font-medium text-fg-secondary">Podcasts</p>
						<p class="text-xs text-fg-muted">{selectedFeedValues.length} selected</p>
					</div>

					{#if mediaFeeds.length === 0}
						<p class="text-sm text-fg-muted">
							No podcast feeds added yet. Add a podcast feed first.
						</p>
					{:else}
						<div class="space-y-3">
							<Combobox
								class="w-full"
								placeholder="Search podcasts..."
								{collection}
								{onComboboxOpenChange}
								onInputValueChange={onComboboxInputValueChange}
								value={selectedFeedValues}
								onValueChange={onComboboxValueChange}
								multiple
							>
								<Combobox.Label class="sr-only">Search podcasts</Combobox.Label>

								<Combobox.Control>
									<Combobox.Input />
									<Combobox.Trigger />
								</Combobox.Control>

								<Portal>
									<Combobox.Positioner>
										<Combobox.Content class="z-50 max-h-64 overflow-y-auto">
											{#if filteredPodcastOptions.length === 0}
												<div class="px-3 py-2 text-sm text-fg-muted">No podcasts found</div>
											{:else}
												{#each filteredPodcastOptions as item (item.value)}
													<Combobox.Item {item}>
														<Combobox.ItemText>{item.label}</Combobox.ItemText>
														<Combobox.ItemIndicator />
													</Combobox.Item>
												{/each}
											{/if}
										</Combobox.Content>
									</Combobox.Positioner>
								</Portal>
							</Combobox>

							{#if selectedFeedOptions.length > 0}
								<div class="flex flex-wrap gap-2">
									{#each selectedFeedOptions as feed (feed.value)}
										<span
											class="inline-flex items-center gap-2 rounded-full border border-border px-3 py-1.5 text-xs text-fg"
										>
											<span class="max-w-56 truncate">{feed.label}</span>

											<button
												type="button"
												class="grid size-4 place-items-center rounded-full text-fg-muted transition-colors hover:text-fg"
												aria-label={`Remove ${feed.label}`}
												onclick={() => removeSelectedFeed(feed.value)}
											>
												<Icon icon="lucide:x" class="size-3" />
											</button>
										</span>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				</div>

				<div class="flex items-center justify-end gap-3 pt-2">
					<button type="button" class="btn rounded-xl preset-tonal" onclick={onClose}>
						Cancel
					</button>

					<button type="submit" class="btn rounded-xl preset-filled" disabled={!isValid}>
						{station ? 'Save changes' : 'Create station'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
