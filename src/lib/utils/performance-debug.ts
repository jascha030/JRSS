function hasQueryFlag(flag: string): boolean {
	if (typeof window === 'undefined') {
		return false;
	}

	return new URL(window.location.href).searchParams.get(flag) === '1';
}

function hasStorageFlag(flag: string): boolean {
	if (typeof window === 'undefined') {
		return false;
	}

	return window.localStorage.getItem(flag) === '1';
}

function isPerfDebugEnabled(): boolean {
	return hasQueryFlag('perf') || hasStorageFlag('jrss:perf');
}

function logPerf(label: string, details?: Record<string, unknown>): void {
	if (!isPerfDebugEnabled()) {
		return;
	}

	if (details) {
		console.info(`[perf] ${label}`, details);
		return;
	}

	console.info(`[perf] ${label}`);
}

export async function measurePerfAsync<T>(
	label: string,
	work: () => Promise<T>,
	details?: Record<string, unknown>
): Promise<T> {
	if (!isPerfDebugEnabled()) {
		return work();
	}

	const startedAt = performance.now();

	try {
		return await work();
	} finally {
		logPerf(label, {
			...details,
			durationMs: Number((performance.now() - startedAt).toFixed(2))
		});
	}
}
