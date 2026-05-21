type VolumeInputEvent = Event & {
	currentTarget: EventTarget & HTMLInputElement;
};

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
		const nextVolume = Number(event.currentTarget.value);
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
		toggleMute
	};
}
