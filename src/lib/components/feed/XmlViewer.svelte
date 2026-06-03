<script lang="ts">
	import { onMount } from 'svelte';
	import type { XmlOffsetRange } from '$lib/utils/xml-inspector';
	import type { XmlEditorController } from './XmlViewerMonaco';

	let {
		value = '',
		height = '100%',
		readOnly = true,
		selectedRange = null,
		onOffsetChange = () => {}
	} = $props<{
		value?: string;
		height?: string;
		readOnly?: boolean;
		selectedRange?: XmlOffsetRange | null;
		onOffsetChange?: (offset: number) => void;
	}>();

	let container = $state<HTMLDivElement | null>(null);
	let controller = $state<XmlEditorController | null>(null);

	onMount(() => {
		let disposed = false;

		void (async () => {
			if (!container) {
				return;
			}

			const { createXmlEditor } = await import('./XmlViewerMonaco');

			if (disposed) {
				return;
			}

			controller = await createXmlEditor(container, value, readOnly, onOffsetChange);
		})();

		return () => {
			disposed = true;
			controller?.dispose();
			controller = null;
		};
	});

	$effect(() => {
		controller?.setValue(value);
	});

	$effect(() => {
		controller?.setReadOnly(readOnly);
	});

	$effect(() => {
		controller?.setSelection(selectedRange);
	});
</script>

<div bind:this={container} class="xml-viewer" style:height></div>

<style>
	.xml-viewer {
		width: 100%;
		min-height: 0;
	}

	:global(.monaco-editor .xml-selected-node-decoration) {
		background: color-mix(in oklch, var(--color-accent) 20%, transparent);
		border-radius: 3px;
	}
</style>
