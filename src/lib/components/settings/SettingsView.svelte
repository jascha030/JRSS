<script lang="ts">
	import { onMount } from 'svelte';

	import { clearAudioCache, loadAppSettings, saveAppSettings } from '$lib/services/feedService';
	import { isTauriRuntime } from '$lib/services/tauriClient';
	import { applyAccentColor, applyColorScheme } from '$lib/stores/app.svelte';
	import {
		DEFAULT_AUTO_REFRESH_INTERVAL_MINUTES,
		DEFAULT_COLOR_SCHEME,
		DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES,
		DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP,
		DEFAULT_ACCENT_COLOR,
		DEFAULT_SKIP_FORWARD_SECONDS,
		DEFAULT_SKIP_BACKWARD_SECONDS,
		type AppSettings
	} from '$lib/types/rss';
	import { APP_SETTINGS } from '$lib/config/settings';
	import SettingRow from './SettingRow.svelte';

	let isDesktop = $state(false);
	let isLoading = $state(true);
	let isSaving = $state(false);
	let errorMessage = $state('');
	let successMessage = $state('');
	let initialized = $state(false);
	let isClearingCache = $state(false);
	let cacheMessage = $state('');

	let pending = $state<AppSettings>({
		maxAudioCacheSizeBytes: DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES,
		miniPlayerAlwaysOnTop: DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP,
		autoRefreshIntervalMinutes: DEFAULT_AUTO_REFRESH_INTERVAL_MINUTES,
		colorScheme: DEFAULT_COLOR_SCHEME,
		accentColor: DEFAULT_ACCENT_COLOR,
		skipForwardSeconds: DEFAULT_SKIP_FORWARD_SECONDS,
		skipBackwardSeconds: DEFAULT_SKIP_BACKWARD_SECONDS
	});

	$effect(() => {
		const scheme = pending.colorScheme;
		if (!initialized) return;
		applyColorScheme(scheme);
	});

	$effect(() => {
		const accent = pending.accentColor;
		if (!initialized) return;
		applyAccentColor(accent);
	});

	onMount(async () => {
		isDesktop = isTauriRuntime();

		if (!isDesktop) {
			isLoading = false;
			initialized = true;
			return;
		}

		try {
			const settings = await loadAppSettings();
			Object.assign(pending, settings);
		} catch (error) {
			errorMessage = `Failed to load settings. ${getErrorMessage(error)}`;
		} finally {
			isLoading = false;
			initialized = true;
		}
	});

	let isFormValid = $derived(
		APP_SETTINGS.every((entry) => {
			if (entry.kind !== 'number') return true;
			if (!entry.validate || !entry.toDisplay) return true;
			const displayVal = entry.toDisplay(pending[entry.key]);
			return entry.validate(displayVal) === null;
		})
	);

	async function handleSave(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		errorMessage = '';
		successMessage = '';

		if (!isDesktop) {
			errorMessage = 'Settings are only available in the desktop app.';
			return;
		}

		for (const entry of APP_SETTINGS) {
			if (entry.kind !== 'number') continue;
			if (!entry.validate || !entry.toDisplay) continue;
			const displayVal = entry.toDisplay(pending[entry.key]);
			const validationError = entry.validate(displayVal);
			if (typeof validationError === 'string') {
				errorMessage = validationError;
				return;
			}
		}

		isSaving = true;
		try {
			const result = await saveAppSettings(pending);
			Object.assign(pending, result);
			successMessage = 'Settings saved.';
		} catch (error) {
			errorMessage = `Failed to save settings. ${getErrorMessage(error)}`;
		} finally {
			isSaving = false;
		}
	}

	async function handleClearCache(): Promise<void> {
		cacheMessage = '';
		isClearingCache = true;
		try {
			await clearAudioCache();
			cacheMessage = 'Cache cleared.';
		} catch (error) {
			cacheMessage = `Failed to clear cache. ${getErrorMessage(error)}`;
		} finally {
			isClearingCache = false;
		}
	}

	function getErrorMessage(error: unknown): string {
		return error instanceof Error ? error.message : 'Something went wrong.';
	}
</script>

<section class="flex-2 overflow-y-auto bg-surface-shell-opaque px-6 py-8 lg:px-8">
	<div class="max-w-4xl">
		<p class="text-sm font-medium tracking-[0.18em] text-fg-muted uppercase">Settings</p>
		<h1 class="mt-3 text-2xl font-semibold tracking-tight text-fg">Settings</h1>
		<p class="mt-2 text-sm text-fg-muted">
			Configure desktop playback behavior and how much downloaded audio JRSS can keep locally.
		</p>

		<form class="mt-8" onsubmit={handleSave}>
			<div class="rounded-2xl border border-border bg-surface p-6 shadow-sm">
				<div class="flex flex-col divide-y divide-border">
					{#each APP_SETTINGS as entry (entry.key)}
						<div class="py-6 first:pt-0 last:pb-0">
							<SettingRow {entry} {pending} disabled={isLoading || isSaving} {isDesktop} />
						</div>
					{/each}
					<div class="py-6 first:pt-0 last:pb-0">
						<div class="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
							<div class="max-w-2xl">
								<h2 class="text-base font-semibold text-fg">Clear audio cache</h2>
								<p class="mt-2 text-sm text-fg-muted">
									Remove all downloaded audio files from the local cache directory.
								</p>
							</div>
							<div class="flex w-full max-w-xs flex-col gap-2">
								<button
									type="button"
									class="btn rounded-xl preset-tonal"
									disabled={isClearingCache || !isDesktop}
									onclick={handleClearCache}
								>
									{isClearingCache ? 'Clearing…' : 'Clear cache'}
								</button>
								{#if cacheMessage}
									<p class="text-sm text-fg-muted">{cacheMessage}</p>
								{/if}
							</div>
						</div>
					</div>
				</div>

				<div class="mt-6 flex flex-wrap items-center gap-3">
					<button
						type="submit"
						class="btn rounded-xl preset-filled"
						disabled={isLoading || isSaving || !isDesktop || !isFormValid}
					>
						{isSaving ? 'Saving…' : 'Save settings'}
					</button>

					{#if isLoading}
						<p class="text-sm text-fg-muted">Loading settings…</p>
					{:else if successMessage}
						<p class="text-success text-sm">{successMessage}</p>
					{:else if errorMessage}
						<p class="text-error text-sm">{errorMessage}</p>
					{/if}
				</div>
			</div>
		</form>
	</div>
</section>
