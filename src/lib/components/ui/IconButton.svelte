<script lang="ts">
	import Icon from '@iconify/svelte';

	type Variant = 'accent' | 'subtle' | 'tonal' | 'error' | 'icon-subtle';

	type Props = {
		icon: string;
		title?: string;
		label?: string;
		variant?: Variant;
		class?: string;
		iconClass?: string;
		disabled?: boolean;
		type?: 'button' | 'submit';
		onclick?: (event: MouseEvent) => void;
	};

	const variantClasses: Record<Variant, string> = {
		accent: 'preset-filled-accent btn-icon rounded-xl',
		subtle: 'preset-outlined-subtle btn-icon rounded-xl',
		tonal: 'preset-tonal btn-icon rounded-xl',
		error: 'preset-filled-error btn-icon rounded-xl',
		'icon-subtle': 'preset-icon-subtle btn-icon rounded-xl'
	};

	let {
		icon,
		title,
		label,
		variant = 'subtle',
		class: className = '',
		iconClass = 'size-4',
		disabled = false,
		type = 'button',
		onclick
	}: Props = $props();
</script>

<button
	{type}
	{disabled}
	{title}
	class="{variantClasses[variant]} {className}"
	aria-label={label ?? title}
	{onclick}
>
	<Icon {icon} class={iconClass} />
</button>
