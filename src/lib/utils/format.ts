/** Reused across calls — construction involves locale resolution and option canonicalization. */
const dateFormatter = new Intl.DateTimeFormat(undefined, {
	month: 'short',
	day: 'numeric',
	year: 'numeric',
	hour: 'numeric',
	minute: '2-digit'
});

const dateOnlyFormatter = new Intl.DateTimeFormat(undefined, {
	month: 'short',
	day: 'numeric',
	year: 'numeric'
});

export function formatDate(dateLike: string): string {
	return dateFormatter.format(new Date(dateLike));
}

export function formatDateOnly(dateLike: string): string {
	return dateOnlyFormatter.format(new Date(dateLike));
}

export function formatDuration(totalSeconds: number): string {
	const safeSeconds =
		Number.isFinite(totalSeconds) && totalSeconds > 0 ? Math.floor(totalSeconds) : 0;

	const hours = Math.floor(safeSeconds / 3600);
	const minutes = Math.floor((safeSeconds % 3600) / 60);
	const seconds = safeSeconds % 60;

	if (hours > 0) {
		return [hours, minutes.toString().padStart(2, '0'), seconds.toString().padStart(2, '0')].join(
			':'
		);
	}

	return [minutes.toString(), seconds.toString().padStart(2, '0')].join(':');
}
