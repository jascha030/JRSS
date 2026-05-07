<style>
	:global(.cover-theme) {
		color: var(--cover-fg);
	}

	:global(.cover-theme .text-fg) {
		color: var(--cover-fg) !important;
	}

	:global(.cover-theme .text-fg-muted) {
		color: var(--cover-fg-muted) !important;
	}

	:global(.cover-theme .text-fg-subtle) {
		color: var(--cover-fg-subtle) !important;
	}

	:global(.cover-theme .text-accent),
	:global(.cover-theme .hover\:text-accent:hover),
	:global(.cover-theme .focus-visible\:text-accent:focus-visible) {
		color: var(--cover-accent) !important;
	}

	:global(.cover-theme .preset-icon-subtle) {
		color: var(--cover-fg) !important;
		background: var(--cover-button-bg) !important;
		border-color: var(--cover-panel-border) !important;
		backdrop-filter: blur(18px);
	}

	:global(.cover-theme .preset-icon-subtle:hover) {
		background: var(--cover-button-bg-hover) !important;
	}

	:global(.cover-theme .preset-filled-accent) {
		color: var(--cover-accent-contrast) !important;
		background: var(--cover-accent) !important;
		border-color: transparent !important;
	}

	:global(.cover-theme .preset-filled-accent:hover) {
		filter: brightness(1.04);
	}

	:global(.cover-theme .btn-icon) {
		box-shadow: none;
	}

	:global(.cover-theme .player-range) {
		--fill: var(--cover-seek-fill) !important;
	}

	:global(.cover-theme .player-range:focus-visible) {
		outline-color: var(--cover-seek-fill) !important;
	}
</style>
