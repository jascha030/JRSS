import { invokeCommand, isTauriRuntime } from '$lib/services/tauri';
import type { AppSettings, ThemeInfo } from '$lib/types/settings';
import {
	DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP,
	DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES,
	DEFAULT_AUTO_REFRESH_INTERVAL_MINUTES,
	DEFAULT_COLOR_SCHEME,
	DEFAULT_ACCENT_COLOR,
	DEFAULT_THEME_NAME,
	DEFAULT_SKIP_FORWARD_SECONDS,
	DEFAULT_SKIP_BACKWARD_SECONDS
} from '$lib/types/settings';

export async function loadAppSettings(): Promise<AppSettings> {
	if (!isTauriRuntime()) {
		return {
			maxAudioCacheSizeBytes: DEFAULT_MAX_AUDIO_CACHE_SIZE_BYTES,
			miniPlayerAlwaysOnTop: DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP,
			autoRefreshIntervalMinutes: DEFAULT_AUTO_REFRESH_INTERVAL_MINUTES,
			colorScheme: DEFAULT_COLOR_SCHEME,
			accentColor: DEFAULT_ACCENT_COLOR,
			themeName: DEFAULT_THEME_NAME,
			skipForwardSeconds: DEFAULT_SKIP_FORWARD_SECONDS,
			skipBackwardSeconds: DEFAULT_SKIP_BACKWARD_SECONDS
		};
	}

	return invokeCommand<AppSettings>('load_app_settings');
}

export async function saveAppSettings(settings: AppSettings): Promise<AppSettings> {
	if (!isTauriRuntime()) {
		return settings;
	}

	return invokeCommand<AppSettings>('save_app_settings', { settings });
}

export async function clearAudioCache(): Promise<void> {
	await invokeCommand('clear_audio_cache');
}

export async function discoverThemes(): Promise<ThemeInfo[]> {
	if (!isTauriRuntime()) {
		return [];
	}
	return invokeCommand<ThemeInfo[]>('discover_themes');
}

export async function loadTheme(filename: string): Promise<string> {
	if (!isTauriRuntime()) {
		return '';
	}
	return invokeCommand<string>('load_theme', { filename });
}
