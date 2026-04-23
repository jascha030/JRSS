<script lang="ts">
	import { onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import Icon from '@iconify/svelte';

	type Props = {
		onOpenDialog: () => void;
	};

	let { onOpenDialog }: Props = $props();

	onMount(() => {
		let unlisten: UnlistenFn | undefined;

		const setupListener = async () => {
			unlisten = await listen('menu-add-feed', () => {
				onOpenDialog();
			});
		};

		void setupListener();

		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});
</script>

<button type="button" class="btn gap-2 preset-filled" onclick={onOpenDialog}>
	<Icon icon="lucide:plus" class="size-4" />
	<span>Add feed</span>
</button>
