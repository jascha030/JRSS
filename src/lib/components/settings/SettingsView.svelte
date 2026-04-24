<script lang="ts">
	import { onMount } from 'svelte';

	import { loadAppSettings, saveAppSettings } from '$lib/services/feedService';
	import { isTauriRuntime } from '$lib/services/tauriClient';
	import { DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES } from '$lib/types/rss';

	const BYTES_PER_GB = 1024 * 1024 * 1024;

	let isDesktopOnly = $state(false);
	let isLoading = $state(true);
	let isSaving = $state(false);
	let maxAudioCacheSizeGb = $state<number | undefined>(undefined);
	let errorMessage = $state('');
	let successMessage = $state('');

	let isValueValid = $derived(
		typeof maxAudioCacheSizeGb === 'number' &&
			Number.isFinite(maxAudioCacheSizeGb) &&
			maxAudioCacheSizeGb > 0
	);

	function bytesToGb(bytes: number): number {
		return Number((bytes / BYTES_PER_GB).toFixed(2));
	}

	function gbToBytes(gigabytes: number): number {
		return Math.round(gigabytes * BYTES_PER_GB);
	}

	function resetMessages(): void {
		errorMessage = '';
		successMessage = '';
	}

	function getErrorMessage(error: unknown): string {
		return error instanceof Error ? error.message : 'Something went wrong.';
	}

	onMount(async () => {
		if (!isTauriRuntime()) {
			isDesktopOnly = true;
			maxAudioCacheSizeGb = bytesToGb(DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES);
			isLoading = false;
			return;
		}

		try {
			const settings = await loadAppSettings();
			maxAudioCacheSizeGb = bytesToGb(settings.maxAudioCacheSizeBytes);
		} catch (error) {
			maxAudioCacheSizeGb = bytesToGb(DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES);
			errorMessage = `Failed to load settings. ${getErrorMessage(error)}`;
		} finally {
			isLoading = false;
		}
	});

	async function handleSave(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		resetMessages();

		if (isDesktopOnly) {
			errorMessage = 'Settings are only available in the desktop app.';
			return;
		}

		if (!isValueValid || maxAudioCacheSizeGb === undefined) {
			errorMessage = 'Enter a value greater than 0 GB.';
			return;
		}

		isSaving = true;

		try {
			const settings = await saveAppSettings({
				maxAudioCacheSizeBytes: gbToBytes(maxAudioCacheSizeGb)
			});

			maxAudioCacheSizeGb = bytesToGb(settings.maxAudioCacheSizeBytes);
			successMessage = 'Settings saved.';
		} catch (error) {
			errorMessage = `Failed to save settings. ${getErrorMessage(error)}`;
		} finally {
			isSaving = false;
		}
	}
</script>

<section class="flex-2 overflow-y-auto px-6 py-8 lg:px-8">
	<div class="max-w-4xl">
		<p class="text-sm font-medium tracking-[0.18em] text-fg-muted uppercase">Settings</p>

		<h1 class="mt-3 text-2xl font-semibold tracking-tight text-fg">Audio cache</h1>
		<p class="mt-2 text-sm text-fg-muted">
			Configure how much downloaded audio JRSS can keep locally.
		</p>

		<form class="mt-8" onsubmit={handleSave}>
			<div class="rounded-2xl border border-border bg-surface p-6 shadow-sm">
				<div class="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
					<div class="max-w-2xl">
						<h2 class="text-base font-semibold text-fg">Max audio cache size</h2>
						<p class="mt-2 text-sm text-fg-muted">
							Set the maximum amount of disk space used for cached audio files.
						</p>

						{#if isDesktopOnly}
							<p
								class="mt-4 rounded-xl border border-border bg-surface-hover px-4 py-3 text-sm text-fg-muted"
							>
								Desktop only. Run JRSS through Tauri to load and save this setting.
							</p>
						{/if}
					</div>

					<div class="w-full max-w-xs">
						<label class="block text-sm font-medium text-fg" for="max-audio-cache-size">
							Value in GB
						</label>
						<input
							id="max-audio-cache-size"
							type="number"
							min="0.1"
							step="0.1"
							bind:value={maxAudioCacheSizeGb}
							disabled={isLoading || isSaving || isDesktopOnly}
							oninput={resetMessages}
							class="mt-1.5 w-full rounded-xl border border-border bg-surface px-4 py-2.5 text-sm text-fg transition outline-none placeholder:text-fg-muted focus:border-border-hover focus:ring-2 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-60"
						/>
						<p class="mt-2 text-xs text-fg-muted">Stored internally in bytes.</p>
					</div>
				</div>

				<div class="mt-6 flex flex-wrap items-center gap-3">
					<button
						type="submit"
						class="btn rounded-xl preset-filled"
						disabled={isLoading || isSaving || isDesktopOnly}
					>
						{isSaving ? 'Saving…' : 'Save settings'}
					</button>

					{#if isLoading}
						<p class="text-sm text-fg-muted">Loading settings…</p>
					{:else if !isValueValid && maxAudioCacheSizeGb !== undefined}
						<p class="text-error text-sm">Enter a value greater than 0 GB.</p>
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
