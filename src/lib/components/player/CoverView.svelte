<script lang="ts">
	import type { Feed, MediaListItem, PlaybackState } from '$lib/types/rss';
	import type { Snippet } from 'svelte';
	import { requestSeekTo, requestTogglePlayback } from '$lib/stores/app.svelte';
	import { extractCoverPalette } from '$lib/services/feedService';
	import Icon from '@iconify/svelte';
	import AudioPlayerControls from './AudioPlayerControls.svelte';
	import AudioPlayerInfo from './AudioPlayerInfo.svelte';
	import AudioSeekBar from './AudioSeekBar.svelte';
	import AudioPlayerVolume from './AudioPlayerVolume.svelte';
	import QueueList from './QueueList.svelte';

	type Props = {
		item: MediaListItem | null;
		imageUrl?: string;
		playbackState: PlaybackState | null;
		onNavigateToItem?: () => void;
		onClose?: () => void;
		controls?: Snippet;
		class?: string;
		queueItems?: MediaListItem[];
		manualQueueLength?: number;
		feeds?: Feed[];
		onRemoveQueueItem?: (itemId: string) => void;
		onMoveQueueItemUp?: (itemId: string) => void;
		onMoveQueueItemDown?: (itemId: string) => void;
		onClearQueue?: () => void;
	};

	type Rgb = {
		r: number;
		g: number;
		b: number;
	};

	type CoverTheme = {
		bg1: string;
		bg2: string;
		bg3: string;
		glow1: string;
		glow2: string;
		glow3: string;
		fg: string;
		fgMuted: string;
		fgSubtle: string;
		accent: string;
		accentContrast: string;
		panelBg: string;
		panelBorder: string;
		buttonBg: string;
		buttonBgHover: string;
	};

	const SKIP_SECONDS = 15;

	const FALLBACK_THEME: CoverTheme = {
		bg1: '#0f172a',
		bg2: '#1e293b',
		bg3: '#334155',
		glow1: 'rgba(59, 130, 246, 0.28)',
		glow2: 'rgba(168, 85, 247, 0.22)',
		glow3: 'rgba(236, 72, 153, 0.20)',
		fg: '#ffffff',
		fgMuted: 'rgba(255, 255, 255, 0.78)',
		fgSubtle: 'rgba(255, 255, 255, 0.56)',
		accent: '#ffffff',
		accentContrast: '#0f172a',
		panelBg: 'rgba(255, 255, 255, 0.08)',
		panelBorder: 'rgba(255, 255, 255, 0.16)',
		buttonBg: 'rgba(255, 255, 255, 0.10)',
		buttonBgHover: 'rgba(255, 255, 255, 0.16)'
	};

	let {
		item,
		imageUrl,
		playbackState,
		onNavigateToItem,
		onClose,
		controls,
		class: className = '',
		queueItems = [],
		manualQueueLength = 0,
		feeds = [],
		onRemoveQueueItem,
		onMoveQueueItemUp,
		onMoveQueueItemDown,
		onClearQueue
	}: Props = $props();

	let coverTheme = $state<CoverTheme>(FALLBACK_THEME);
	let paletteRequestToken = 0;

	function clamp(value: number, min: number, max: number) {
		return Math.min(max, Math.max(min, value));
	}

	function hexToRgb(hex: string): Rgb {
		const normalized = hex.replace('#', '').trim();
		const full =
			normalized.length === 3
				? normalized
						.split('')
						.map((part) => part + part)
						.join('')
				: normalized;

		const int = Number.parseInt(full, 16);

		return {
			r: (int >> 16) & 255,
			g: (int >> 8) & 255,
			b: int & 255
		};
	}

	function rgbToHex({ r, g, b }: Rgb) {
		return `#${[r, g, b]
			.map((value) => clamp(Math.round(value), 0, 255).toString(16).padStart(2, '0'))
			.join('')}`;
	}

	function rgba({ r, g, b }: Rgb, alpha: number) {
		return `rgba(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)}, ${alpha})`;
	}

	function mix(a: Rgb, b: Rgb, amount: number): Rgb {
		const t = clamp(amount, 0, 1);
		return {
			r: a.r + (b.r - a.r) * t,
			g: a.g + (b.g - a.g) * t,
			b: a.b + (b.b - a.b) * t
		};
	}

	function average(colors: Rgb[]): Rgb {
		if (colors.length === 0) return { r: 255, g: 255, b: 255 };

		const total = colors.reduce(
			(acc, color) => {
				acc.r += color.r;
				acc.g += color.g;
				acc.b += color.b;
				return acc;
			},
			{ r: 0, g: 0, b: 0 }
		);

		return {
			r: total.r / colors.length,
			g: total.g / colors.length,
			b: total.b / colors.length
		};
	}

	function channelToLinear(value: number) {
		const s = value / 255;
		return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
	}

	function luminance(color: Rgb) {
		return (
			0.2126 * channelToLinear(color.r) +
			0.7152 * channelToLinear(color.g) +
			0.0722 * channelToLinear(color.b)
		);
	}

	function contrastRatio(a: Rgb, b: Rgb) {
		const l1 = luminance(a);
		const l2 = luminance(b);
		const lighter = Math.max(l1, l2);
		const darker = Math.min(l1, l2);
		return (lighter + 0.05) / (darker + 0.05);
	}

	function readableTextFor(color: Rgb) {
		const white = hexToRgb('#ffffff');
		const slate = hexToRgb('#0f172a');

		return contrastRatio(white, color) >= contrastRatio(slate, color) ? white : slate;
	}

	function buildThemeFromPalette(hexes: string[]): CoverTheme {
		const source = hexes.map(hexToRgb).slice(0, 3);

		while (source.length < 3) {
			source.push(source[source.length - 1] ?? hexToRgb('#334155'));
		}

		const slate950 = hexToRgb('#020617');
		const slate900 = hexToRgb('#0f172a');
		const slate800 = hexToRgb('#1e293b');
		const white = hexToRgb('#ffffff');

		/**
		 * Important:
		 * we intentionally darken the extracted palette.
		 * That gives us a stable "cover mode" where white-ish text remains readable,
		 * including children that still use explicit white classes.
		 */
		const bg1 = mix(source[0], slate950, 0.52);
		const bg2 = mix(source[1], slate900, 0.45);
		const bg3 = mix(source[2], slate800, 0.36);

		const averageBg = average([bg1, bg2, bg3]);

		/**
		 * In practice this will almost always resolve to white because the background
		 * is darkened, but we still compute it.
		 */
		const fgBase = readableTextFor(averageBg);
		const lightForeground = contrastRatio(white, averageBg) >= 4.5 || fgBase.r > 127;

		const fg = lightForeground ? white : hexToRgb('#0f172a');
		const accentBase = mix(source[0], white, lightForeground ? 0.08 : 0);
		const accentContrast = readableTextFor(accentBase);

		return {
			bg1: rgbToHex(bg1),
			bg2: rgbToHex(bg2),
			bg3: rgbToHex(bg3),
			glow1: rgba(source[0], 0.3),
			glow2: rgba(source[1], 0.24),
			glow3: rgba(source[2], 0.2),
			fg: rgbToHex(fg),
			fgMuted: rgba(fg, lightForeground ? 0.78 : 0.76),
			fgSubtle: rgba(fg, lightForeground ? 0.56 : 0.52),
			accent: rgbToHex(accentBase),
			accentContrast: rgbToHex(accentContrast),
			panelBg: lightForeground ? 'rgba(255, 255, 255, 0.08)' : 'rgba(15, 23, 42, 0.08)',
			panelBorder: lightForeground ? 'rgba(255, 255, 255, 0.16)' : 'rgba(15, 23, 42, 0.12)',
			buttonBg: lightForeground ? 'rgba(255, 255, 255, 0.10)' : 'rgba(15, 23, 42, 0.08)',
			buttonBgHover: lightForeground ? 'rgba(255, 255, 255, 0.16)' : 'rgba(15, 23, 42, 0.14)'
		};
	}

	async function updateCoverTheme(url?: string) {
		if (!url) {
			coverTheme = FALLBACK_THEME;
			return;
		}

		const token = ++paletteRequestToken;

		try {
			const palette = await extractCoverPalette(url);

			if (token !== paletteRequestToken) return;

			coverTheme = palette.length > 0 ? buildThemeFromPalette(palette) : FALLBACK_THEME;
		} catch (error) {
			if (token !== paletteRequestToken) return;
			console.error('Failed to extract cover palette:', error);
			coverTheme = FALLBACK_THEME;
		}
	}

	function durationForPlayer(): number {
		if (playbackState && playbackState.durationSeconds > 0) {
			return playbackState.durationSeconds;
		}
		return item?.mediaEnclosure.durationSeconds ?? 0;
	}

	function skip(deltaSeconds: number) {
		const current = playbackState?.positionSeconds ?? 0;
		const dur = durationForPlayer();
		const target = Math.max(0, Math.min(current + deltaSeconds, dur));
		requestSeekTo(target);
	}

	function togglePlayback() {
		requestTogglePlayback();
	}

	$effect(() => {
		void updateCoverTheme(imageUrl);
	});

	$effect(() => {
		if (!item) return;

		function handleKeydown(event: KeyboardEvent) {
			if (event.code !== 'Space') return;

			const tag = event.target instanceof Element ? event.target.tagName : '';
			if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || tag === 'BUTTON') return;
			if (event.target instanceof HTMLElement && event.target.isContentEditable) return;

			event.preventDefault();
			togglePlayback();
		}

		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
	});

	$effect(() => {
		if (!item || !('mediaSession' in navigator)) {
			return;
		}

		const back = () => skip(-SKIP_SECONDS);
		const forward = () => skip(SKIP_SECONDS);

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
</script>

{#if item && playbackState}
	<div
		class={`cover-view-theme fixed inset-0 overflow-hidden px-12 ${className}`}
		style:--cover-bg-1={coverTheme.bg1}
		style:--cover-bg-2={coverTheme.bg2}
		style:--cover-bg-3={coverTheme.bg3}
		style:--cover-glow-1={coverTheme.glow1}
		style:--cover-glow-2={coverTheme.glow2}
		style:--cover-glow-3={coverTheme.glow3}
		style:--cover-fg={coverTheme.fg}
		style:--cover-fg-muted={coverTheme.fgMuted}
		style:--cover-fg-subtle={coverTheme.fgSubtle}
		style:--cover-accent={coverTheme.accent}
		style:--cover-accent-contrast={coverTheme.accentContrast}
		style:--cover-panel-bg={coverTheme.panelBg}
		style:--cover-panel-border={coverTheme.panelBorder}
		style:--cover-button-bg={coverTheme.buttonBg}
		style:--cover-button-bg-hover={coverTheme.buttonBgHover}
		style:--color-fg-muted={coverTheme.fgMuted}
	>
		<div class="pointer-events-none absolute inset-0">
			<div class="cover-view-backdrop absolute inset-0"></div>
			<div class="cover-view-glow cover-view-glow-1"></div>
			<div class="cover-view-glow cover-view-glow-2"></div>
			<div class="cover-view-glow cover-view-glow-3"></div>
			<div class="cover-view-scrim absolute inset-0"></div>
		</div>

		{#if onClose}
			<button
				type="button"
				class="cover-view-close absolute top-6 left-6 z-20 flex size-12 items-center justify-center rounded-full transition-colors"
				aria-label="Close cover view"
				onclick={onClose}
			>
				<Icon icon="lucide:x" class="size-6" />
			</button>
		{/if}

		<div
			class="relative z-10 grid h-full grid-cols-[minmax(0,1.6fr)_minmax(20rem,0.9fr)] items-center justify-center gap-8 p-8"
		>
			<div class="min-w-0">
				<div class="mx-auto flex w-full max-w-6xl flex-col p-12">
					{#if imageUrl}
						<img
							src={imageUrl}
							alt=""
							class="mx-auto aspect-square w-full max-w-3xl rounded-lg object-cover shadow-sm select-none"
						/>
					{:else}
						<div
							class="mx-auto grid aspect-square w-full max-w-3xl place-items-center rounded-lg text-[var(--cover-fg-subtle)]"
							style:background-color={coverTheme.panelBg}
						>
							<Icon icon="lucide:disc-3" class="size-16" />
						</div>
					{/if}
				</div>

				<div class="mx-auto flex w-full max-w-6xl flex-col gap-6">
					<div
						class="mx-auto grid w-full max-w-6xl grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-6 4xl:max-w-400"
					>
						<div class="min-w-0">
							<AudioPlayerInfo {item} showCover={false} onNavigate={onNavigateToItem} />
						</div>

						<AudioPlayerControls
							durationSeconds={durationForPlayer()}
							isPlaying={playbackState.isPlaying}
							onTogglePlayback={togglePlayback}
							onSkip={skip}
							skipSeconds={SKIP_SECONDS}
						/>

						<div class="flex min-w-0 items-center justify-end gap-2">
							<AudioPlayerVolume volume={playbackState.volume} />

							{#if controls}
								<div class="relative ml-1 shrink-0">
									{@render controls()}
								</div>
							{/if}
						</div>
					</div>

					<div class="flex min-w-0 flex-row gap-4">
						<AudioSeekBar {playbackState} durationSeconds={durationForPlayer()} class="mt-1" />
					</div>
				</div>
			</div>

			<div class="cover-view-side-panel flex h-full max-h-[80vh] min-w-0 flex-col rounded-2xl">
				<div class="flex h-16 shrink-0 items-center justify-between border-b px-4">
					<div>
						<h2 class="text-sm font-semibold text-white">Playing next</h2>
						<p class="text-xs text-white/60">
							{queueItems.length}
							{queueItems.length === 1 ? 'episode' : 'episodes'}
						</p>
					</div>

					{#if queueItems.length > 0 && onClearQueue}
						<button
							type="button"
							class="rounded-lg px-3 py-1.5 text-xs font-medium text-white/60 transition-colors hover:bg-white/10 hover:text-white"
							onclick={onClearQueue}
						>
							Clear
						</button>
					{/if}
				</div>

				<div class="flex-1 overflow-y-auto">
					<QueueList
						{queueItems}
						{manualQueueLength}
						{feeds}
						appearance="inverse"
						rowPaddingClass="px-4"
						separatorPaddingClass="px-4"
						onRemoveItem={onRemoveQueueItem}
						onMoveItemUp={onMoveQueueItemUp}
						onMoveItemDown={onMoveQueueItemDown}
					/>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.cover-view-theme {
		color: var(--cover-fg);
	}

	.cover-view-backdrop {
		background: linear-gradient(135deg, var(--cover-bg-1), var(--cover-bg-2), var(--cover-bg-3));
	}

	.cover-view-scrim {
		background: linear-gradient(180deg, rgba(2, 6, 23, 0.16) 0%, rgba(2, 6, 23, 0.3) 100%);
		backdrop-filter: blur(80px) saturate(135%);
	}

	.cover-view-glow {
		position: absolute;
		border-radius: 9999px;
		filter: blur(120px);
		opacity: 1;
		pointer-events: none;
	}

	.cover-view-glow-1 {
		top: 6%;
		left: 6%;
		width: 26rem;
		height: 26rem;
		background: var(--cover-glow-1);
	}

	.cover-view-glow-2 {
		top: 18%;
		right: 8%;
		width: 28rem;
		height: 28rem;
		background: var(--cover-glow-2);
	}

	.cover-view-glow-3 {
		bottom: 8%;
		left: 24%;
		width: 32rem;
		height: 32rem;
		background: var(--cover-glow-3);
	}

	.cover-view-close {
		color: var(--cover-fg-muted);
		background: var(--cover-button-bg);
		border: 1px solid var(--cover-panel-border);
		backdrop-filter: blur(18px);
	}

	.cover-view-close:hover {
		color: var(--cover-fg);
		background: var(--cover-button-bg-hover);
	}

	.cover-view-side-panel {
		background: rgba(0, 0, 0, 0.18);
		border: 1px solid rgba(255, 255, 255, 0.12);
		backdrop-filter: blur(22px);
	}

	.cover-view-side-panel > :global(div:first-child) {
		border-color: rgba(255, 255, 255, 0.14);
	}

	/**
     * Local semantic token overrides for descendants.
     * These do not affect the same components elsewhere.
     */
	.cover-view-theme :global(.text-fg) {
		color: var(--cover-fg) !important;
	}

	.cover-view-theme :global(.text-fg-muted) {
		color: var(--cover-fg-muted) !important;
	}

	.cover-view-theme :global(.text-fg-subtle) {
		color: var(--cover-fg-subtle) !important;
	}

	.cover-view-theme :global(.text-accent),
	.cover-view-theme :global(.hover\:text-accent:hover),
	.cover-view-theme :global(.focus-visible\:text-accent:focus-visible) {
		color: var(--cover-accent) !important;
	}

	/**
     * Local overrides for your preset button styles.
     */
	.cover-view-theme :global(.preset-icon-subtle) {
		color: var(--cover-fg) !important;
		background: var(--cover-button-bg) !important;
		border-color: var(--cover-panel-border) !important;
		backdrop-filter: blur(18px);
	}

	.cover-view-theme :global(.preset-icon-subtle:hover) {
		background: var(--cover-button-bg-hover) !important;
	}

	.cover-view-theme :global(.preset-filled-accent) {
		color: var(--cover-accent-contrast) !important;
		background: var(--cover-accent) !important;
		border-color: transparent !important;
	}

	.cover-view-theme :global(.preset-filled-accent:hover) {
		filter: brightness(1.04);
	}

	/**
     * If btn-icon adds its own colors/borders, this keeps them aligned in cover mode.
     */
	.cover-view-theme :global(.btn-icon) {
		box-shadow: none;
	}
</style>
