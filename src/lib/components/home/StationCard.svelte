<script lang="ts">
	import type { Station } from '$lib/types/station';
	import { STATION_GRADIENTS } from '$lib/components/station/station-gradients';
	import Icon from '@iconify/svelte';
	import { openStationContextMenu } from '$lib/utils/tauri-menu';

	let { station, onClick }: { station: Station; onClick: () => void } = $props();

	const gradient = $derived(STATION_GRADIENTS[station.gradient]);
</script>

<button
	type="button"
	class="group relative flex w-full flex-col items-center gap-2 rounded-xl p-2 transition-colors contain-[paint] hover:bg-surface-hover"
	onclick={onClick}
	title={station.name}
	oncontextmenu={(e) => void openStationContextMenu(e, station)}
	aria-label={`Open station ${station.name}`}
>
	<div
		class={`relative aspect-square w-full overflow-hidden rounded-xl bg-linear-to-br ${gradient.from} ${gradient.to} shadow-sm`}
	>
		<span class="flex size-full items-center justify-center text-2xl font-bold text-fg-inverse">
			<Icon icon="heroicons:microphone" class="size-10" />
		</span>
	</div>

	<span class="line-clamp-2 w-full text-center text-xs font-medium text-fg">
		{station.name}
	</span>
</button>
