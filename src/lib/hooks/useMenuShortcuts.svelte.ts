import { onMount } from 'svelte';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

type MenuEventHandler = {
	event: string;
	handler: () => void;
};

export function useMenuShortcuts(handlers: MenuEventHandler[]) {
	onMount(() => {
		const unlisteners: UnlistenFn[] = [];

		const setup = async () => {
			for (const { event, handler } of handlers) {
				const unlisten = await listen(event, handler);
				unlisteners.push(unlisten);
			}
		};

		void setup();

		return () => {
			for (const unlisten of unlisteners) {
				unlisten();
			}
		};
	});
}
