import { WebviewWindow, getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

export const MAIN_WINDOW_LABEL = 'main';
export const MINI_WINDOW_LABEL = 'mini-player';
export const MINI_WINDOW_URL = '/?window=mini';

async function ensureMiniPlayerWindow(): Promise<WebviewWindow> {
	const existing = await WebviewWindow.getByLabel(MINI_WINDOW_LABEL);
	if (existing) return existing;

	return new Promise((resolve, reject) => {
		let settled = false;

		const miniWindow = new WebviewWindow(MINI_WINDOW_LABEL, {
			url: MINI_WINDOW_URL,
			center: true,
			decorations: true,
			height: 420,
			maxHeight: 520,
			maxWidth: 400,
			minHeight: 360,
			minWidth: 280,
			resizable: true,
			title: 'JRSS Mini Player',
			titleBarStyle: 'overlay',
			// trafficLightPosition: {
			//     x: 16,
			//     y: 18
			// },
			visible: false,
			width: 320
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
