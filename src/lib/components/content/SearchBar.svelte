<script lang="ts">
	import SearchInput from '$lib/components/ui/SearchInput.svelte';
	import {
		selection,
		setFeedSearchTerm,
		setSectionSearchTerm,
		setStationSearchTerm
	} from '$lib/state/selection.svelte';
	import { getSelectedFeed, getSelectedStation } from '$lib/state/selectors.svelte';

	type SearchContext = {
		id: string;
		label: string;
		placeholder: string;
		term: string;
		onChange: (term: string) => void;
	};

	type Props = {
		inputRef?: HTMLInputElement | null;
	};

	let { inputRef = $bindable(null) }: Props = $props();

	let localSearchValue = $state('');
	const selectedFeed = $derived(getSelectedFeed());
	const selectedStation = $derived(getSelectedStation());

	const searchContext = $derived.by((): SearchContext | null => {
		if (selectedFeed) {
			return {
				id: 'feed-search',
				label: 'Search this feed',
				placeholder: 'Search this feed',
				term: selection.feedSearchTerm,
				onChange: setFeedSearchTerm
			};
		}

		if (selectedStation) {
			return {
				id: 'station-search',
				label: 'Search this station',
				placeholder: 'Search this station',
				term: selection.stationSearchTerm,
				onChange: setStationSearchTerm
			};
		}

		if (selection.selectedSection === 'unread' || selection.selectedSection === 'media') {
			return {
				id: 'section-search',
				label: `Search ${selection.selectedSection === 'unread' ? 'unread' : 'media'}`,
				placeholder: `Search ${selection.selectedSection === 'unread' ? 'unread' : 'media'}`,
				term: selection.sectionSearchTerm,
				onChange: setSectionSearchTerm
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
