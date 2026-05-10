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
		bgClass?: string;
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
		bgClass = 'bg-surface-sidebar',
		...rest
	}: Props = $props();
</script>

{#if label}
	<label class="sr-only" for={id}>{label}</label>
{/if}

<div
	class="flex h-9 items-center gap-2 rounded-xl border border-border {bgClass} px-3 transition-colors focus-within:border-border-hover focus-within:ring-2 focus-within:ring-ring pointer-events-auto"
>
	{#if isLoading}
		<Icon icon="lucide:loader-circle" class="size-4 shrink-0 animate-spin text-fg-muted" />
	{:else}
		<Icon icon="lucide:search" class="size-4 shrink-0 text-fg-muted" />
	{/if}

	<input
		{id}
		bind:this={inputRef}
		bind:value
		class="min-w-0 flex-1 bg-transparent text-sm text-fg outline-none placeholder:text-fg-muted [&::-webkit-search-cancel-button]:hidden"
		{placeholder}
		type="search"
		autocomplete="off"
		{...rest}
	/>

	{#if kbdShortcuts && kbdShortcuts.length > 0}
		<div class="flex items-center gap-1">
			{#each kbdShortcuts as key (key)}
				<kbd
					class="rounded border border-border bg-surface-sidebar-hover px-1.5 py-0.5 text-xs font-medium text-fg-muted select-none"
				>
					{key}
				</kbd>
			{/each}
		</div>
	{/if}
</div>
