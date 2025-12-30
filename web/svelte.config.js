import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapter({
			pages: 'dist/client',
			assets: 'dist/client',
			fallback: undefined,
			precompress: false,
			strict: true
		}),
		// Inline CSS below 5KB for fewer requests
		inlineStyleThreshold: 5000,
		alias: {
			$lib: './src/lib'
		}
	}
};

export default config;
