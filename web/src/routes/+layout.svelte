<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';
	import { onNavigate } from '$app/navigation';
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import {
		IconBrandGithub,
		IconDownload,
		IconDeviceGamepad3,
		IconTrophy
	} from '@tabler/icons-svelte';
	import NavLink from '$lib/components/NavLink.svelte';

	let { children } = $props();

	let opened = $state(false);

	// Keys that the game uses - only these should reach SDL/Emscripten on the play page
	const GAME_KEYS = new Set([
		'ArrowUp',
		'ArrowDown',
		'ArrowLeft',
		'ArrowRight',
		'w',
		'W',
		'a',
		'A',
		's',
		'S',
		'd',
		'D',
		'Escape',
		' ',
		'm',
		'M',
		'r',
		'R',
		't',
		'T'
	]);

	onMount(() => {
		// Global keyboard filter to prevent SDL/Emscripten from capturing keys.
		// SDL's handlers persist globally even after navigating away from the play page.
		// This filter ensures browser shortcuts (F5, F12, Ctrl+R, etc.) always work.
		const filterKeyEvent = (event: KeyboardEvent) => {
			const isPlayPage = window.location.pathname === '/';
			const canvas = document.getElementById('canvas');

			// On non-play pages, block ALL keys from reaching SDL
			if (!isPlayPage) {
				event.stopPropagation();
				return;
			}

			// On play page: nuanced filtering

			// Tab: blur canvas and let browser handle focus navigation
			if (event.key === 'Tab') {
				if (document.activeElement === canvas && canvas) {
					canvas.blur();
					// Focus first tabbable element in the header
					const firstLink = document.querySelector('header a') as HTMLElement | null;
					if (firstLink && !event.shiftKey) {
						firstLink.focus();
						event.preventDefault();
					}
				}
				event.stopPropagation();
				return;
			}

			// Escape: let it through to game (for pause) but also blur canvas
			if (event.key === 'Escape') {
				canvas?.blur();
				return; // Don't stop propagation - game still receives it for pause
			}

			// If it's a game key, let it through to SDL
			if (GAME_KEYS.has(event.key)) {
				return;
			}

			// For all other keys (F5, F12, Ctrl+anything, etc.):
			// Stop SDL from seeing them so browser can handle them normally
			event.stopPropagation();
		};

		// Register in capturing phase to intercept before SDL sees the events
		window.addEventListener('keydown', filterKeyEvent, true);
		window.addEventListener('keyup', filterKeyEvent, true);
		window.addEventListener('keypress', filterKeyEvent, true);

		return () => {
			window.removeEventListener('keydown', filterKeyEvent, true);
			window.removeEventListener('keyup', filterKeyEvent, true);
			window.removeEventListener('keypress', filterKeyEvent, true);
		};
	});

	// Use View Transitions API for smooth page transitions
	onNavigate((navigation) => {
		if (!document.startViewTransition) return;

		return new Promise((resolve) => {
			document.startViewTransition(async () => {
				resolve();
				await navigation.complete;
			});
		});
	});

	const links = [
		{
			label: 'Play',
			href: '/',
			icon: IconDeviceGamepad3
		},
		{
			label: 'Leaderboard',
			href: '/leaderboard',
			icon: IconTrophy
		},
		{
			label: 'Download',
			href: '/download',
			icon: IconDownload
		},
		{
			label: 'GitHub',
			href: 'https://github.com/Xevion/Pac-Man',
			icon: IconBrandGithub
		}
	];

	const toggle = () => (opened = !opened);
	const close = () => (opened = false);

	let currentPath = $derived($page.url.pathname);
	let isIndexPage = $derived(currentPath === '/');

	function isActive(href: string): boolean {
		return href === '/' ? currentPath === href : currentPath.startsWith(href);
	}

	const sourceLinks = links.filter((link) => !link.href.startsWith('/'));
</script>

<svelte:head>
	<link rel="icon" href="/favicon.ico" />
	<title>Pac-Man</title>
	<meta name="description" content="A Pac-Man game built with Rust and Svelte." />
</svelte:head>

<div class="bg-black text-yellow-400 h-screen flex flex-col overflow-hidden">
	<header class="shrink-0 h-[60px] border-b border-yellow-400/25 bg-black z-20">
		<div class="h-full px-4 flex items-center justify-center">
			<button
				aria-label="Open navigation"
				onclick={toggle}
				class="sm:hidden absolute left-4 inline-flex items-center justify-center w-9 h-9 rounded border border-yellow-400/30 text-yellow-400"
			>
				<span class="sr-only">Toggle menu</span>
				<div class="w-5 h-0.5 bg-yellow-400"></div>
			</button>

			<div class="flex items-center gap-8">
				<NavLink
					href="/leaderboard"
					icon={IconTrophy}
					label="Leaderboard"
					active={isActive('/leaderboard')}
				/>

				<a
					href="/"
					onclick={(e) => {
						if (isIndexPage) {
							e.preventDefault();
						}
					}}
				>
					<h1
						class="text-3xl tracking-[0.3em] text-yellow-400 title-base {isIndexPage
							? ''
							: 'title-glimmer title-hover'}"
						style="font-family: 'Russo One'"
						data-text="PAC-MAN"
					>
						PAC-MAN
					</h1>
				</a>

				<NavLink
					href="/download"
					icon={IconDownload}
					label="Download"
					active={isActive('/download')}
				/>
			</div>

			<div class="absolute right-4 hidden sm:flex gap-4 items-center">
				{#each sourceLinks as link (link.href)}
					<a
						href={link.href}
						title={link.label}
						target="_blank"
						class="text-gray-500 hover:text-gray-300 transition-colors duration-200"
					>
						<link.icon size={28} />
					</a>
				{/each}
			</div>
		</div>
	</header>

	{#if browser}
		<OverlayScrollbarsComponent
			defer
			options={{
				scrollbars: {
					theme: 'os-theme-light',
					autoHide: 'scroll',
					autoHideDelay: 1300
				}
			}}
			class="flex-1"
		>
			<main>
				{@render children()}
			</main>
		</OverlayScrollbarsComponent>
	{:else}
		<div class="flex-1 overflow-auto">
			<main>{@render children()}</main>
		</div>
	{/if}

	{#if opened}
		<div class="fixed inset-0 z-30">
			<div
				role="button"
				tabindex="0"
				class="absolute inset-0 bg-black/60"
				onclick={close}
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						close();
					}
				}}
			></div>
			<div
				class="absolute left-0 top-0 h-full w-72 max-w-[80vw] bg-black border-r border-yellow-400/25 p-4"
			>
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-lg font-bold">Navigation</h2>
					<button
						aria-label="Close navigation"
						onclick={close}
						class="inline-flex items-center justify-center w-8 h-8 rounded border border-yellow-400/30 text-yellow-400"
					>
						✕
					</button>
				</div>
				<div class="flex flex-col gap-3">
					{#each links as link (link.href)}
						<NavLink
							href={link.href}
							icon={link.icon}
							label={link.label}
							active={isActive(link.href)}
							size={28}
						/>
					{/each}
				</div>
			</div>
		</div>
	{/if}
</div>
