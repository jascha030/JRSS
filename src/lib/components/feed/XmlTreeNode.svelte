<script lang="ts">
	import Self from './XmlTreeNode.svelte';

	import type { XmlTreeItem } from '$lib/utils/xml-inspector';
	import { nodeMatchesQuery } from '$lib/utils/xml-inspector';

	let {
		node,
		depth = 0,
		selectedId = null,
		expandedIds,
		forceOpenIds,
		query = '',
		onToggle,
		onSelect
	} = $props<{
		node: XmlTreeItem;
		depth?: number;
		selectedId?: string | null;
		expandedIds: Set<string>;
		forceOpenIds: Set<string>;
		query?: string;
		onToggle: (id: string) => void;
		onSelect: (id: string) => void;
	}>();

	let hasChildren = $derived(node.children.length > 0);
	let isSelected = $derived(selectedId === node.id);
	let isMatch = $derived(query.trim().length > 0 && nodeMatchesQuery(node, query));
	let isOpen = $derived(hasChildren && (forceOpenIds.has(node.id) || expandedIds.has(node.id)));
</script>

<div class="bg-surface-shell-opaque select-none">
	<div class="flex items-start gap-1 py-0.5" style={`padding-left:${depth * 12}px;`}>
		{#if hasChildren}
			<button
				type="button"
				class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded text-fg-muted hover:bg-surface-hover"
				onclick={() => onToggle(node.id)}
				aria-label={isOpen ? 'Collapse node' : 'Expand node'}
			>
				{isOpen ? '▾' : '▸'}
			</button>
		{:else}
			<span class="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center text-fg-muted">
				•
			</span>
		{/if}

		<button
			type="button"
			class={`flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1 text-left text-sm ${
				isSelected ? 'bg-surface-active text-fg' : 'text-fg hover:bg-surface-hover'
			} ${isMatch && !isSelected ? 'ring-1 ring-accent/30 ring-inset' : ''}`}
			onclick={() => onSelect(node.id)}
			title={node.path}
		>
			<span class="shrink-0 font-medium text-accent">{node.name}</span>

			{#if node.attributes.length > 0}
				<span class="shrink-0 text-xs text-fg-subtle">
					{node.attributes.length} attrs
				</span>
			{/if}

			{#if node.textPreview}
				<span class="truncate text-xs text-fg-muted">
					{node.textPreview}
				</span>
			{/if}
		</button>
	</div>

	{#if hasChildren && isOpen}
		{#each node.children as child (child.id)}
			<Self
				node={child}
				depth={depth + 1}
				{selectedId}
				{expandedIds}
				{forceOpenIds}
				{query}
				{onToggle}
				{onSelect}
			/>
		{/each}
	{/if}
</div>
