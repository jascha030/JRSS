import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	resolve: process.env.VITEST
		? {
				conditions: ['browser']
			}
		: undefined,
	build: {
		cssMinify: true,
		rolldownOptions: {
			output: {
				manualChunks(id) {
					if (id.includes('monaco-editor')) {
						return 'monaco';
					}
				}
			}
		}
	},
	test: {
		include: ['src/**/*.{test,spec}.{js,ts,svelte}'],
		globals: true,
		environment: 'happy-dom',
		setupFiles: ['./vitest.setup.ts'],
		coverage: {
			include: ['src/**/*.{svelte,ts,js}'],
			exclude: [
				'src/**/*.test.{js,ts,svelte}',
				'src/**/*.spec.{js,ts,svelte}',
				'src/lib/types/**',
				'src/routes/+layout.ts'
			]
		}
	}
});
