<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
</script>

<section
	class="flex h-full flex-col items-center justify-center bg-surface-shell-opaque p-8 text-center"
>
	<h1 class="text-4xl font-bold text-fg">Error</h1>
	<p class="mt-4 text-lg text-fg-muted">
		{page.error?.message ?? 'Something went wrong.'}
	</p>
	<a href={resolve('/')} class="mt-6 text-accent hover:underline">Go home</a>
</section>
