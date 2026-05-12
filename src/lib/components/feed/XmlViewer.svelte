<script lang="ts">
	import { onMount } from 'svelte';
	import type * as Monaco from 'monaco-editor';
	import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';

	import type { XmlOffsetRange } from '$lib/utils/xml-inspector';
	import {
		registerJrssThemes,
		getJrssThemeName,
		isDarkMode,
		type MonacoThemeName
	} from '$lib/utils/monaco-themes';

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
	let editor = $state<Monaco.editor.IStandaloneCodeEditor | null>(null);
	let model = $state<Monaco.editor.ITextModel | null>(null);
	let monaco = $state<typeof import('monaco-editor') | null>(null);
	let currentTheme = $state<MonacoThemeName>(getJrssThemeName(isDarkMode()));

	let decorationsCollection = $state<Monaco.editor.IEditorDecorationsCollection | null>(null);

	if (!(globalThis as typeof globalThis & { MonacoEnvironment?: unknown }).MonacoEnvironment) {
		(
			globalThis as typeof globalThis & {
				MonacoEnvironment: {
					getWorker: (_: unknown, label: string) => Worker;
				};
			}
		).MonacoEnvironment = {
			getWorker() {
				return new editorWorker();
			}
		};
	}

	onMount(() => {
		let disposed = false;
		let observer: MutationObserver | null = null;

		void (async () => {
			if (!container) {
				return;
			}

			const monacoModule = await import('monaco-editor');
			await import('monaco-editor/esm/vs/basic-languages/xml/xml.contribution');

			if (disposed) {
				return;
			}

			monaco = monacoModule;

			registerJrssThemes(monaco);
			monaco.editor.setTheme(currentTheme);

			model = monaco.editor.createModel(value, 'xml');

			editor = monaco.editor.create(container, {
				model,
				readOnly,
				automaticLayout: true,
				minimap: { enabled: false },
				scrollBeyondLastLine: false,
				wordWrap: 'on',
				folding: true,
				foldingStrategy: 'indentation',
				showFoldingControls: 'always',
				lineNumbers: 'on',
				glyphMargin: false,
				stickyScroll: { enabled: true },
				tabSize: 2,
				renderWhitespace: 'selection',
				formatOnPaste: false,
				formatOnType: false,
				bracketPairColorization: {
					enabled: true
				}
			});

			decorationsCollection = editor.createDecorationsCollection([]);

			editor.onDidChangeCursorPosition((event) => {
				if (!model) {
					return;
				}

				const offset = model.getOffsetAt(event.position);
				onOffsetChange(offset);
			});

			// Watch for theme changes via the .dark class on <html>
			observer = new MutationObserver(() => {
				const newTheme = getJrssThemeName(isDarkMode());
				if (newTheme !== currentTheme && monaco) {
					currentTheme = newTheme;
					monaco.editor.setTheme(newTheme);
				}
			});

			observer.observe(document.documentElement, {
				attributes: true,
				attributeFilter: ['class']
			});
		})();

		return () => {
			disposed = true;
			observer?.disconnect();
			editor?.dispose();
			model?.dispose();
		};
	});

	$effect(() => {
		if (model && value !== model.getValue()) {
			model.setValue(value);
		}
	});

	$effect(() => {
		if (editor) {
			editor.updateOptions({ readOnly });
		}
	});

	$effect(() => {
		if (!editor || !model || !monaco) {
			return;
		}

		if (!selectedRange) {
			decorationsCollection?.clear();
			return;
		}

		const maxOffset = model.getValueLength();
		const safeStart = Math.max(0, Math.min(selectedRange.startOffset, maxOffset));
		const safeEnd = Math.max(safeStart + 1, Math.min(selectedRange.endOffset, maxOffset));

		const start = model.getPositionAt(safeStart);
		const end = model.getPositionAt(safeEnd);

		const range = new monaco.Range(start.lineNumber, start.column, end.lineNumber, end.column);

		decorationsCollection?.set([
			{
				range,
				options: {
					className: 'xml-selected-node-decoration'
				}
			}
		]);

		editor.setPosition(start);
		editor.revealRangeInCenter(range);
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
