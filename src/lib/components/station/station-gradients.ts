import type { StationGradient } from '$lib/types/station';

export const STATION_GRADIENTS: Record<
	StationGradient,
	{ from: string; to: string; activeFrom: string; activeTo: string }
> = {
	emerald: {
		from: 'from-success-400',
		to: 'to-success-600',
		activeFrom: 'from-success-500',
		activeTo: 'to-success-700'
	},
	violet: {
		from: 'from-violet-400',
		to: 'to-violet-600',
		activeFrom: 'from-violet-500',
		activeTo: 'to-violet-700'
	},
	rose: {
		from: 'from-rose-400',
		to: 'to-rose-600',
		activeFrom: 'from-rose-500',
		activeTo: 'to-rose-700'
	},
	amber: {
		from: 'from-amber-400',
		to: 'to-amber-600',
		activeFrom: 'from-amber-500',
		activeTo: 'to-amber-700'
	},
	cyan: {
		from: 'from-cyan-400',
		to: 'to-cyan-600',
		activeFrom: 'from-cyan-500',
		activeTo: 'to-cyan-700'
	},
	fuchsia: {
		from: 'from-fuchsia-400',
		to: 'to-fuchsia-600',
		activeFrom: 'from-fuchsia-500',
		activeTo: 'to-fuchsia-700'
	},
	slate: {
		from: 'from-slate-400',
		to: 'to-slate-600',
		activeFrom: 'from-slate-500',
		activeTo: 'to-slate-700'
	},
	orange: {
		from: 'from-orange-400',
		to: 'to-orange-600',
		activeFrom: 'from-orange-500',
		activeTo: 'to-orange-700'
	}
};

export const STATION_GRADIENT_OPTIONS: StationGradient[] = [
	'emerald',
	'violet',
	'rose',
	'amber',
	'cyan',
	'fuchsia',
	'slate',
	'orange'
];
