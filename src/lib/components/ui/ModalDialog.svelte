<script lang="ts">
	import { Dialog } from '@skeletonlabs/skeleton-svelte';
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

<Dialog
	{open}
	onOpenChange={(details: { open: boolean }) => {
		if (!details.open) {
			onClose();
		}
	}}
>
	<Dialog.Backdrop class="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm" />
	<Dialog.Positioner class="fixed inset-0 z-50 flex items-start justify-center pt-[15vh]">
		<Dialog.Content
			class="w-full max-w-lg rounded-xl border border-border bg-surface-shell-opaque p-6 shadow-xl"
		>
			<div class="flex items-center justify-between">
				<Dialog.Title class="text-lg font-semibold text-fg">{title}</Dialog.Title>
				<Dialog.CloseTrigger
					type="button"
					class="btn-icon rounded-xl text-fg-muted hover:preset-tonal hover:text-fg"
					aria-label="Close dialog"
				>
					<Icon icon="lucide:x" class="size-4" />
				</Dialog.CloseTrigger>
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
		</Dialog.Content>
	</Dialog.Positioner>
</Dialog>
