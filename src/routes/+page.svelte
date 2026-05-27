<script lang="ts">
	import { page } from '$app/state';
	import App from '$lib/components/App.svelte';
	import MiniPlayer from '$lib/components/MiniPlayer.svelte';
	import { playbackState, getCurrentAudioItem } from '$lib/state';

	const isMiniWindow = $derived(page.url.searchParams.get('window') === 'mini');

	const currentPlaybackState = $derived(playbackState.currentPlaybackState);
	const currentAudioItem = $derived(getCurrentAudioItem());
</script>

<svelte:head>
	<title>JRSS</title>
	<meta name="description" content="RSS reader and podcast player." />
</svelte:head>

{#if isMiniWindow}
	<MiniPlayer
		item={currentAudioItem}
		imageUrl={currentAudioItem?.imageUrl}
		playbackState={currentPlaybackState}
	/>
{:else}
	<App />
{/if}
