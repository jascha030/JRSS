type VolumeInputEvent = Event & {
	currentTarget: EventTarget & HTMLInputElement;
};

type VolumeWheelEvent = WheelEvent & {
	currentTarget: EventTarget & HTMLDivElement;
};

const VOLUME_MIN = 0;
const VOLUME_MAX = 1;
const VOLUME_WHEEL_STEP = 0.05;

function clampVolume(volume: number): number {
	return Math.min(VOLUME_MAX, Math.max(VOLUME_MIN, volume));
}

type Options = {
	getVolume: () => number;
	setVolume: (value: number) => void;
	defaultVolume?: number;
};

export function useVolumeControl({ getVolume, setVolume, defaultVolume = 1 }: Options) {
	let volumeOverride = $state<number | null>(null);
	let previousNonZeroVolume = $state(defaultVolume);

	const effectiveVolume = $derived(volumeOverride ?? getVolume());
	const isMuted = $derived(effectiveVolume === 0);

	function handleVolumeInput(event: VolumeInputEvent) {
		const nextVolume = clampVolume(Number(event.currentTarget.value));
		volumeOverride = nextVolume;

		if (nextVolume > 0) {
			previousNonZeroVolume = nextVolume;
		}
	}

	function handleVolumeWheel(event: VolumeWheelEvent) {
		if (event.deltaY === 0) {
			return;
		}

		event.preventDefault();

		const direction = Math.sign(event.deltaY);
		const nextVolume = clampVolume(effectiveVolume - direction * VOLUME_WHEEL_STEP);

		volumeOverride = nextVolume;

		if (nextVolume > 0) {
			previousNonZeroVolume = nextVolume;
		}
	}

	function toggleMute() {
		if (effectiveVolume === 0) {
			volumeOverride = previousNonZeroVolume > 0 ? previousNonZeroVolume : defaultVolume;
			return;
		}

		previousNonZeroVolume = effectiveVolume;
		volumeOverride = 0;
	}

	$effect(() => {
		const volume = getVolume();

		if (volume > 0) {
			previousNonZeroVolume = volume;
		}
	});

	$effect(() => {
		const volume = getVolume();

		if (volumeOverride !== null && Math.abs(volumeOverride - volume) < 0.001) {
			volumeOverride = null;
		}
	});

	$effect(() => {
		const volume = getVolume();

		if (Math.abs(effectiveVolume - volume) < 0.001) {
			return;
		}

		setVolume(effectiveVolume);
	});

	return {
		get effectiveVolume() {
			return effectiveVolume;
		},
		get isMuted() {
			return isMuted;
		},
		handleVolumeInput,
		handleVolumeWheel,
		toggleMute
	};
}
