<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	type Props = {
		isLoading: boolean;
		onSubmit: (url: string) => void;
	};

	let { isLoading, onSubmit }: Props = $props();

	let inputValue = $state('');
	let inputRef = $state<HTMLInputElement | null>(null);

	function handleSubmit(event: Event) {
		event.preventDefault();

		const url = inputValue.trim();
		if (!url) return;

		onSubmit(url);
		inputValue = '';
	}

	onMount(() => {
		let unlisten: UnlistenFn | undefined;

		const setupListener = async () => {
			unlisten = await listen('menu-add-feed', () => {
				inputRef?.focus();
			});
		};

		void setupListener();

		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});
</script>

<form class="w-full min-w-full" onsubmit={handleSubmit}>
	<label class="sr-only" for="feedUrl">RSS URL, Apple Podcasts URL, or Apple ID</label>

	<div class="input-group grid-cols-[1fr_auto]">
		<input
			id="feedUrl"
			name="feedUrl"
			type="text"
			bind:this={inputRef}
			bind:value={inputValue}
			disabled={isLoading}
			placeholder="RSS URL, Apple Podcasts URL, or Apple ID"
			class="ig-input"
		/>

		<button
			class="preset-filled-accent ig-btn whitespace-nowrap"
			disabled={isLoading}
			type="submit"
		>
			{isLoading ? 'Adding…' : 'Add feed'}
		</button>
	</div>
</form>
