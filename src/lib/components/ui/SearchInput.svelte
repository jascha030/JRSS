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
	class="input-group pointer-events-auto h-9 grid-cols-[auto_1fr_auto] rounded-xl border input-group-integrated border-border transition-colors focus-within:ring-2"
>
	<div class="flex items-center justify-center px-3">
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
		class="ig-input placeholder:text-fg-muted"
		{placeholder}
		type="search"
		autocomplete="off"
		{...rest}
	/>

	{#if kbdShortcuts && kbdShortcuts.length > 0}
		<div class="flex items-center gap-1 px-3">
			{#each kbdShortcuts as key (key)}
				<kbd class="kbd">
					{key}
				</kbd>
			{/each}
		</div>
	{/if}
</div>
<!-- </div> -->
