<script lang="ts">
	import Icon from '@iconify/svelte';
	import IconButton from '$lib/components/ui/IconButton.svelte';
	import XmlNodeView from './XmlNodeView.svelte';
	import { inspectorState, closeInspector } from '$lib/state/inspector.svelte';

	type XmlNode = {
		name: string;
		attributes: Array<{ name: string; value: string }>;
		children: XmlNode[];
		text: string;
	};

	let rootNode = $derived.by((): XmlNode | null => {
		if (!inspectorState.xmlContent) return null;

		try {
			const parser = new DOMParser();
			const doc = parser.parseFromString(inspectorState.xmlContent, 'text/xml');

			const errorNode = doc.querySelector('parsererror');
			if (errorNode) {
				inspectorState.error = 'Failed to parse XML.';
				return null;
			}

			return domToXmlNode(doc.documentElement);
		} catch {
			inspectorState.error = 'Failed to parse XML.';
			return null;
		}
	});

	function domToXmlNode(element: Element): XmlNode {
		const attrs: Array<{ name: string; value: string }> = [];
		for (const attr of element.attributes) {
			attrs.push({ name: attr.name, value: attr.value });
		}

		const children: XmlNode[] = [];
		let text = '';

		for (const child of element.childNodes) {
			if (child.nodeType === Node.ELEMENT_NODE) {
				children.push(domToXmlNode(child as Element));
			} else if (child.nodeType === Node.TEXT_NODE || child.nodeType === Node.CDATA_SECTION_NODE) {
				const content = child.textContent?.trim() ?? '';
				if (content) {
					text += content;
				}
			}
		}

		return {
			name: element.tagName ?? element.nodeName,
			attributes: attrs,
			children,
			text
		};
	}
</script>

{#if inspectorState.activeFeedId}
	<section class="flex h-full w-full flex-1 flex-col overflow-hidden bg-surface">
		<div
			class="flex shrink-0 items-center justify-between border-b border-border px-6 py-4 lg:px-8"
		>
			<div>
				<h2 class="text-lg font-semibold tracking-tight text-fg">Feed Inspector</h2>
				<p class="mt-1 text-xs text-fg-subtle">
					Raw XML for feed
					<code class="rounded bg-surface-active px-1 text-xs">{inspectorState.activeFeedId}</code>
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

		<div class="min-h-0 flex-1 overflow-auto">
			{#if inspectorState.loading}
				<div class="flex h-full items-center justify-center py-16 text-fg-muted">
					<Icon icon="lucide:loader-circle" class="mr-2 size-5 animate-spin" />
					Fetching feed XML…
				</div>
			{:else if inspectorState.error}
				<div class="text-fg-error flex h-full items-center justify-center py-16">
					<p>{inspectorState.error}</p>
				</div>
			{:else if rootNode}
				<div class="p-4">
					<pre class="font-mono text-sm leading-relaxed text-fg"><XmlNodeView
							node={rootNode}
							depth={0}
						/></pre>
				</div>
			{/if}
		</div>
	</section>
{/if}
