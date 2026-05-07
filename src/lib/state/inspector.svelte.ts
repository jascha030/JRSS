import { fetchFeedRawXml } from '$lib/services/feedService';

export const inspectorState = $state({
	activeFeedId: null as string | null,
	xmlContent: null as string | null,
	loading: false,
	error: null as string | null
});

export function resetInspectorState(): void {
	inspectorState.activeFeedId = null;
	inspectorState.xmlContent = null;
	inspectorState.loading = false;
	inspectorState.error = null;
}

export async function openInspector(feedId: string): Promise<void> {
	inspectorState.activeFeedId = feedId;
	inspectorState.xmlContent = null;
	inspectorState.loading = true;
	inspectorState.error = null;

	try {
		inspectorState.xmlContent = await fetchFeedRawXml(feedId);
	} catch (error: unknown) {
		inspectorState.error = error instanceof Error ? error.message : 'Failed to fetch feed XML.';
	} finally {
		inspectorState.loading = false;
	}
}

export function closeInspector(): void {
	inspectorState.activeFeedId = null;
	inspectorState.xmlContent = null;
	inspectorState.loading = false;
	inspectorState.error = null;
}
