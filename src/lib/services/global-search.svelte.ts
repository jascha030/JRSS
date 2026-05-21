import { queryItems } from '$lib/services/item';
import type { Feed } from '$lib/types/feed';
import type { FeedListItem } from '$lib/types/item';
import type { Station } from '$lib/types/station';

const SEARCH_DEBOUNCE_MS = 220;
const MAX_FEED_RESULTS = 3;
const MAX_STATION_RESULTS = 3;
const MAX_ITEM_RESULTS = 8;

type Options = {
	getTerm: () => string;
	getFeeds: () => Feed[];
	getStations: () => Station[];
};

export type SearchActionResult = {
	title: string;
	description: string;
	url: string;
};

export type SearchResultEntry =
	| {
			id: string;
			kind: 'action';
			data: SearchActionResult;
	  }
	| {
			id: string;
			kind: 'feed';
			data: Feed;
	  }
	| {
			id: string;
			kind: 'station';
			data: Station;
	  }
	| {
			id: string;
			kind: 'item';
			data: FeedListItem;
	  };

function canParseUrl(input: string) {
	try {
		new URL(input);
		return true;
	} catch {
		return false;
	}
}

function getSearchTerm(options: Options) {
	return options.getTerm().trim();
}

export function createGlobalSearch(options: Options) {
	let feedResults = $state<Feed[]>([]);
	let stationResults = $state<Station[]>([]);
	let itemResults = $state<FeedListItem[]>([]);
	let isLoading = $state(false);
	let isOpen = $state(false);

	const actionResults = $derived.by<SearchActionResult[]>(() => {
		const term = getSearchTerm(options);

		if (!term || !canParseUrl(term)) {
			return [];
		}

		return [
			{
				title: 'Add feed',
				description: `Add ${term} as a new feed`,
				url: term
			}
		];
	});

	const feedTitleById = $derived.by(() => {
		return new Map(options.getFeeds().map((feed) => [feed.id, feed.title]));
	});

	const entries = $derived.by<SearchResultEntry[]>(() => [
		...actionResults.map((action) => ({
			id: `action:${action.url}`,
			kind: 'action' as const,
			data: action
		})),
		...feedResults.map((feed) => ({
			id: `feed:${feed.id}`,
			kind: 'feed' as const,
			data: feed
		})),
		...stationResults.map((station) => ({
			id: `station:${station.id}`,
			kind: 'station' as const,
			data: station
		})),
		...itemResults.map((item) => ({
			id: `item:${item.id}`,
			kind: 'item' as const,
			data: item
		}))
	]);

	function clearResults() {
		feedResults = [];
		stationResults = [];
		itemResults = [];
	}

	function clear() {
		clearResults();
		isLoading = false;
		isOpen = false;
	}

	function close() {
		isOpen = false;
	}

	function open() {
		if (!getSearchTerm(options)) return;
		isOpen = true;
	}

	$effect(() => {
		const term = getSearchTerm(options);
		const feeds = options.getFeeds();
		const stations = options.getStations();

		if (!term) {
			clear();
			return;
		}

		const lowerTerm = term.toLowerCase();

		const nextFeedResults = feeds
			.filter((feed) => {
				return (
					feed.title.toLowerCase().includes(lowerTerm) || feed.url.toLowerCase().includes(lowerTerm)
				);
			})
			.slice(0, MAX_FEED_RESULTS);

		const nextStationResults = stations
			.filter((station) => station.name.toLowerCase().includes(lowerTerm))
			.slice(0, MAX_STATION_RESULTS);

		feedResults = nextFeedResults;
		stationResults = nextStationResults;
		itemResults = [];
		isLoading = true;
		isOpen = true;

		const hasImmediateResults =
			nextFeedResults.length > 0 || nextStationResults.length > 0 || canParseUrl(term);

		let cancelled = false;

		const timer = setTimeout(() => {
			void queryItems({
				section: 'all',
				offset: 0,
				limit: MAX_ITEM_RESULTS,
				search: term,
				sortOrder: 'newest_first'
			})
				.then((page) => {
					if (cancelled) return;

					itemResults = page.items;

					if (page.items.length > 0 || hasImmediateResults) {
						isOpen = true;
					}
				})
				.catch(() => {
					if (cancelled) return;
					itemResults = [];
				})
				.finally(() => {
					if (cancelled) return;
					isLoading = false;
				});
		}, SEARCH_DEBOUNCE_MS);

		return () => {
			cancelled = true;
			clearTimeout(timer);
		};
	});

	return {
		get feedResults() {
			return feedResults;
		},
		get stationResults() {
			return stationResults;
		},
		get itemResults() {
			return itemResults;
		},
		get actionResults() {
			return actionResults;
		},
		get entries() {
			return entries;
		},
		get feedTitleById() {
			return feedTitleById;
		},
		get isLoading() {
			return isLoading;
		},
		get isOpen() {
			return isOpen;
		},
		clear,
		close,
		open
	};
}
