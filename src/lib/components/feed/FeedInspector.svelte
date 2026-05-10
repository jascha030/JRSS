<script lang="ts">
	import Icon from '@iconify/svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import XmlTreeNode from '$lib/components/feed/XmlTreeNode.svelte';
	import XmlViewer from '$lib/components/feed/XmlViewer.svelte';
	import { inspectorState, closeInspector } from '$lib/state/inspector.svelte';
	import { SvelteSet } from 'svelte/reactivity';

	import {
		collectNodeIds,
		countNodes,
		filterXmlTree,
		findDeepestNodeAtOffset,
		findNodeById,
		getLineAndColumnFromOffset,
		getPathToNodeIds,
		inspectXml
	} from '$lib/utils/xml-inspector';

	let treeQuery = $state('');
	let expandedIds = $state(new SvelteSet<string>());
	let selectedNodeId = $state<string | null>(null);
	let lastFeedId = $state<string | null>(null);

	let inspection = $derived.by(() => {
		if (!inspectorState.xmlContent) {
			return {
				formatted: '',
				root: null,
				error: null
			};
		}

		return inspectXml(inspectorState.xmlContent);
	});

	let formattedXml = $derived.by(() => inspection.formatted || inspectorState.xmlContent || '');

	let filteredRoot = $derived.by(() => filterXmlTree(inspection.root, treeQuery));

	let forceOpenIds = $derived.by(() => {
		if (!treeQuery.trim() || !filteredRoot) {
			return new Set<string>();
		}

		return new Set(collectNodeIds(filteredRoot));
	});

	let visibleCount = $derived.by(() => countNodes(filteredRoot));

	let selectedNode = $derived.by(() => {
		if (!inspection.root || !selectedNodeId) {
			return null;
		}

		return findNodeById(inspection.root, selectedNodeId);
	});

	let selectedRange = $derived.by(() => selectedNode?.range ?? null);

	let selectedLocation = $derived.by(() => {
		if (!selectedNode || !formattedXml) {
			return null;
		}

		return {
			start: getLineAndColumnFromOffset(formattedXml, selectedNode.range.startOffset),
			end: getLineAndColumnFromOffset(formattedXml, selectedNode.range.endOffset)
		};
	});

	function expandPathToNode(id: string): void {
		if (!inspection.root) {
			return;
		}

		for (const pathId of getPathToNodeIds(inspection.root, id)) {
			expandedIds.add(pathId);
		}
	}

	function handleSelectNode(id: string): void {
		selectedNodeId = id;
		expandPathToNode(id);
	}

	function handleToggleNode(id: string): void {
		if (expandedIds.has(id)) {
			expandedIds.delete(id);
		} else {
			expandedIds.add(id);
		}
	}

	function handleEditorOffsetChange(offset: number): void {
		if (!inspection.root) {
			return;
		}

		const hit = findDeepestNodeAtOffset(inspection.root, offset);

		if (!hit || hit.id === selectedNodeId) {
			return;
		}

		selectedNodeId = hit.id;
		expandPathToNode(hit.id);
	}

	function expandAllVisible(): void {
		const source = filteredRoot ?? inspection.root;

		if (!source) {
			return;
		}

		expandedIds.clear();
		for (const id of collectNodeIds(source)) {
			expandedIds.add(id);
		}
	}

	function collapseTree(): void {
		expandedIds.clear();

		if (inspection.root) {
			expandedIds.add(inspection.root.id);
		}
	}

	function clearTreeQuery(): void {
		treeQuery = '';
	}

	$effect(() => {
		const feedId = inspectorState.activeFeedId ?? null;

		if (feedId !== lastFeedId) {
			lastFeedId = feedId;
			treeQuery = '';
			expandedIds.clear();
			selectedNodeId = null;
		}
	});

	$effect(() => {
		const root = inspection.root;

		if (!root) {
			return;
		}

		if (!selectedNodeId || !findNodeById(root, selectedNodeId)) {
			selectedNodeId = root.id;
			expandPathToNode(root.id);
		}
	});

	$effect(() => {
		if (!treeQuery.trim() || !filteredRoot) {
			return;
		}

		if (selectedNodeId && !findNodeById(filteredRoot, selectedNodeId)) {
			selectedNodeId = filteredRoot.id;
			expandPathToNode(filteredRoot.id);
		}
	});
