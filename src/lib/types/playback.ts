export interface BackendPlaybackState {
	itemId: string;
	positionSeconds: number;
	durationSeconds: number;
	fileDurationSeconds: number | null;
	isPlaying: boolean;
	isBuffering: boolean;
	isFullyDownloaded: boolean;
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
