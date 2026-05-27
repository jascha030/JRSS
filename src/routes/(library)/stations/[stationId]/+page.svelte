<script lang="ts">
	import { page } from '$app/state';
	import { selection } from '$lib/state';

	const stationId = $derived(page.params.stationId ?? null);

	$effect(() => {
		selection.selectedFeedId = null;
		selection.selectedStationId = stationId;
		selection.selectedSection = null;
	});
</script>
