/**
 * Canonical settings definition — the single source of truth for the settings UI.
 *
 * To add a new setting:
 *   1. Add the field to `AppSettings` in `rss.ts` and `AppSettingsRecord` in
 *      `src-tauri/src/models.rs`, with a migration in `schema.rs`.
 *   2. Add an entry here. TypeScript will enforce key ↔ kind alignment.
 *
 * The order of entries controls the render order in the settings page.
 */

import type { SettingsDefinition } from '$lib/types/settings';

const BYTES_PER_GB = 1024 * 1024 * 1024;

export const APP_SETTINGS = [
	{
		key: 'colorScheme',
		kind: 'segmented',
		label: 'Color scheme',
		description: 'Choose whether JRSS follows your OS preference or forces a specific appearance.',
		hint: 'Applied immediately, saved with other settings.',
		options: [
			{ value: 'system', label: 'System' },
			{ value: 'light', label: 'Light' },
			{ value: 'dark', label: 'Dark' }
		]
	},
	{
		key: 'accentColor',
		kind: 'color',
		label: 'Accent color',
		description: 'Override the default accent color used for buttons, links, and highlights.',
		hint: 'Applied immediately as you pick. Saved with other settings.',
		fallbackHex: '#4f46e5'
	},
	{
		key: 'miniPlayerAlwaysOnTop',
		kind: 'toggle',
		label: 'Mini-player window',
		description: 'Choose whether the mini-player stays above other windows when it opens.',
		checkboxLabel: 'Always on top',
		hint: 'Applies the next time the mini-player window is opened.',
		desktopOnly: true
	},
	{
		key: 'maxAudioCacheSizeBytes',
		kind: 'number',
		label: 'Max audio cache size',
		description: 'Set the maximum amount of disk space used for cached audio files.',
		unit: 'GB',
		min: 0.1,
		step: 0.1,
		toDisplay: (bytes: number) => Number((bytes / BYTES_PER_GB).toFixed(2)),
		fromDisplay: (gb: number) => Math.round(gb * BYTES_PER_GB),
		validate: (gb: number) => (gb > 0 ? null : 'Enter a value greater than 0 GB.'),
		desktopOnly: true
	},
	{
		key: 'autoRefreshIntervalMinutes',
		kind: 'select',
		label: 'Auto-refresh',
		description: 'Automatically check for new episodes and articles in the background.',
		hint: 'Changes take effect immediately.',
		options: [
			{ value: 0, label: 'Off' },
			{ value: 15, label: 'Every 15 minutes' },
			{ value: 30, label: 'Every 30 minutes' },
			{ value: 60, label: 'Every hour' },
			{ value: 120, label: 'Every 2 hours' }
		],
		desktopOnly: true
	}
] satisfies SettingsDefinition;
