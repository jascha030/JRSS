<script lang="ts">
	import Icon from '@iconify/svelte';

	type Props = {
		id?: string;
		label?: string;
		placeholder?: string;
		value?: string;
		inputRef?: HTMLInputElement | null;
		kbdShortcuts?: string[];
		isLoading?: boolean;
		oninput?: (event: Event & { currentTarget: HTMLInputElement }) => void;
		onkeydown?: (event: KeyboardEvent & { currentTarget: HTMLInputElement }) => void;
		onfocus?: (event: FocusEvent & { currentTarget: HTMLInputElement }) => void;
		onblur?: (event: FocusEvent & { currentTarget: HTMLInputElement }) => void;
	};

	let {
		id,
		label,
		placeholder,
		value = $bindable(''),
		inputRef = $bindable(null),
		kbdShortcuts,
		isLoading = false,
		...rest
	}: Props = $props();
</script>

{#if label}
	<label class="sr-only" for={id}>{label}</label>
{/if}

<div
	class="input-group h-9 grid-cols-[auto_1fr_auto] rounded-xl border border-border bg-surface-shell-opaque pointer-events-auto transition-colors focus-within:border-border-hover focus-within:ring-2"
>
	<div class="ig-cell preset-tonal">
		{#if isLoading}
			<Icon icon="lucide:loader-circle" class="size-4 shrink-0 animate-spin text-fg-muted" />
		{:else}
			<Icon icon="lucide:search" class="size-4 shrink-0 text-fg-muted" />
		{/if}
	</div>

	<input
		{id}
		bind:this={inputRef}
		bind:value
		class="ig-input placeholder:text-fg-muted focus:ring-0"
		{placeholder}
		type="search"
		autocomplete="off"
		{...rest}
	/>

	{#if kbdShortcuts && kbdShortcuts.length > 0}
		<div class="ig-cell border-none flex gap-1">
			{#each kbdShortcuts as key (key)}
				<kbd class="kbd">
					{key}
				</kbd>
			{/each}
		</div>
	{/if}
</div>
<!-- </div> -->
