import { initializeApp } from '$lib/stores/app.svelte';

/** @type {import('@sveltejs/kit').ClientInit} */
export async function init() {
	await initializeApp();
}
