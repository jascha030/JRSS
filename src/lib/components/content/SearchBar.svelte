<script lang="ts">
	import SearchInput from '$lib/components/ui/SearchInput.svelte';

	type Props = {
		label: string;
		placeholder: string;
		value: string;
		onChange: (value: string) => void;
		inputRef?: HTMLInputElement | null;
	};

	let { label, placeholder, value, onChange, inputRef = $bindable(null) }: Props = $props();

	function handleInput(event: Event & { currentTarget: HTMLInputElement }) {
		onChange(event.currentTarget.value);
	}

	function handleEscape() {
		inputRef?.blur();
		if (value !== '') {
			onChange('');
		}
	}
</script>

<div class="flex w-full flex-row flex-wrap items-center justify-between gap-4">
	<div class="flex-1">
		<SearchInput
			id="list-search"
			{label}
			{placeholder}
			{value}
			bind:inputRef
			kbdShortcuts={['⌘', 'F']}
			oninput={handleInput}
			onkeydown={(event) => {
				if (event.key === 'Escape') {
					handleEscape();
				}
			}}
		/>
	</div>
</div>
