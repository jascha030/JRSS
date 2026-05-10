import { DEFAULT_SKIP_FORWARD_SECONDS, DEFAULT_SKIP_BACKWARD_SECONDS } from '$lib/types/settings';

let _skipForwardSeconds = $state(DEFAULT_SKIP_FORWARD_SECONDS);
let _skipBackwardSeconds = $state(DEFAULT_SKIP_BACKWARD_SECONDS);

export const playbackSettings = {
	get skipForwardSeconds(): number {
		return _skipForwardSeconds;
	},
	get skipBackwardSeconds(): number {
		return _skipBackwardSeconds;
	},
	setSkipForwardSeconds(v: number) {
		_skipForwardSeconds = v;
	},
	setSkipBackwardSeconds(v: number) {
		_skipBackwardSeconds = v;
	}
};
