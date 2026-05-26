/**
 * Theme State
 *
 * Owns color scheme (light/dark/system), accent color overrides, and custom
 * theme CSS injection. Applies them directly to `document.documentElement` so
 * the CSS `.dark` class, `--color-accent` custom property, and any injected
 * theme stylesheet are always in sync with stored settings.
 *
 * Call `initTheme(settings)` once during app startup (before first render).
 * Call `applyColorScheme` / `applyAccentColor` / `applyThemeCss` for immediate
 * live preview.
 */

import type { AppSettings, ColorScheme } from '$lib/types/settings';

let _colorScheme = $state<ColorScheme>('system');
let _accentColor = $state<string | null>(null);

/** Read-only view of the active theme settings. */
export const themeState = {
	get colorScheme(): ColorScheme {
		return _colorScheme;
	},
	get accentColor(): string | null {
		return _accentColor;
	}
};

let _mediaQuery: MediaQueryList | null = null;
let _onMediaChange: (() => void) | null = null;

function tearDownMediaListener(): void {
	if (_mediaQuery && _onMediaChange) {
		_mediaQuery.removeEventListener('change', _onMediaChange);
	}
	_mediaQuery = null;
	_onMediaChange = null;
}

function setDarkClass(isDark: boolean): void {
	document.documentElement.classList.toggle('dark', isDark);
}

/**
 * Apply a color scheme to `<html>`.
 *
 * - `'system'` — watches `prefers-color-scheme` and toggles `.dark` automatically.
 * - `'dark'`   — forces `.dark` class regardless of OS setting.
 * - `'light'`  — removes `.dark` class regardless of OS setting.
 *
 * Also sets the `color-scheme` CSS property so native browser elements
 * (scrollbars, form controls) render in the correct mode.
 */
export function applyColorScheme(scheme: ColorScheme): void {
	_colorScheme = scheme;
	tearDownMediaListener();

	const root = document.documentElement;

	if (scheme === 'system') {
		_mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		_onMediaChange = () => setDarkClass(_mediaQuery!.matches);
		_mediaQuery.addEventListener('change', _onMediaChange);
		setDarkClass(_mediaQuery.matches);
		root.style.removeProperty('color-scheme'); // falls back to CSS `light dark`
	} else if (scheme === 'dark') {
		setDarkClass(true);
		root.style.setProperty('color-scheme', 'dark');
	} else {
		setDarkClass(false);
		root.style.setProperty('color-scheme', 'light');
	}
}

/**
 * Override `--color-accent` on `:root`.
 *
 * Pass `null` to remove the override and fall back to the theme default.
 * `--color-accent-hover` and `--color-accent-dot` are derived via `color-mix()`
 * in CSS so only this one property needs to be set.
 */
export function applyAccentColor(color: string | null): void {
	_accentColor = color;
	const root = document.documentElement;

	if (color !== null) {
		root.style.setProperty('--color-accent', color);
	} else {
		root.style.removeProperty('--color-accent');
	}
}

const THEME_STYLE_ID = 'user-theme-css';

/** Inject or remove a `<style>` tag with custom theme CSS. */
export function applyThemeCss(css: string | null): void {
	let style = document.getElementById(THEME_STYLE_ID) as HTMLStyleElement | null;

	if (css === null || css.trim() === '') {
		if (style) {
			style.remove();
		}
		return;
	}

	if (!style) {
		style = document.createElement('style');
		style.id = THEME_STYLE_ID;
		document.head.appendChild(style);
	}

	style.textContent = css;
}

/**
 * Initialise the theme from persisted settings.
 * Called once in `initializeApp()` before the first render.
 */
export function initTheme(
	settings: Pick<AppSettings, 'colorScheme' | 'accentColor' | 'themeName'>,
	css?: string
): void {
	applyColorScheme(settings.colorScheme);
	applyAccentColor(settings.accentColor);
	applyThemeCss(css ?? null);
}

/** Reset theme to defaults (system scheme, no custom accent, no custom CSS). */
export function resetThemeState(): void {
	applyColorScheme('system');
	applyAccentColor(null);
	applyThemeCss(null);
}
