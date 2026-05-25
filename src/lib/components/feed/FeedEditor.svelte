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
		<label class="label">
			<span class="label-text">RSS URL, Apple Podcasts URL, or Apple ID</span>
			<input
				type="text"
				bind:this={inputRef}
				bind:value={url}
				disabled={isLoading}
				placeholder="https://example.com/feed.xml"
				class="input"
			/>
		</label>
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
