<script lang="ts">
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import { openCreateMenu } from '$lib/utils/tauri-menu';
	import GlobalSearch from '../search/GlobalSearch.svelte';

	function handleOpenDialog(event: MouseEvent) {
		void openCreateMenu(event);
	}
</script>

<div class="pointer-events-none flex w-full items-center gap-4 px-2">
	<GlobalSearch />
	<IconButton icon="lucide:plus" title="Create" variant="accent" onclick={handleOpenDialog} />
</div>
