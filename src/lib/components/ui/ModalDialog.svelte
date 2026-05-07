<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';

	type Props = {
		open: boolean;
		title: string;
		onClose: () => void;
		children: Snippet;
		actions?: Snippet;
		onSubmit?: (event: SubmitEvent) => void;
	};

	let { open, title, onClose, children, actions, onSubmit }: Props = $props();
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
		tabindex="-1"
		onkeydown={(event) => {
			if (event.key === 'Escape') {
				onClose();
			}
		}}
	>
		<button type="button" class="absolute inset-0" aria-label="Close dialog" onclick={onClose}
		></button>

		<div
			class="relative z-10 w-full max-w-lg rounded-2xl border border-border bg-surface p-6 shadow-xl"
		>
			<div class="flex items-center justify-between">
				<h2 class="text-lg font-semibold text-fg">{title}</h2>

				<button
					type="button"
					class="btn-icon rounded-xl text-fg-muted hover:preset-tonal hover:text-fg"
					aria-label="Close dialog"
					onclick={onClose}
				>
					<Icon icon="lucide:x" class="size-4" />
				</button>
			</div>

			{#if onSubmit}
				<form class="mt-6 space-y-5" onsubmit={onSubmit}>
					{@render children()}

					{#if actions}
						<div class="flex items-center justify-end gap-3 pt-2">
							{@render actions()}
						</div>
					{/if}
				</form>
			{:else}
				<div class="mt-6 space-y-5">
					{@render children()}

					{#if actions}
						<div class="flex items-center justify-end gap-3 pt-2">
							{@render actions()}
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>
{/if}
