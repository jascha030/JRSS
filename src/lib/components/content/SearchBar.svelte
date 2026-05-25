<script lang="ts">
	import SearchInput from '$lib/components/ui/SearchInput.svelte';

	import type { SidebarSection } from '$lib/state';
	import type { Feed } from '$lib/types/feed';
	import type { Station } from '$lib/types/station';

	type SearchContext = {
		id: string;
		label: string;
		placeholder: string;
		term: string;
		onChange: (term: string) => void;
	};

	type Props = {
		selectedFeed: Feed | null;
		selectedStation: Station | null;
		selectedSection: SidebarSection;
		searchTerm: string;
		onSearchChange: (term: string) => void;
		stationSearchTerm: string;
		onStationSearchChange: (term: string) => void;
		sectionSearchTerm: string;
		onSectionSearchChange: (term: string) => void;
		inputRef?: HTMLInputElement | null;
	};

	let {
		selectedFeed,
		selectedStation,
		selectedSection,
		searchTerm,
		onSearchChange,
		stationSearchTerm,
		onStationSearchChange,
		sectionSearchTerm,
		onSectionSearchChange,
		inputRef = $bindable(null)
	}: Props = $props();

	let localSearchValue = $state('');

	const searchContext = $derived.by((): SearchContext | null => {
		if (selectedFeed) {
			return {
				id: 'feed-search',
				label: 'Search this feed',
				placeholder: 'Search this feed',
				term: searchTerm,
				onChange: onSearchChange
			};
		}

		if (selectedStation) {
			return {
				id: 'station-search',
				label: 'Search this station',
				placeholder: 'Search this station',
				term: stationSearchTerm,
				onChange: onStationSearchChange
			};
		}

		if (selectedSection === 'unread' || selectedSection === 'media') {
			return {
				id: 'section-search',
				label: `Search ${selectedSection === 'unread' ? 'unread' : 'media'}`,
				placeholder: `Search ${selectedSection === 'unread' ? 'unread' : 'media'}`,
				term: sectionSearchTerm,
				onChange: onSectionSearchChange
			};
		}

		return null;
	});

	$effect(() => {
		if (searchContext) {
			localSearchValue = searchContext.term;
		}
	});

	function handleInput(event: Event & { currentTarget: HTMLInputElement }) {
		const value = event.currentTarget.value;
		localSearchValue = value;
		searchContext?.onChange(value);
	}

	function handleEscape() {
		localSearchValue = '';
		inputRef?.blur();
		searchContext?.onChange('');
	}
</script>

{#if searchContext}
	<div class="flex w-full flex-row flex-wrap items-center justify-between gap-4">
		<div class="flex-1">
			<SearchInput
				id={searchContext.id}
				label={searchContext.label}
				placeholder={searchContext.placeholder}
				value={localSearchValue}
				bind:inputRef
				kbdShortcuts={['⌘', 'F']}
				oninput={handleInput}
				onkeydown={(event) => {
					if (event.key === 'Escape') {
						handleEscape();
					}
				}}
			/>
		</div>
	</div>
{/if}
