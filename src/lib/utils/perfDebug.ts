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

export function isPerfDebugEnabled(): boolean {
	return hasQueryFlag('perf') || hasStorageFlag('jrss:perf');
}

export function isPerfDebugFlagEnabled(flag: string): boolean {
	return hasQueryFlag(flag) || hasStorageFlag(`jrss:perf:${flag}`);
}

export function logPerf(label: string, details?: Record<string, unknown>): void {
	if (!isPerfDebugEnabled()) {
		return;
	}

	if (details) {
		console.info(`[perf] ${label}`, details);
		return;
	}

	console.info(`[perf] ${label}`);
}

export function startPerfMeasure(label: string): () => void {
	if (!isPerfDebugEnabled()) {
		return () => {};
	}

	const startedAt = performance.now();

	return () => {
		logPerf(label, {
			durationMs: Number((performance.now() - startedAt).toFixed(2))
		});
	};
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
