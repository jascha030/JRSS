export const appState = $state({
	initialized: false
});

export function resetAppState(): void {
	appState.initialized = false;
}

export function markAppInitialized(): void {
	appState.initialized = true;
}
