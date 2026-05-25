<script lang="ts">
	import type { SettingEntry } from '$lib/types/settings';
	import type { AppSettings } from '$lib/types/settings';
	import ToggleSetting from './inputs/ToggleSetting.svelte';
	import SegmentedSetting from './inputs/SegmentedSetting.svelte';
	import SelectSetting from './inputs/SelectSetting.svelte';
	import NumberSetting from './inputs/NumberSetting.svelte';
	import ColorSetting from './inputs/ColorSetting.svelte';

	interface Props {
		entry: SettingEntry;
		/** $state object from parent — mutations propagate reactively. */
		pending: AppSettings;
		/** true when loading or saving */
		disabled: boolean;
		isDesktop: boolean;
	}

	let { entry, pending = $bindable(), disabled, isDesktop }: Props = $props();

	const isDisabled = $derived(disabled || Boolean(entry.desktopOnly && !isDesktop));

	/**
	 * Type-safe setter — bridges the discriminated `entry.key` union to the
	 * matching `AppSettings` field without requiring `as` assertions.
	 */
	function set<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
		pending[key] = value;
	}
</script>

<div class="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
	<div class="max-w-2xl">
		<h2 class="text-base font-semibold text-fg">{entry.label}</h2>
		<p class="mt-2 text-sm text-fg-muted">{entry.description}</p>
		{#if entry.desktopOnly && !isDesktop}
			<p
				class="mt-4 rounded-xl border border-border bg-surface-hover px-4 py-3 text-sm text-fg-muted"
			>
				Desktop only. Run JRSS through Tauri to load and save these settings.
			</p>
		{/if}
	</div>

	<div class="flex w-full max-w-xs flex-col gap-2">
		{#if entry.kind === 'toggle'}
			<ToggleSetting
				def={entry}
				value={pending[entry.key]}
				disabled={isDisabled}
				onchange={(v) => set(entry.key, v)}
			/>
		{:else if entry.kind === 'segmented'}
			<SegmentedSetting
				def={entry}
				value={pending[entry.key]}
				disabled={isDisabled}
				onchange={(v) => set(entry.key, v)}
			/>
		{:else if entry.kind === 'number'}
			<NumberSetting
				def={entry}
				value={pending[entry.key]}
				disabled={isDisabled}
				onchange={(v) => set(entry.key, v)}
			/>
		{:else if entry.kind === 'select'}
			<SelectSetting
				def={entry}
				value={pending[entry.key]}
				disabled={isDisabled}
				onchange={(v) => set(entry.key, v)}
			/>
		{:else if entry.kind === 'color'}
			<ColorSetting
				def={entry}
				value={pending[entry.key]}
				disabled={isDisabled}
				onchange={(v) => set(entry.key, v)}
			/>
		{/if}

		{#if entry.hint}
			<p class="text-xs text-fg-muted">{entry.hint}</p>
		{/if}
	</div>
</div>
