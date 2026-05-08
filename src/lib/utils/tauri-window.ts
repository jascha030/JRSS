import { WebviewWindow, getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { loadAppSettings } from '$lib/services/settings';
import { invokeCommand } from '$lib/services/tauri';
import { DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP } from '$lib/types/settings';

export const MAIN_WINDOW_LABEL = 'main';
export const MINI_WINDOW_LABEL = 'mini-player';
export const MINI_WINDOW_URL = '/?window=mini';

async function loadMiniPlayerAlwaysOnTop(): Promise<boolean> {
	try {
		const settings = await loadAppSettings();
		return settings.miniPlayerAlwaysOnTop;
	} catch {
		return DEFAULT_MINI_PLAYER_ALWAYS_ON_TOP;
	}
}

async function ensureMiniPlayerWindow(): Promise<WebviewWindow> {
	const miniPlayerAlwaysOnTop = await loadMiniPlayerAlwaysOnTop();
	const existing = await WebviewWindow.getByLabel(MINI_WINDOW_LABEL);
	if (existing) {
		await existing.setAlwaysOnTop(miniPlayerAlwaysOnTop);
		return existing;
	}

	return new Promise((resolve, reject) => {
		let settled = false;

		const miniWindow = new WebviewWindow(MINI_WINDOW_LABEL, {
			url: MINI_WINDOW_URL,
			center: true,
			decorations: true,
			height: 400,
			maximizable: false,
			maxHeight: 800,
			maxWidth: 800,
			minHeight: 340,
			minWidth: 340,
			alwaysOnTop: miniPlayerAlwaysOnTop,
			width: 400,
			resizable: true,
			title: 'JRSS Mini Player',
			titleBarStyle: 'overlay',
			visible: false
		});

		const timeoutId = window.setTimeout(() => {
			if (settled) return;
			settled = true;
			reject(new Error('Timed out creating mini player window.'));
		}, 4000);

		miniWindow.once('tauri://created', () => {
			if (settled) return;
			settled = true;
			window.clearTimeout(timeoutId);
			resolve(miniWindow);
		});

		miniWindow.once('tauri://error', (event) => {
			if (settled) return;
			settled = true;
			window.clearTimeout(timeoutId);
			reject(
				new Error(
					typeof event.payload === 'string' ? event.payload : 'Unable to create mini player window.'
				)
			);
		});
	});
}

export async function openMiniPlayer(): Promise<void> {
	const miniWindow = await ensureMiniPlayerWindow();
	await invokeCommand('set_window_content_aspect_ratio', {
		label: MINI_WINDOW_LABEL,
		width: 1,
		height: 1
	});
	await miniWindow.show();

	const mainWindow = getCurrentWebviewWindow();
	await mainWindow.hide();
}

export async function restoreMainWindow(): Promise<void> {
	const mainWindow = await WebviewWindow.getByLabel(MAIN_WINDOW_LABEL);
	if (!mainWindow) {
		throw new Error('Main window not found.');
	}

	await mainWindow.show();

	const miniWindow = await WebviewWindow.getByLabel(MINI_WINDOW_LABEL);
	if (miniWindow) {
		await miniWindow.destroy();
	}
}

export { ensureMiniPlayerWindow };
