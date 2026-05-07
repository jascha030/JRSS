import type { MediaListItem } from '$lib/types/rss';

type SkipFn = (deltaSeconds: number) => void;
type EpisodeFn = () => void;

export function useMediaSession(
	getItem: () => MediaListItem | null,
	skip: SkipFn,
	onPreviousEpisode?: EpisodeFn,
	onNextEpisode?: EpisodeFn
) {
	$effect(() => {
		const item = getItem();
		if (!item || !('mediaSession' in navigator)) {
			return;
		}

		const seekBack = () => skip(-15);
		const seekForward = () => skip(15);
		const prevTrack = onPreviousEpisode ? onPreviousEpisode : seekBack;
		const nextTrack = onNextEpisode ? onNextEpisode : seekForward;

		navigator.mediaSession.setActionHandler('previoustrack', prevTrack);
		navigator.mediaSession.setActionHandler('nexttrack', nextTrack);
		navigator.mediaSession.setActionHandler('seekbackward', seekBack);
		navigator.mediaSession.setActionHandler('seekforward', seekForward);

		return () => {
			navigator.mediaSession.setActionHandler('previoustrack', null);
			navigator.mediaSession.setActionHandler('nexttrack', null);
			navigator.mediaSession.setActionHandler('seekbackward', null);
			navigator.mediaSession.setActionHandler('seekforward', null);
		};
	});
}
