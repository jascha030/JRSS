<script lang="ts">
	import { page } from '$app/state';
	import { selection } from '$lib/state';

	const feedId = $derived(page.params.feedId ?? null);

	$effect(() => {
		selection.selectedFeedId = feedId;
		selection.selectedStationId = null;
		selection.selectedSection = null;
	});
</script>
