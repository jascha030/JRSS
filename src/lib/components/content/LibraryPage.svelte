<script lang="ts">
	import { appUi } from '$lib/hooks/useAppUi.svelte';
	import ItemListView from '$lib/components/content/ItemListView.svelte';
	import ReaderPane from '$lib/components/content/ReaderPane.svelte';

	let {
		onReadyStateChange,
		class: className = ''
	}: {
		onReadyStateChange?: (ready: boolean) => void;
		class?: string;
	} = $props();
</script>

<div class="flex min-h-0 flex-1 overflow-hidden {className}">
	<ItemListView
		{onReadyStateChange}
		class="min-h-0 min-w-0 grow border-r border-border xl:basis-1/2 2xl:shrink-0 2xl:grow-0 4xl:basis-4/10 {appUi.isReaderMaximized
			? 'hidden'
			: ''}"
	/>
	<ReaderPane
		class="min-h-0 min-w-0 {appUi.isReaderMaximized
			? 'grow'
			: 'xl:basis-1/2 2xl:flex 2xl:flex-1 4xl:basis-4/10'}"
	/>
</div>
