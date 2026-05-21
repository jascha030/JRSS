import { tick } from 'svelte';

const START_DELAY_MS = 1200;
const END_PAUSE_MS = 900;
const RESET_PAUSE_MS = 3000;
const PIXELS_PER_SECOND = 28;
const LOOP_TICK_MS = 16;

type Options = {
	getViewportElement: () => HTMLDivElement | null;
	getTextElement: () => HTMLSpanElement | null;
	getText: () => string | null;
};

export function useOverflowMarquee(options: Options) {
	let isOverflowing = $state(false);
	let overflowDistance = $state(0);
	let offset = $state(0);
	let reducedMotion = $state(false);
	let lastMeasuredText = $state<string | null>(null);

	let loopToken = 0;
	let paused = false;
	let currentAnimationFrame: number | null = null;

	function cancelCurrentAnimationFrame() {
		if (currentAnimationFrame !== null) {
			cancelAnimationFrame(currentAnimationFrame);
			currentAnimationFrame = null;
		}
	}

	function sleep(ms: number) {
		return new Promise<void>((resolve) => {
			window.setTimeout(resolve, ms);
		});
	}

	async function waitWithPause(ms: number, token: number) {
		let remaining = ms;

		while (remaining > 0) {
			if (token !== loopToken || !isOverflowing || reducedMotion) {
				return false;
			}

			if (paused) {
				await sleep(LOOP_TICK_MS);
				continue;
			}

			const slice = Math.min(LOOP_TICK_MS, remaining);
			await sleep(slice);
			remaining -= slice;
		}

		return token === loopToken && isOverflowing && !reducedMotion;
	}

	async function animateOffset(to: number, durationMs: number, token: number) {
		cancelCurrentAnimationFrame();

		if (durationMs <= 0) {
			offset = to;
			return token === loopToken;
		}

		const from = offset;
		const delta = to - from;

		let animationStart: number | null = null;
		let pausedAt: number | null = null;
		let pausedTotal = 0;

		return await new Promise<boolean>((resolve) => {
			const step = (now: number) => {
				if (token !== loopToken || !isOverflowing || reducedMotion) {
					currentAnimationFrame = null;
					resolve(false);
					return;
				}

				if (animationStart === null) {
					animationStart = now;
				}

				if (paused) {
					if (pausedAt === null) {
						pausedAt = now;
					}

					currentAnimationFrame = requestAnimationFrame(step);
					return;
				}

				if (pausedAt !== null) {
					pausedTotal += now - pausedAt;
					pausedAt = null;
				}

				const elapsed = now - animationStart - pausedTotal;
				const progress = Math.max(0, Math.min(1, elapsed / durationMs));

				offset = from + delta * progress;

				if (progress >= 1) {
					offset = to;
					currentAnimationFrame = null;
					resolve(true);
					return;
				}

				currentAnimationFrame = requestAnimationFrame(step);
			};

			currentAnimationFrame = requestAnimationFrame(step);
		});
	}

	function stopLoop() {
		loopToken += 1;
		cancelCurrentAnimationFrame();
		offset = 0;
	}

	async function measureOverflow() {
		await tick();

		const viewport = options.getViewportElement();
		const text = options.getTextElement();
		const value = options.getText();

		if (!viewport || !text || !value) {
			isOverflowing = false;
			overflowDistance = 0;
			stopLoop();
			return;
		}

		const nextOverflowDistance = Math.max(0, text.scrollWidth - viewport.clientWidth);

		overflowDistance = nextOverflowDistance;
		isOverflowing = nextOverflowDistance > 1;

		if (!isOverflowing || reducedMotion) {
			stopLoop();
			return;
		}

		void startLoop();
	}

	async function startLoop() {
		const token = ++loopToken;
		cancelCurrentAnimationFrame();
		offset = 0;

		const scrollDurationMs = Math.max(3000, (overflowDistance / PIXELS_PER_SECOND) * 1000);

		while (token === loopToken && isOverflowing && !reducedMotion) {
			offset = 0;

			if (!(await waitWithPause(START_DELAY_MS, token))) {
				return;
			}

			if (!(await animateOffset(overflowDistance, scrollDurationMs, token))) {
				return;
			}

			if (!(await waitWithPause(END_PAUSE_MS, token))) {
				return;
			}

			offset = 0;

			if (!(await waitWithPause(RESET_PAUSE_MS, token))) {
				return;
			}
		}
	}

	function pause() {
		paused = true;
	}

	function resume() {
		paused = false;
	}

	$effect(() => {
		const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');

		const apply = () => {
			reducedMotion = mediaQuery.matches;
			void measureOverflow();
		};

		apply();
		mediaQuery.addEventListener('change', apply);

		return () => mediaQuery.removeEventListener('change', apply);
	});

	$effect(() => {
		const nextText = options.getText();

		if (nextText === lastMeasuredText) {
			return;
		}

		lastMeasuredText = nextText;
		void measureOverflow();
	});

	$effect(() => {
		const viewport = options.getViewportElement();
		const text = options.getTextElement();

		if (!viewport) {
			return;
		}

		const resizeObserver = new ResizeObserver(() => {
			void measureOverflow();
		});

		resizeObserver.observe(viewport);

		if (text) {
			resizeObserver.observe(text);
		}

		return () => resizeObserver.disconnect();
	});

	$effect(() => {
		return () => {
			stopLoop();
		};
	});

	return {
		get isOverflowing() {
			return isOverflowing;
		},
		get offset() {
			return offset;
		},
		get reducedMotion() {
			return reducedMotion;
		},
		pause,
		resume
	};
}
