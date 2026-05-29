<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { Snippet } from 'svelte';
	import { cubicOut } from 'svelte/easing';

	type Props = {
		open: boolean;
		title: string;
		onClose: () => void;
		children: Snippet;
		actions?: Snippet;
		onSubmit?: (event: SubmitEvent) => void;
	};

	let { open, title, onClose, children, actions, onSubmit }: Props = $props();
	let dialog = $state<HTMLDialogElement | null>(null);
	let panelState = $state<'closed' | 'opening' | 'open' | 'closing'>('closed');
	let closeTimer = 0;

	const transitionDurationMs = 200;
	const transitionEasing = cubicOut;

	function clearCloseTimer(): void {
		if (closeTimer) {
			window.clearTimeout(closeTimer);
			closeTimer = 0;
		}
	}

	function animateOpen(): void {
		if (!dialog) {
			return;
		}

		clearCloseTimer();

		if (!dialog.open) {
			dialog.showModal();
		}

		panelState = 'opening';

		requestAnimationFrame(() => {
			panelState = 'open';
		});
	}

	function animateClose(): void {
		if (!dialog?.open) {
			panelState = 'closed';
			return;
		}

		clearCloseTimer();
		panelState = 'closing';
		closeTimer = window.setTimeout(() => {
			dialog?.close();
			panelState = 'closed';
			closeTimer = 0;
		}, transitionDurationMs);
	}

	$effect(() => {
		if (!dialog) {
			return;
		}

		if (open) {
			animateOpen();
			return;
		}

		animateClose();

		return () => {
			clearCloseTimer();
		};
	});

	function handleCancel(event: Event): void {
		event.preventDefault();
		onClose();
	}

	function handleClose(): void {
		clearCloseTimer();
		panelState = 'closed';
	}

	function handleBackdropClick(event: MouseEvent): void {
		if (event.currentTarget === event.target) {
			onClose();
		}
	}
</script>

<dialog
	bind:this={dialog}
	class="modal-dialog fixed inset-0 z-50 m-auto w-full max-w-lg bg-transparent p-0 text-fg"
	data-state={panelState}
	oncancel={handleCancel}
	onclose={handleClose}
	onclick={handleBackdropClick}
>
	<div
		class="modal-panel rounded-2xl border border-border bg-surface-shell-opaque p-6 shadow-xl"
		style:transition-duration={`${transitionDurationMs}ms`}
		style:transition-timing-function={transitionEasing(1).toString()}
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
</dialog>

<style>
	.modal-dialog {
		opacity: 0;
		transition:
			opacity 200ms ease,
			overlay 200ms ease allow-discrete,
			display 200ms ease allow-discrete;
	}

	.modal-dialog[data-state='opening'],
	.modal-dialog[data-state='open'] {
		opacity: 1;
	}

	.modal-panel {
		opacity: 0;
		transform: translateY(100px);
		transition:
			opacity 200ms ease,
			transform 200ms ease;
	}

	.modal-dialog[data-state='opening'] .modal-panel,
	.modal-dialog[data-state='open'] .modal-panel {
		opacity: 1;
		transform: translateY(0);
	}

	@starting-style {
		.modal-dialog[data-state='open'] {
			opacity: 0;
		}

		.modal-dialog[data-state='open'] .modal-panel {
			opacity: 0;
			transform: translateY(100px);
		}
	}

	.modal-dialog::backdrop {
		background: rgb(0 0 0 / 0);
		backdrop-filter: blur(0);
		transition:
			background 200ms ease,
			backdrop-filter 200ms ease,
			overlay 200ms ease allow-discrete,
			display 200ms ease allow-discrete;
	}

	.modal-dialog[data-state='opening']::backdrop,
	.modal-dialog[data-state='open']::backdrop {
		background: rgb(0 0 0 / 0.5);
		backdrop-filter: blur(6px);
	}

	@starting-style {
		.modal-dialog[data-state='open']::backdrop {
			background: rgb(0 0 0 / 0);
			backdrop-filter: blur(0);
		}
	}
</style>
