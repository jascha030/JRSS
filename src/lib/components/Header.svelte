<script lang="ts">
	type Props = {
		isLoading: boolean;
		onSubmit: (url: string) => void;
	};

	let { isLoading, onSubmit }: Props = $props();

	let inputValue = $state('');

	function handleSubmit(event: Event) {
		event.preventDefault();
		const url = inputValue.trim();
		if (!url) return;
		onSubmit(url);
		inputValue = '';
	}
</script>

<form class="flex min-w-full flex-col gap-4 sm:flex-row" onsubmit={handleSubmit}>
	<input
		id="feedUrl"
		name="feedUrl"
		class="min-w-1 flex-1 rounded-2xl border border-border-strong bg-surface px-4 py-3 text-sm text-fg transition outline-none focus:border-border-hover focus:ring-2 focus:ring-ring"
		disabled={isLoading}
		placeholder="RSS URL, Apple Podcasts URL, or Apple ID"
		type="text"
		bind:value={inputValue}
	/>
	<button class="btn-primary btn" disabled={isLoading} type="submit">
		{isLoading ? 'Adding...' : 'Add feed'}
	</button>
</form>
