<script lang="ts">
	import XmlNodeView from './XmlNodeView.svelte';

	type XmlNode = {
		name: string;
		attributes: Array<{ name: string; value: string }>;
		children: XmlNode[];
		text: string;
	};

	type Props = {
		node: XmlNode;
		depth: number;
	};

	let { node, depth }: Props = $props();

	let collapsedMap = $state<Record<string, boolean>>({});

	let nodeKey = $derived(`${depth}-${node.name}`);

	function toggleCollapse(): void {
		collapsedMap = { ...collapsedMap, [nodeKey]: !collapsedMap[nodeKey] };
	}

	let isCollapsed = $derived(collapsedMap[nodeKey] ?? false);
	let hasChildren = $derived(node.children.length > 0);
	let indent = $derived('  '.repeat(depth));
	let isEmpty = $derived(node.children.length === 0 && !node.text);
	let nl = '\n';

	function attrString(): string {
		if (node.attributes.length === 0) return '';
		return ' ' + node.attributes.map((a) => `${a.name}="${a.value}"`).join(' ');
	}
</script>

{#if hasChildren}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<span
		class="cursor-pointer text-fg-muted select-none"
		onclick={toggleCollapse}
		tabindex="0"
		role="button"
		aria-expanded={!isCollapsed}>{indent}{isCollapsed ? '▸ ' : '▾ '}</span
	><span class="text-accent">&lt;{node.name}{attrString()}&gt;{nl}</span>
	{#if isCollapsed}
		<span class="text-fg-muted">{indent} …{nl}</span>
		<span class="text-accent">{indent}&lt;/{node.name}&gt;{nl}</span>
	{:else}
		{#each node.children as child, idx (`${depth}-${idx}`)}
			<XmlNodeView node={child} depth={depth + 1} />
		{/each}
		<span class="text-accent">{indent}&lt;/{node.name}&gt;{nl}</span>
	{/if}
{:else if isEmpty}
	<span class="text-accent">{indent}&lt;{node.name}{attrString()} /&gt;{nl}</span>
{:else}
	<span class="text-accent">{indent}&lt;{node.name}{attrString()}&gt;</span><span class="text-fg"
		>{node.text}</span
	><span class="text-accent">&lt;/{node.name}&gt;{nl}</span>
{/if}
