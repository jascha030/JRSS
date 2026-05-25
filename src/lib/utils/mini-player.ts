import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { openMiniPlayer, MINI_WINDOW_LABEL } from './tauri-window';
import { toast } from 'svelte-sonner';

export async function popOutMiniPlayer(): Promise<void> {
	try {
		const miniWindow = await WebviewWindow.getByLabel(MINI_WINDOW_LABEL);
		if (miniWindow && (await miniWindow.isVisible())) {
			return;
		}

		await openMiniPlayer();
	} catch (error: unknown) {
		const message = error instanceof Error ? error.message : 'Unable to open mini player.';
		if (message.includes('already in progress')) {
			return;
		}
		toast.error(message);
	}
}
