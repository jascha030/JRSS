import { initializeApp } from '$lib/state';

/** @type {import('@sveltejs/kit').ClientInit} */
export async function init() {
	await initializeApp();
}
