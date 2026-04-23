<script lang="ts">
	import Icon from '@iconify/svelte';

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

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
		onkeydown={(event) => {
			if (event.key === 'Escape') {
				onClose();
			}
		}}
	>
		<button type="button" class="absolute inset-0" aria-label="Close feed editor" onclick={onClose}
		></button>

		<div
			class="relative z-10 w-full max-w-lg rounded-2xl border border-border bg-surface p-6 shadow-xl"
		>
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-fg">Add feed</h2>

				<button
					type="button"
					class="btn-icon rounded-xl text-fg-muted hover:preset-tonal hover:text-fg"
					aria-label="Close feed editor"
					onclick={onClose}
				>
					<Icon icon="lucide:x" class="size-4" />
				</button>
			</div>

			<form
				class="mt-6 space-y-5"
				class:opacity-50={isLoading}
				onsubmit={(event: SubmitEvent) => {
					event.preventDefault();
					handleSubmit();
				}}
			>
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

				<div class="flex items-center justify-end gap-3 pt-2">
					<button
						type="button"
						class="btn rounded-xl preset-tonal"
						disabled={isLoading}
						onclick={onClose}
					>
						Cancel
					</button>

					<button
						type="submit"
						class="btn rounded-xl preset-filled"
						disabled={!isValid || isLoading}
					>
						{isLoading ? 'Adding…' : 'Add feed'}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
