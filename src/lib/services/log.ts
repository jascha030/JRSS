import { debug, error, info, warn } from '@tauri-apps/plugin-log';
import { isTauriRuntime } from './tauri';

export const log = {
	debug: (message: string) => {
		if (isTauriRuntime()) {
			void debug(message);
		} else {
			console.debug(message);
		}
	},
	error: (message: string) => {
		if (isTauriRuntime()) {
			void error(message);
		} else {
			console.error(message);
		}
	},
	info: (message: string) => {
		if (isTauriRuntime()) {
			void info(message);
		} else {
			console.info(message);
		}
	},
	warn: (message: string) => {
		if (isTauriRuntime()) {
			void warn(message);
		} else {
			console.warn(message);
		}
	}
};
