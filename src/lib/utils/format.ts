export function formatDate(dateLike: string): string {
	return new Intl.DateTimeFormat(undefined, {
		month: 'short',
		day: 'numeric',
		year: 'numeric'
	}).format(new Date(dateLike));
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
