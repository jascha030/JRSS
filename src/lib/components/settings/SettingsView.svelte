<script lang="ts">
	import { onMount } from 'svelte';

	import {
		clearAudioCache,
		discoverThemes,
		loadAppSettings,
		loadTheme,
		saveAppSettings
	} from '$lib/services/settings';
	import { isTauriRuntime } from '$lib/services/tauri';
	import { applyAccentColor, applyColorScheme, applyThemeCss } from '$lib/state';
	import {
		DEFAULT_AUTO_REFRESH_INTERVAL_MINUTES,
		DEFAULT_COLOR_SCHEME,
		DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES,
		DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP,
		DEFAULT_ACCENT_COLOR,
		DEFAULT_THEME_NAME,
		DEFAULT_SKIP_FORWARD_SECONDS,
		DEFAULT_SKIP_BACKWARD_SECONDS,
		type AppSettings,
		type SettingEntry,
		type ThemeInfo
	} from '$lib/types/settings';
	import { APP_SETTINGS } from '$lib/constants/settings';
	import SettingRow from './SettingRow.svelte';

	const allEntries: SettingEntry[] = [];
	for (const section of APP_SETTINGS) {
		allEntries.push(...section.entries);
	}

	let isDesktop = $state(false);
	let isLoading = $state(true);
	let isSaving = $state(false);
	let errorMessage = $state('');
	let successMessage = $state('');
	let initialized = $state(false);
	let isClearingCache = $state(false);
	let cacheMessage = $state('');
	let themes = $state<ThemeInfo[]>([]);
	let isLoadingThemes = $state(false);

	let pending = $state<AppSettings>({
		maxAudioCacheSizeBytes: DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES,
		miniPlayerAlwaysOnTop: DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP,
		autoRefreshIntervalMinutes: DEFAULT_AUTO_REFRESH_INTERVAL_MINUTES,
		colorScheme: DEFAULT_COLOR_SCHEME,
		accentColor: DEFAULT_ACCENT_COLOR,
		themeName: DEFAULT_THEME_NAME,
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

	$effect(() => {
		const name = pending.themeName;
		if (!initialized) return;
		if (name) {
			loadTheme(name)
				.then((css) => applyThemeCss(css))
				.catch(() => applyThemeCss(null));
		} else {
			applyThemeCss(null);
		}
	});

	onMount(async () => {
		isDesktop = isTauriRuntime();

		if (!isDesktop) {
			isLoading = false;
			initialized = true;
			return;
		}

		isLoadingThemes = true;
		try {
			const [settings, discovered] = await Promise.all([loadAppSettings(), discoverThemes()]);
			Object.assign(pending, settings);
			themes = discovered;
		} catch (error) {
			errorMessage = `Failed to load settings. ${getErrorMessage(error)}`;
		} finally {
			isLoading = false;
			isLoadingThemes = false;
			initialized = true;
		}
	});

	let isFormValid = $derived(
		allEntries.every((entry) => {
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

		for (const entry of allEntries) {
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

		<form class="mt-8 flex flex-col gap-6" onsubmit={handleSave}>
			{#each APP_SETTINGS as section (section.title)}
				<div class="rounded-2xl border border-border bg-surface p-6 shadow-sm">
					<h3 class="text-lg font-semibold text-fg">{section.title}</h3>
					{#if section.description}
						<p class="mt-1 text-sm text-fg-muted">{section.description}</p>
					{/if}
					<div class="mt-5 flex flex-col divide-y divide-border">
						{#if section.title === 'Appearance'}
							<!-- Theme selector -->
							<div class="py-5 first:pt-0 last:pb-0">
								<div class="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
									<div class="max-w-sm">
										<h4 class="text-sm font-medium text-fg">Theme</h4>
										<p class="mt-1 text-sm text-fg-muted">
											Choose a custom CSS theme. Place `.css` files in the themes directory to add
											more options.
										</p>
									</div>
									<div class="flex w-full max-w-xs flex-col gap-2">
										<select
											class="w-full rounded-xl border border-border bg-surface text-sm text-fg transition focus:border-border-hover focus:ring-2 focus:ring-ring disabled:opacity-60"
											disabled={isLoading || isSaving || isLoadingThemes}
											value={pending.themeName ?? ''}
											onchange={(e) => {
												const value = e.currentTarget.value;
												pending.themeName = value === '' ? null : value;
											}}
										>
											<option value="">Default</option>
											{#each themes as theme (theme.filename)}
												<option value={theme.filename}>{theme.name}</option>
											{/each}
										</select>
										{#if isLoadingThemes}
											<p class="text-sm text-fg-muted">Loading themes…</p>
										{/if}
									</div>
								</div>
							</div>
						{/if}
						{#each section.entries as entry (entry.key)}
							<div class="py-5 first:pt-0 last:pb-0">
								<SettingRow {entry} {pending} disabled={isLoading || isSaving} {isDesktop} />
							</div>
						{/each}
					</div>
				</div>
			{/each}

			<div class="rounded-2xl border border-border bg-surface p-6 shadow-sm">
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

			<div class="flex flex-wrap items-center justify-end gap-3">
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
					<p class="text-sm text-success">{successMessage}</p>
				{:else if errorMessage}
					<p class="text-sm text-error">{errorMessage}</p>
				{/if}
			</div>
		</form>
	</div>
</section>
