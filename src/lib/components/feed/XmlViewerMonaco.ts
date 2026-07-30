import * as monaco from 'monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';

import {
	registerJrssThemes,
	getJrssThemeName,
	isDarkMode,
	type MonacoThemeName
} from '$lib/utils/monaco-themes';
import type { XmlOffsetRange } from '$lib/utils/xml-inspector';

if (!(globalThis as typeof globalThis & { MonacoEnvironment?: unknown }).MonacoEnvironment) {
	(
		globalThis as typeof globalThis & {
			MonacoEnvironment: { getWorker: (_: unknown, label: string) => Worker };
		}
	).MonacoEnvironment = {
		getWorker() {
			return new editorWorker();
		}
	};
}

export interface XmlEditorController {
	setValue(value: string): void;
	setReadOnly(readOnly: boolean): void;
	setSelection(range: XmlOffsetRange | null): void;
	setTheme(isDark: boolean): void;
	dispose(): void;
}

export async function createXmlEditor(
	container: HTMLElement,
	initialValue: string,
	readOnly: boolean,
	onOffsetChange: (offset: number) => void
): Promise<XmlEditorController> {
	await import('monaco-editor/esm/vs/basic-languages/xml/xml.contribution');

	registerJrssThemes(monaco);

	const currentTheme: MonacoThemeName = getJrssThemeName(isDarkMode());
	monaco.editor.setTheme(currentTheme);

	const model = monaco.editor.createModel(initialValue, 'xml');

	const editor = monaco.editor.create(container, {
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

	const decorationsCollection = editor.createDecorationsCollection([]);

	editor.onDidChangeCursorPosition((event) => {
		onOffsetChange(model.getOffsetAt(event.position));
	});

	let activeTheme = currentTheme;

	const observer = new MutationObserver(() => {
		const newTheme = getJrssThemeName(isDarkMode());
		if (newTheme !== activeTheme) {
			activeTheme = newTheme;
			monaco.editor.setTheme(newTheme);
		}
	});

	observer.observe(document.documentElement, {
		attributes: true,
		attributeFilter: ['class']
	});

	return {
		setValue(value: string) {
			if (value !== model.getValue()) {
				model.setValue(value);
			}
		},

		setReadOnly(ro: boolean) {
			editor.updateOptions({ readOnly: ro });
		},

		setSelection(range: XmlOffsetRange | null) {
			if (!range) {
				decorationsCollection.clear();
				return;
			}

			const maxOffset = model.getValueLength();
			const safeStart = Math.max(0, Math.min(range.startOffset, maxOffset));
			const safeEnd = Math.max(safeStart + 1, Math.min(range.endOffset, maxOffset));

			const start = model.getPositionAt(safeStart);
			const end = model.getPositionAt(safeEnd);

			const selectionRange = new monaco.Range(
				start.lineNumber,
				start.column,
				end.lineNumber,
				end.column
			);

			decorationsCollection.set([
				{
					range: selectionRange,
					options: {
						className: 'xml-selected-node-decoration'
					}
				}
			]);

			editor.setPosition(start);
			editor.revealRangeInCenter(selectionRange);
		},

		setTheme(isDark: boolean) {
			const newTheme = getJrssThemeName(isDark);
			if (newTheme !== activeTheme) {
				activeTheme = newTheme;
				monaco.editor.setTheme(newTheme);
			}
		},

		dispose() {
			observer.disconnect();
			editor.dispose();
			model.dispose();
		}
	};
}
