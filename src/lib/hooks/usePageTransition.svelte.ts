import { onNavigate, afterNavigate } from '$app/navigation';

const FADE_OUT_MS = 180;
const FADE_IN_MS = 200;
const MIN_SPINNER_MS = 250;

export type PageTransitionPhase = 'idle' | 'fading-out' | 'spinner' | 'fading-in';

export function usePageTransition() {
	let phase = $state<PageTransitionPhase>('idle');
	let transitionStartTime = 0;

	onNavigate((navigation) => {
		if (!navigation.to) return;

		transitionStartTime = performance.now();
		phase = 'fading-out';

		return new Promise<void>((resolve) => {
			window.setTimeout(() => {
				phase = 'spinner';
				resolve();
			}, FADE_OUT_MS);
		});
	});

	afterNavigate(() => {
		if (phase === 'idle') return;

		const elapsed = performance.now() - transitionStartTime;
		const spinnerEnd = FADE_OUT_MS + MIN_SPINNER_MS;

		if (elapsed < spinnerEnd) {
			window.setTimeout(() => {
				phase = 'fading-in';
				window.setTimeout(() => {
					phase = 'idle';
				}, FADE_IN_MS);
			}, spinnerEnd - elapsed);
		} else {
			phase = 'fading-in';
			window.setTimeout(() => {
				phase = 'idle';
			}, FADE_IN_MS);
		}
	});

	return {
		get phase() {
			return phase;
		}
	};
}
