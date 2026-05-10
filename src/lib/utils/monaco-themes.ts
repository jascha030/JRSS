import type * as Monaco from 'monaco-editor';

/**
 * Monaco Editor theme definitions based on standard VS Code themes
 * with JRSS UI-matched background colors.
 */

export type MonacoThemeName = 'jrss-light' | 'jrss-dark';

interface ThemeDefinition {
	base: 'vs' | 'vs-dark' | 'hc-black';
	inherit: boolean;
	rules: Monaco.editor.ITokenThemeRule[];
	colors: Record<string, string>;
}

const jrssLightTheme: ThemeDefinition = {
	base: 'vs',
	inherit: true,
	rules: [],
	colors: {
		'editor.background': '#F8FAFC',
		'editor.lineHighlightBackground': '#F1F5F9',
		'editorWidget.background': '#FFFFFF',
		'editorSuggestWidget.background': '#FFFFFF'
	}
};

const jrssDarkTheme: ThemeDefinition = {
	base: 'vs-dark',
	inherit: true,
	rules: [],
	colors: {
		'editor.background': '#0F172A',
		'editor.lineHighlightBackground': '#1E293B',
		'editorWidget.background': '#1E293B',
		'editorSuggestWidget.background': '#0F172A'
	}
};

const registeredThemes = new Set<string>();

/**
 * Register JRSS custom themes with Monaco.
 * Safe to call multiple times; only registers once per theme.
 */
export function registerJrssThemes(monaco: typeof import('monaco-editor')): void {
	if (!registeredThemes.has('jrss-light')) {
		monaco.editor.defineTheme('jrss-light', jrssLightTheme);
		registeredThemes.add('jrss-light');
	}
	if (!registeredThemes.has('jrss-dark')) {
		monaco.editor.defineTheme('jrss-dark', jrssDarkTheme);
		registeredThemes.add('jrss-dark');
	}
}

/**
 * Get the appropriate JRSS Monaco theme name based on the app's color scheme.
 * Returns 'jrss-dark' for dark mode, 'jrss-light' for light mode.
 */
export function getJrssThemeName(isDark: boolean): MonacoThemeName {
	return isDark ? 'jrss-dark' : 'jrss-light';
}

/**
 * Check if the document is in dark mode by looking for the .dark class on <html>.
 * This matches the app's theme system in theme.svelte.ts.
 */
export function isDarkMode(): boolean {
	return document.documentElement.classList.contains('dark');
}
