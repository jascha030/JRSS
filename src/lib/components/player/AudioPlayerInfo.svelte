<script lang="ts">
	/**
	 * Audio player info component with album art and scrolling title.
	 * Simplified for the compact bottom bar player only.
	 */
	import type { MediaListItem } from '$lib/types/rss';
	import { tick } from 'svelte';
	import { openAudioContextMenu } from '$lib/utils/tauri-menu';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		onNavigate?: () => void;
		onShowCover?: () => void;
		class?: string;
	};

	let {
		item,
		imageUrl,
		onNavigate,
		onShowCover: onShowCover = undefined,
		class: className = ''
	}: Props = $props();

	const TITLE_START_DELAY_MS = 1200;
	const TITLE_END_PAUSE_MS = 900;
	const TITLE_RESET_PAUSE_MS = 3000;
	const TITLE_PIXELS_PER_SECOND = 28;
	const TITLE_LOOP_TICK_MS = 16;

	let titleViewportEl: HTMLDivElement | null = $state(null);
	let titleTextEl: HTMLSpanElement | null = $state(null);

	let titleIsOverflowing = $state(false);
	let titleOverflowDistance = $state(0);
	let titleOffset = $state(0);
	let titleReducedMotion = $state(false);
	let lastMeasuredTitle = $state<string | null>(null);

	let titleLoopToken = 0;
	let titlePaused = false;
	let titleCurrentAnimationFrame: number | null = null;

	function cancelTitleAnimationFrame() {
		if (titleCurrentAnimationFrame !== null) {
			cancelAnimationFrame(titleCurrentAnimationFrame);
			titleCurrentAnimationFrame = null;
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
			if (token !== titleLoopToken) return false;
			if (!titleIsOverflowing) return false;
			if (titleReducedMotion) return false;

			if (titlePaused) {
				await sleep(TITLE_LOOP_TICK_MS);
				continue;
			}

			const slice = Math.min(TITLE_LOOP_TICK_MS, remaining);
			await sleep(slice);
			remaining -= slice;
		}

		return token === titleLoopToken && titleIsOverflowing && !titleReducedMotion;
	}

	async function animateTitleOffset(to: number, durationMs: number, token: number) {
		cancelTitleAnimationFrame();

		if (durationMs <= 0) {
			titleOffset = to;
			return token === titleLoopToken;
		}

		const from = titleOffset;
		const delta = to - from;

		let animationStart: number | null = null;
		let pausedAt: number | null = null;
		let pausedTotal = 0;

		return await new Promise<boolean>((resolve) => {
			const step = (now: number) => {
				if (token !== titleLoopToken || !titleIsOverflowing || titleReducedMotion) {
					titleCurrentAnimationFrame = null;
					resolve(false);
					return;
				}

				if (animationStart === null) {
					animationStart = now;
				}

				if (titlePaused) {
					if (pausedAt === null) {
						pausedAt = now;
					}

					titleCurrentAnimationFrame = requestAnimationFrame(step);
					return;
				}

				if (pausedAt !== null) {
					pausedTotal += now - pausedAt;
					pausedAt = null;
				}

				const elapsed = now - animationStart - pausedTotal;
				const progress = Math.max(0, Math.min(1, elapsed / durationMs));

				titleOffset = from + delta * progress;

				if (progress >= 1) {
					titleOffset = to;
					titleCurrentAnimationFrame = null;
					resolve(true);
					return;
				}

				titleCurrentAnimationFrame = requestAnimationFrame(step);
			};

			titleCurrentAnimationFrame = requestAnimationFrame(step);
		});
	}

	function stopTitleLoop() {
		titleLoopToken += 1;
		cancelTitleAnimationFrame();
		titleOffset = 0;
	}

	async function measureTitleOverflow() {
		await tick();

		if (!titleViewportEl || !titleTextEl || !item?.title) {
			titleIsOverflowing = false;
			titleOverflowDistance = 0;
			stopTitleLoop();
			return;
		}

		const viewportWidth = titleViewportEl.clientWidth;
		const textWidth = titleTextEl.scrollWidth;
		const overflow = Math.max(0, textWidth - viewportWidth);

		titleOverflowDistance = overflow;
		titleIsOverflowing = overflow > 1;

		if (!titleIsOverflowing || titleReducedMotion) {
			stopTitleLoop();
			return;
		}

		void startTitleLoop();
	}

	async function startTitleLoop() {
		const token = ++titleLoopToken;
		cancelTitleAnimationFrame();
		titleOffset = 0;

		const scrollDurationMs = Math.max(
			3000,
			(titleOverflowDistance / TITLE_PIXELS_PER_SECOND) * 1000
		);

		while (token === titleLoopToken && titleIsOverflowing && !titleReducedMotion) {
			titleOffset = 0;

			{
				const ok = await waitWithPause(TITLE_START_DELAY_MS, token);
				if (!ok) return;
			}

			{
				const ok = await animateTitleOffset(titleOverflowDistance, scrollDurationMs, token);
				if (!ok) return;
			}

			{
				const ok = await waitWithPause(TITLE_END_PAUSE_MS, token);
				if (!ok) return;
			}

			titleOffset = 0;

			{
				const ok = await waitWithPause(TITLE_RESET_PAUSE_MS, token);
				if (!ok) return;
			}
		}
	}

	function pauseTitleMarquee() {
		titlePaused = true;
	}

	function resumeTitleMarquee() {
		titlePaused = false;
	}

	$effect(() => {
		if (typeof window === 'undefined') {
			return;
		}

		const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');

		const apply = () => {
			titleReducedMotion = mediaQuery.matches;
			void measureTitleOverflow();
		};

		apply();
		mediaQuery.addEventListener('change', apply);

		return () => mediaQuery.removeEventListener('change', apply);
	});

	$effect(() => {
		const nextTitle = item?.title ?? null;

		if (nextTitle === lastMeasuredTitle) {
			return;
		}

		lastMeasuredTitle = nextTitle;
		void measureTitleOverflow();
	});

	$effect(() => {
		const viewport = titleViewportEl;
		const text = titleTextEl;

		if (!viewport) {
			return;
		}

		const resizeObserver = new ResizeObserver(() => {
			void measureTitleOverflow();
		});

		resizeObserver.observe(viewport);

		if (text) {
			resizeObserver.observe(text);
		}

		return () => resizeObserver.disconnect();
	});

	$effect(() => {
		return () => {
			stopTitleLoop();
		};
	});
