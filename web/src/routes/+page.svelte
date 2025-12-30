<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { beforeNavigate, afterNavigate } from '$app/navigation';
	import { getPacmanWindow, type LoadingError } from '$lib/pacman';

	const LOADING_FADE_DURATION = 300;
	const LOADING_TIMEOUT_MS = 15000;

	let gameReady = $state(false);
	let gameStarted = $state(false);
	let loadingVisible = $state(true);
	let loadError = $state<LoadingError | null>(null);
	let timeoutId: ReturnType<typeof setTimeout> | null = null;

	// Fade out loading overlay when game becomes ready
	$effect(() => {
		if (gameReady && loadingVisible) {
			const timer = setTimeout(() => {
				loadingVisible = false;
			}, LOADING_FADE_DURATION);
			return () => clearTimeout(timer);
		}
	});

	// Clear timeout when game is ready or error occurs
	$effect(() => {
		if (gameReady || loadError) {
			if (timeoutId) {
				clearTimeout(timeoutId);
				timeoutId = null;
			}
		}
	});

	function handleInteraction() {
		if (gameReady && !gameStarted) {
			const win = getPacmanWindow();
			if (win.Module?._start_game) {
				win.Module._start_game();
				gameStarted = true;
			}
		}
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (!gameReady || gameStarted) return;
		handleInteraction();
	}

	// Stop game when navigating away
	beforeNavigate(({ to }) => {
		if (to) {
			const win = getPacmanWindow();
			if (win.Module?._stop_game) {
				try {
					console.log('Stopping game loop for page transition');
					win.Module._stop_game();
				} catch (error) {
					console.warn('Failed to stop game (game may have already crashed):', error);
				}
			}
		}
	});

	// Restart game when returning to this page
	afterNavigate(() => {
		requestAnimationFrame(() => {
			setTimeout(() => {
				restartGame();
			}, 0);
		});
	});

	function restartGame() {
		const win = getPacmanWindow();
		const module = win.Module;

		if (!module?._restart_game) {
			console.warn('Game restart function not available (WASM may not be initialized)');
			return;
		}

		const canvas = document.getElementById('canvas') as HTMLCanvasElement | null;
		if (!canvas) {
			console.error('Canvas element not found during game restart');
			return;
		}

		module.canvas = canvas;
		win.SDL_CANVAS_ID = '#canvas';

		try {
			console.log('Restarting game with fresh App instance');
			module._restart_game();
		} catch (error) {
			console.error('Failed to restart game:', error);
		}
	}

	onMount(() => {
		const win = getPacmanWindow();

		// Set up ready callback
		win.pacmanReady = () => {
			gameReady = true;
		};

		// Error callback for WASM runtime errors
		win.pacmanError = (error: LoadingError) => {
			console.error('Pacman error:', error);
			loadError = error;
		};

		// Canvas is needed for both first-time init and return navigation
		const canvas = document.getElementById('canvas') as HTMLCanvasElement | null;
		if (!canvas) {
			console.error('Canvas element not found');
			loadError = { type: 'runtime', message: 'Canvas element not found' };
			return;
		}

		// Click outside canvas to unfocus it
		const handleClickOutside = (event: MouseEvent) => {
			if (event.target !== canvas) {
				canvas.blur();
			}
		};
		document.addEventListener('click', handleClickOutside);

		// Keyboard listener for click-to-start interaction
		window.addEventListener('keydown', handleKeyDown);

		// Cleanup function used by both paths (return navigation and first-time init)
		const cleanup = () => {
			document.removeEventListener('click', handleClickOutside);
			window.removeEventListener('keydown', handleKeyDown);
		};

		const module = win.Module;

		// If Module already exists (returning after navigation), restart it
		if (module?._restart_game) {
			gameStarted = false;
			return cleanup;
		}

		// First time initialization
		const version = import.meta.env.VITE_PACMAN_VERSION;
		console.log(`Loading Pacman with version: ${version}`);

		win.Module = {
			canvas,
			// Restrict keyboard capture to canvas only (not whole document)
			// This allows Tab, F5, etc. to work when canvas isn't focused
			keyboardListeningElement: canvas,
			locateFile: (path: string) => {
				const normalizedPath = path.startsWith('/') ? path : `/${path}`;
				return `${normalizedPath}?v=${version}`;
			},
			preRun: [
				function () {
					console.log('PreRun: Waiting for filesystem to be ready');
				}
			],
			monitorRunDependencies: (left: number) => {
				console.log(`Run dependencies remaining: ${left}`);
			},
			onRuntimeInitialized: () => {
				console.log('Emscripten runtime initialized, filesystem ready');
			},
			onAbort: (what: unknown) => {
				const message = typeof what === 'string' ? what : 'WebAssembly execution aborted';
				console.error('WASM abort:', what);
				loadError = { type: 'runtime', message };
			}
		};

		const script = document.createElement('script');
		script.src = `/pacman.js?v=${version}`;
		script.async = false;

		script.onerror = () => {
			loadError = { type: 'script', message: 'Failed to load game script' };
		};

		document.body.appendChild(script);

		// Set up loading timeout
		timeoutId = setTimeout(() => {
			if (!loadError) {
				loadError = { type: 'timeout' };
			}
		}, LOADING_TIMEOUT_MS);

		return cleanup;
	});

	onDestroy(() => {
		const win = getPacmanWindow();
		delete win.pacmanReady;
		delete win.pacmanError;
		if (timeoutId) {
			clearTimeout(timeoutId);
		}
	});

	function focusCanvas(e: MouseEvent) {
		(e.currentTarget as HTMLCanvasElement).focus();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="flex justify-center items-center h-full pt-4">
	<div
		role="button"
		tabindex="-1"
		class="relative block aspect-[5/6]"
		style="height: min(calc(100vh - 96px), calc((100vw - 32px) * 6 / 5));"
		onclick={handleInteraction}
	>
		<canvas id="canvas" tabindex="-1" class="w-full h-full" onclick={focusCanvas}></canvas>

		<!-- Loading overlay -->
		{#if loadingVisible}
			<div
				class="absolute inset-0 flex flex-col items-center justify-center bg-black/80 transition-opacity"
				style="transition-duration: {LOADING_FADE_DURATION}ms; opacity: {gameReady ? 0 : 1};"
			>
				{#if loadError}
					<div class="error-indicator"></div>
					<span class="text-red-500 text-2xl mt-4 font-semibold">
						{loadError.type === 'timeout'
							? 'Loading timed out'
							: loadError.type === 'script'
								? 'Failed to load'
								: 'Error occurred'}
					</span>
					<span class="text-gray-400 text-sm mt-2 max-w-xs text-center">
						{loadError.type === 'timeout'
							? 'The game took too long to load. Please refresh the page.'
							: loadError.type === 'script'
								? 'Could not load game files. Check your connection and refresh.'
								: loadError.message}
					</span>
					<button
						onclick={() => window.location.reload()}
						class="mt-4 px-4 py-2 bg-yellow-400 text-black font-semibold rounded hover:bg-yellow-300 transition-colors"
					>
						Reload
					</button>
				{:else}
					<div class="loading-spinner"></div>
					<span class="text-yellow-400 text-2xl mt-4">Loading...</span>
				{/if}
			</div>
		{/if}

		<!-- Click to Start overlay -->
		{#if gameReady && !gameStarted}
			<div class="absolute inset-0 flex items-center justify-center bg-black/60 cursor-pointer">
				<span class="text-yellow-400 text-5xl font-bold">Click to Start</span>
			</div>
		{/if}
	</div>
</div>
