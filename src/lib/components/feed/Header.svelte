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

<form
	class="flex w-full min-w-full flex-col gap-3 sm:flex-row sm:items-center"
	onsubmit={handleSubmit}
>
	<label class="sr-only" for="feedUrl"> RSS URL, Apple Podcasts URL, or Apple ID </label>

	<input
		id="feedUrl"
		name="feedUrl"
		type="text"
		bind:value={inputValue}
		disabled={isLoading}
		placeholder="RSS URL, Apple Podcasts URL, or Apple ID"
		class="min-w-0 flex-1 rounded-xl border border-border bg-surface px-4 py-3 text-sm text-fg transition outline-none focus:border-border-hover focus:ring-2 focus:ring-ring"
	/>

	<button class="btn preset-filled whitespace-nowrap" disabled={isLoading} type="submit">
		{isLoading ? 'Adding…' : 'Add feed'}
	</button>
</form>
