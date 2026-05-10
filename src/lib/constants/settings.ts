/**
 * Canonical settings definition — the single source of truth for the settings UI.
 *
 * The order of sections and entries controls the render order in the settings page.
 *
 * @see {@link https://github.com/jascha030/JRSS/blob/main/AGENTS.md#adding-a-new-setting|Adding a new setting}
 */

import type { SettingsDefinition } from '$lib/types/settings';

const BYTES_PER_GB = 1024 * 1024 * 1024;

export const APP_SETTINGS = [
	{
		title: 'Appearance',
		description: 'Personalise how JRSS looks.',
		entries: [
			{
				key: 'colorScheme',
				kind: 'segmented',
				label: 'Color scheme',
				description:
					'Choose whether JRSS follows your OS preference or forces a specific appearance.',
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
			}
		]
	},
	{
		title: 'Playback',
		description: 'Control how audio and the mini-player behave.',
		entries: [
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
				key: 'skipForwardSeconds',
				kind: 'number',
				label: 'Skip forward',
				description: 'How many seconds to skip forward when pressing the fast-forward button.',
				unit: 'seconds',
				min: 1,
				step: 1,
				validate: (v: number) => (v >= 1 ? null : 'Must be at least 1 second.')
			},
			{
				key: 'skipBackwardSeconds',
				kind: 'number',
				label: 'Skip backward',
				description: 'How many seconds to skip backward when pressing the rewind button.',
				unit: 'seconds',
				min: 1,
				step: 1,
				validate: (v: number) => (v >= 1 ? null : 'Must be at least 1 second.')
			}
		]
	},
	{
		title: 'Storage & refreshing',
		description: 'Manage disk usage and automatic feed updates.',
		entries: [
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
		]
	}
] satisfies SettingsDefinition;
