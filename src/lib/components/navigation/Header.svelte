<script lang="ts">
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import GlobalSearch from '../search/GlobalSearch.svelte';
	import { openFeedEditor } from '$lib/hooks/useAppUi.svelte';

	function handleOpenDialog() {
		openFeedEditor();
	}
</script>

<div class="pointer-events-none flex w-full items-center gap-4 px-2">
	<GlobalSearch />
	<IconButton icon="lucide:plus" title="Add feed" variant="accent" onclick={handleOpenDialog} />
</div>
