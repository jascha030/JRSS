<script lang="ts">
	import Icon from '@iconify/svelte';

	type Props = {
		icon?: string;
		overline?: string;
		title: string;
		description?: string;
		class?: string;
		iconClass?: string;
		titleClass?: string;
		descriptionClass?: string;
	};

	let {
		icon,
		overline,
		title,
		description,
		class: className = '',
		iconClass = 'mb-3 size-10 text-fg-subtle',
		titleClass = 'text-xl font-semibold text-fg',
		descriptionClass = 'mx-auto mt-3 max-w-xl text-sm leading-6 text-fg-secondary'
	}: Props = $props();
</script>

<div class="flex flex-col items-center justify-center px-6 py-12 text-center {className}">
	{#if icon}
		<Icon {icon} class={iconClass} />
	{/if}
	{#if overline}
		<p class="text-xs font-medium tracking-[0.18em] text-fg-muted uppercase">{overline}</p>
	{/if}
	<h2 class="{titleClass} {overline ? 'mt-3' : ''}">{title}</h2>
	{#if description}
		<p class={descriptionClass}>{description}</p>
	{/if}
</div>