</script>

{#if inspectorState.activeFeedId}
	<section class="flex h-full w-full flex-1 flex-col overflow-hidden bg-surface">
		<div
			class="flex shrink-0 items-center justify-between border-b border-border bg-surface-shell-opaque px-6 py-4 lg:px-8"
		>
			<div>
				<h2 class="text-lg font-semibold tracking-tight text-fg">Feed inspector</h2>
				<p class="mt-1 text-xs text-fg-subtle">
					Raw XML for feed
					<code class="rounded bg-surface-active px-1 text-xs">
						{inspectorState.activeFeedId}
					</code>
				</p>
			</div>

			<IconButton
				icon="lucide:x"
				title="Close inspector"
				label="Close inspector"
				variant="subtle"
				onclick={closeInspector}
			/>
		</div>

		<div class="min-h-0 flex-1 overflow-hidden bg-surface-shell-opaque">
			{#if inspectorState.loading}
				<div class="flex h-full items-center justify-center py-16 text-fg-muted">
					<Icon icon="lucide:loader-circle" class="mr-2 size-5 animate-spin" />
					Fetching feed XML…
				</div>
			{:else if inspectorState.error}
				<div class="text-fg-error flex h-full items-center justify-center py-16">
					<p>{inspectorState.error}</p>
				</div>
			{:else if inspectorState.xmlContent}
				<div class="flex h-full min-h-0">
					<aside class="flex w-96 shrink-0 flex-col border-r border-border bg-surface/60">
						<div class="shrink-0 border-b border-border p-3">
							<div class="flex items-center gap-2">
								<input
									bind:value={treeQuery}
									type="search"
									placeholder="Filter by tag, attribute, text, or path"
									class="w-full rounded-md border border-border bg-surface px-3 py-2 text-sm text-fg outline-none placeholder:text-fg-muted"
								/>

								{#if treeQuery}
									<button
										type="button"
										class="rounded-md border border-border px-2 py-2 text-xs text-fg-muted hover:bg-surface-hover hover:text-fg"
										onclick={clearTreeQuery}
									>
										Clear
									</button>
								{/if}
							</div>

							<div class="mt-2 flex items-center justify-between text-xs text-fg-muted">
								<span>
									{#if treeQuery.trim()}
										{visibleCount} matching node{visibleCount === 1 ? '' : 's'}
									{:else}
										{countNodes(inspection.root)} node{countNodes(inspection.root) === 1 ? '' : 's'}
									{/if}
								</span>

								<div class="flex items-center gap-1">
									<button
										type="button"
										class="rounded-md border border-border px-2 py-1 hover:bg-surface-hover hover:text-fg"
										onclick={expandAllVisible}
									>
										Expand all
									</button>
									<button
										type="button"
										class="rounded-md border border-border px-2 py-1 hover:bg-surface-hover hover:text-fg"
										onclick={collapseTree}
									>
										Collapse
									</button>
								</div>
							</div>
						</div>

						<div class="min-h-0 flex-1 overflow-auto p-2">
							{#if inspection.root && filteredRoot}
								<XmlTreeNode
									node={filteredRoot}
									selectedId={selectedNodeId}
									{expandedIds}
									{forceOpenIds}
									query={treeQuery}
									onToggle={handleToggleNode}
									onSelect={handleSelectNode}
								/>
							{:else if inspection.error}
								<div
									class="rounded-md border border-border bg-surface-active p-3 text-sm text-fg-muted"
								>
									Failed to build the XML tree. The raw XML is still shown on the right.
								</div>
							{:else if treeQuery.trim()}
								<div
									class="rounded-md border border-border bg-surface-active p-3 text-sm text-fg-muted"
								>
									No matches for <span class="font-medium text-fg">{treeQuery}</span>.
								</div>
							{:else}
								<div
									class="rounded-md border border-border bg-surface-active p-3 text-sm text-fg-muted"
								>
									No XML tree available.
								</div>
							{/if}
						</div>

						<div class="shrink-0 border-t border-border p-3">
							{#if selectedNode}
								<div class="space-y-3 text-sm">
									<div>
										<div class="text-xs tracking-wide text-fg-muted uppercase">Selected node</div>
										<div class="mt-1 font-medium text-fg">
											{selectedNode.name}
										</div>
										<code
											class="mt-2 block overflow-x-auto rounded-md bg-surface-active px-2 py-1 text-xs text-fg-subtle"
										>
											{selectedNode.path}
										</code>
									</div>

									<div class="grid grid-cols-2 gap-2 text-xs">
										<div class="rounded-md bg-surface-active px-2 py-2">
											<div class="text-fg-muted">Children</div>
											<div class="mt-1 text-fg">
												{selectedNode.children.length}
											</div>
										</div>

										<div class="rounded-md bg-surface-active px-2 py-2">
											<div class="text-fg-muted">Attributes</div>
											<div class="mt-1 text-fg">
												{selectedNode.attributes.length}
											</div>
										</div>
									</div>

									{#if selectedLocation}
										<div class="rounded-md bg-surface-active px-2 py-2 text-xs">
											<div class="text-fg-muted">Location</div>
											<div class="mt-1 text-fg">
												L{selectedLocation.start.line}:C{selectedLocation.start.column}
												→ L{selectedLocation.end.line}:C{selectedLocation.end.column}
											</div>
											<div class="mt-1 text-fg-subtle">
												Offsets {selectedNode.range.startOffset}–
												{selectedNode.range.endOffset}
											</div>
										</div>
									{/if}

									<div>
										<div class="text-xs tracking-wide text-fg-muted uppercase">Attributes</div>

										{#if selectedNode.attributes.length > 0}
											<div class="mt-2 space-y-2">
												{#each selectedNode.attributes as attribute (`${selectedNode.id}-${attribute.name}-${attribute.value}`)}
													<div class="rounded-md bg-surface-active px-2 py-2 text-xs">
														<div class="font-medium text-fg">
															{attribute.name}
														</div>
														<div class="mt-1 break-all text-fg-subtle">
															{attribute.value}
														</div>
													</div>
												{/each}
											</div>
										{:else}
											<div class="mt-2 text-xs text-fg-muted">No attributes</div>
										{/if}
									</div>

									{#if selectedNode.textPreview}
										<div>
											<div class="text-xs tracking-wide text-fg-muted uppercase">Text preview</div>
											<div
												class="mt-2 rounded-md bg-surface-active px-2 py-2 text-xs text-fg-subtle"
											>
												{selectedNode.textPreview}
											</div>
										</div>
									{/if}
								</div>
							{:else}
								<div class="text-sm text-fg-muted">Select a node to inspect its details.</div>
							{/if}
						</div>
					</aside>

					<div class="min-w-0 flex-1 p-4">
						{#if inspection.error}
							<div
								class="text-fg-error mb-3 rounded-md border border-border bg-surface-active px-3 py-2 text-sm"
							>
								Failed to build the XML tree. Showing raw XML in the editor.
							</div>
						{/if}

						<div class="h-full overflow-hidden rounded-lg border border-border">
							<XmlViewer
								value={formattedXml}
								height="100%"
								readOnly={true}
								{selectedRange}
								onOffsetChange={handleEditorOffsetChange}
							/>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</section>
{/if}
