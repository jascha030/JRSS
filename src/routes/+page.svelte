<script lang="ts">
	import { page } from '$app/state';
	import MainWindowPage from '$lib/components/windows/MainWindowPage.svelte';
	import MiniPlayerWindow from '$lib/components/windows/MiniPlayerWindow.svelte';
	import {
		playbackState,
		getCurrentAudioItem,
		getCurrentAudioItemFeed
	} from '$lib/stores/app.svelte';

	const isMiniWindow = $derived(page.url.searchParams.get('window') === 'mini');

	const currentPlaybackState = $derived(playbackState.currentPlaybackState);
	const currentAudioItem = $derived(getCurrentAudioItem());
	const currentAudioItemFeed = $derived(getCurrentAudioItemFeed());
</script>

<svelte:head>
	<title>JRSS</title>
	<meta name="description" content="RSS reader and podcast player." />
</svelte:head>

{#if isMiniWindow}
	<MiniPlayerWindow
		item={currentAudioItem}
		imageUrl={currentAudioItemFeed?.imageUrl}
		feedTitle={currentAudioItemFeed?.title}
		playbackState={currentPlaybackState}
	/>
{:else}
	<MainWindowPage />
{/if}
