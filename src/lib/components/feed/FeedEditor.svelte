<script lang="ts">
	import ModalDialog from '$lib/components/ui/ModalDialog.svelte';

	type Props = {
		open: boolean;
		isLoading: boolean;
		onSave: (url: string) => void;
		onClose: () => void;
	};

	let { open, isLoading, onSave, onClose }: Props = $props();

	let url = $state('');
	let inputRef = $state<HTMLInputElement | null>(null);

	const isValid = $derived(url.trim().length > 0);

	$effect(() => {
		if (open) {
			url = '';
			queueMicrotask(() => {
				inputRef?.focus();
			});
		}
	});

	function handleSubmit(): void {
		const trimmedUrl = url.trim();

		if (!trimmedUrl) {
			return;
		}

		onSave(trimmedUrl);
	}
</script>

<ModalDialog
	{open}
	title="Add feed"
	{onClose}
	onSubmit={(event: SubmitEvent) => {
		event.preventDefault();
		handleSubmit();
	}}
>
	<div class={isLoading ? 'opacity-50' : ''}>
		<div>
			<label for="feed-url" class="block text-sm font-medium text-fg-secondary">
				RSS URL, Apple Podcasts URL, or Apple ID
			</label>

			<input
				id="feed-url"
				type="text"
				bind:this={inputRef}
				bind:value={url}
				disabled={isLoading}
				placeholder="https://example.com/feed.xml"
				class="mt-1.5 w-full rounded-xl border border-border bg-surface px-4 py-2.5 text-sm text-fg transition outline-none placeholder:text-fg-muted focus:border-border-hover focus:ring-2 focus:ring-ring"
			/>
		</div>
	</div>

	{#snippet actions()}
		<button
			type="button"
			class="btn rounded-xl preset-tonal"
			disabled={isLoading}
			onclick={onClose}
		>
			Cancel
		</button>

		<button type="submit" class="btn rounded-xl preset-filled" disabled={!isValid || isLoading}>
			{isLoading ? 'Adding…' : 'Add feed'}
		</button>
	{/snippet}
</ModalDialog>
