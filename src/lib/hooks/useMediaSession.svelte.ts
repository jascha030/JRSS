import type { MediaListItem } from '$lib/types/rss';

type SkipFn = (deltaSeconds: number) => void;

export function useMediaSession(getItem: () => MediaListItem | null, skip: SkipFn) {
	$effect(() => {
		const item = getItem();
		if (!item || !('mediaSession' in navigator)) {
			return;
		}

		const back = () => skip(-15);
		const forward = () => skip(15);

		navigator.mediaSession.setActionHandler('previoustrack', back);
		navigator.mediaSession.setActionHandler('nexttrack', forward);
		navigator.mediaSession.setActionHandler('seekbackward', back);
		navigator.mediaSession.setActionHandler('seekforward', forward);

		return () => {
			navigator.mediaSession.setActionHandler('previoustrack', null);
			navigator.mediaSession.setActionHandler('nexttrack', null);
			navigator.mediaSession.setActionHandler('seekbackward', null);
			navigator.mediaSession.setActionHandler('seekforward', null);
		};
	});
}
