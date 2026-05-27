<script lang="ts">
	import { appUi } from '$lib/hooks/useAppUi.svelte';
	import ReaderPane from '$lib/components/content/ReaderPane.svelte';
	import LibraryListPane from './LibraryListPane.svelte';
</script>

<div class="flex min-h-0 flex-1 overflow-hidden">
	<LibraryListPane />
	<ReaderPane
		class="min-h-0 min-w-0 {appUi.isReaderMaximized
			? 'grow'
			: 'xl:basis-1/2 2xl:flex 2xl:flex-1 4xl:basis-4/10'}"
	/>
</div>
