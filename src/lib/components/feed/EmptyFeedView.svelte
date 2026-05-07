<script lang="ts">
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
</script>

<section class="flex flex-2 items-center justify-center overflow-y-auto px-6 py-12 lg:px-8">
	<EmptyState
		overline="No feeds yet"
		title="Add your first RSS feed or podcast"
		description="Add an RSS or Atom URL above, or paste an Apple Podcasts show link or ID. The desktop app resolves the feed and persists it in local SQLite."
		class="max-w-lg"
	/>
</section>
