<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { Toaster } from 'svelte-sonner';

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
<Toaster
	position="top-center"
	toastOptions={{
		unstyled: true,
		classes: {
			toast: 'toast-base',
			success: 'toast-success',
			error: 'toast-error',
			warning: 'toast-warning',
			info: 'toast-info',
			title: 'text-sm font-medium text-fg',
			description: 'text-xs text-fg-secondary mt-0.5'
		}
	}}
/>
{@render children()}
