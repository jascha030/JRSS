export interface PlaybackState {
	itemId: string;
	positionSeconds: number;
	durationSeconds: number;
	isPlaying: boolean;
	isBuffering: boolean;
	volume: number;
}

export interface PlaybackContext {
	contextType: 'feed' | 'station';
	id: string;
}

export interface PlaybackSession {
	currentItemId?: string;
	positionSeconds: number;
	durationSeconds: number;
	historyQueue: string[];
	manualQueue: string[];
	autoQueue: string[];
	playbackContext?: PlaybackContext;
}

export interface BackendPlaybackState {
	itemId: string;
	positionSeconds: number;
	durationSeconds: number;
	isPlaying: boolean;
	isBuffering: boolean;
	volume: number;
}

export interface BackendPlaybackEndedEvent {
	itemId: string;
}

export interface BackendPlaybackErrorEvent {
	itemId: string;
	error: string;
}

export interface BackendQueuedItem {
	itemId: string;
	url: string;
	title: string;
	durationSeconds: number;
}

export interface BackendQueueState {
	history: BackendQueuedItem[];
	manual: BackendQueuedItem[];
	auto: BackendQueuedItem[];
	current: BackendQueuedItem | null;
}
