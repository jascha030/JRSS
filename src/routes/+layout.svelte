<script lang="ts">
	import './layout.css';
	import { Toaster } from 'svelte-sonner';

	let { children } = $props();
</script>

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
