import devtoolsJson from 'vite-plugin-devtools-json';
import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { type Plugin } from 'vite';
import { execSync } from 'child_process';
import { fontSubsetPlugin, type FontSubsetConfig } from './vite-plugin-font-subset';

// Character sets for font subsetting
const TITLE_CHARS = 'PACMN-';

const COMMON_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 .,!?':;-_()/@#&*+=%<>";

const fontConfig: FontSubsetConfig = {
	fonts: [
		{
			source: '@fontsource/russo-one/files/russo-one-latin-400-normal.woff2',
			whitelist: TITLE_CHARS
		},

		{
			source: '@fontsource/outfit/files/outfit-latin-400-normal.woff2',
			whitelist: COMMON_CHARS,
			family: 'Outfit'
		},

		{
			source: '@fontsource/outfit/files/outfit-latin-500-normal.woff2',
			whitelist: COMMON_CHARS,
			family: 'Outfit'
		}
	]
};

/**
 * Vite plugin that injects the Pacman version hash at build time.
 * Uses git commit hash in production/dev, falls back to timestamp if git unavailable.
 */
function pacmanVersionPlugin(): Plugin {
	function getVersion(mode: string): string {
		if (mode === 'development') {
			return 'dev';
		}

		try {
			const hash = execSync('git rev-parse --short HEAD', { encoding: 'utf8', stdio: ['pipe', 'pipe', 'pipe'] }).trim();

			if (hash) {
				return hash;
			}
		} catch {
			// Git not available or command failed
		}

		return Date.now().toString(36);
	}

	return {
		name: 'pacman-version',

		config(_, { mode }) {
			const version = getVersion(mode);

			console.log(`[pacman-version] Using version: ${version}`);

			return {
				define: {
					'import.meta.env.VITE_PACMAN_VERSION': JSON.stringify(version)
				}
			};
		}
	};
}

export default defineConfig({
	plugins: [
		fontSubsetPlugin(fontConfig),
		pacmanVersionPlugin(),
		sveltekit(),
		tailwindcss(),
		devtoolsJson()
	],

	build: { target: 'es2022' },

	server: {
		proxy: {
			'/api': {
				target: process.env.VITE_API_TARGET || 'http://localhost:3001',
				changeOrigin: true
			}
		}
	},

	test: {
		expect: { requireAssertions: true },

		projects: [
			{
				extends: './vite.config.ts',

				test: {
					name: 'client',

					browser: {
						enabled: true,
						provider: playwright(),
						instances: [{ browser: 'chromium', headless: true }]
					},

					include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
					exclude: ['src/lib/server/**']
				}
			},

			{
				extends: './vite.config.ts',

				test: {
					name: 'server',
					environment: 'node',
					include: ['src/**/*.{test,spec}.{js,ts}'],
					exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
				}
			}
		]
	}
});
