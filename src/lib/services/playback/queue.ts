import { invokeCommand } from '$lib/services/tauri';
import type { BackendQueueState } from '$lib/types/playback';

export interface QueuedItem {
	itemId: string;
	url: string;
	title: string;
	durationSeconds: number;
}

export async function audioPlayWithQueue(
	item: QueuedItem,
	manualQueue: QueuedItem[],
	autoQueue: QueuedItem[],
	startPositionSeconds: number
): Promise<void> {
	await invokeCommand('audio_play_with_queue', {
		item,
		manualQueue,
		autoQueue,
		startPositionSeconds
	});
}

export async function audioQueueEnqueue(item: QueuedItem): Promise<void> {
	await invokeCommand('audio_queue_enqueue', { item });
}

export async function audioQueuePlayNext(item: QueuedItem): Promise<void> {
	await invokeCommand('audio_queue_play_next', { item });
}

export async function audioQueueRemove(itemId: string): Promise<void> {
	await invokeCommand('audio_queue_remove', { itemId });
}

export async function audioQueueMoveUp(itemId: string): Promise<void> {
	await invokeCommand('audio_queue_move_up', { itemId });
}

export async function audioQueueMoveDown(itemId: string): Promise<void> {
	await invokeCommand('audio_queue_move_down', { itemId });
}

export async function audioQueueNext(): Promise<void> {
	await invokeCommand('audio_queue_next');
}

export async function audioQueuePrev(): Promise<void> {
	await invokeCommand('audio_queue_prev');
}

export async function audioQueueClear(): Promise<void> {
	await invokeCommand('audio_queue_clear');
}

export async function audioQueueClearHistory(): Promise<void> {
	await invokeCommand('audio_queue_clear_history');
}

export async function audioQueueGetState(): Promise<BackendQueueState> {
	return invokeCommand<BackendQueueState>('audio_queue_get_state');
}

export async function audioQueueSet(items: QueuedItem[]): Promise<void> {
	await invokeCommand('audio_queue_set', { items });
}