</script>

{#if item}
	<div class="flex min-w-0 items-center gap-3 {className}">
		{#if imageUrl}
			<button
				class="shrink-0"
				type="button"
				onclick={onShowCover}
				oncontextmenu={(event) => item && openAudioContextMenu(event, item)}
			>
				<img src={imageUrl} alt="" class="size-12 rounded-lg object-cover shadow-sm select-none" />
			</button>
		{/if}

		<div class="min-w-0">
			<p class="text-xs font-medium tracking-[0.18em] text-fg-muted uppercase">Now playing</p>

			<button
				class="mt-1 block w-full text-left text-sm font-semibold text-fg transition-colors select-none hover:text-accent focus-visible:text-accent"
				type="button"
				onclick={onNavigate}
				oncontextmenu={(event) => item && openAudioContextMenu(event, item)}
				onmouseenter={pauseTitleMarquee}
				onmouseleave={resumeTitleMarquee}
				onfocus={pauseTitleMarquee}
				onblur={resumeTitleMarquee}
			>
				<div bind:this={titleViewportEl} class="overflow-hidden">
					<span
						bind:this={titleTextEl}
						class:truncate={!titleIsOverflowing || titleReducedMotion}
						class="block whitespace-nowrap will-change-transform"
						style={`transform: translateX(-${titleOffset}px);`}
					>
						{item.title}
					</span>
				</div>
			</button>
		</div>
	</div>
{/if}

<style>
	@media (prefers-reduced-motion: reduce) {
		[style*='will-change: transform'] {
			transform: translateX(0) !important;
		}
	}
</style>
