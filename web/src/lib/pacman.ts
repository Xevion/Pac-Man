export interface PacmanModule {
	canvas: HTMLCanvasElement;
	// Restrict keyboard capture to this element (default: document)
	keyboardListeningElement?: HTMLElement;
	_start_game?: () => void;
	_stop_game?: () => void;
	_restart_game?: () => void;
	// Hand the canvas's drawable size (physical pixels) to the game so SDL resizes
	// its window and the adaptive layout recomputes.
	_pacman_resize?: (width: number, height: number) => void;
	// Emscripten's runtime helper for calling exported C functions with marshaled
	// arguments (used to push leaderboard rows, which carry a string, into the game).
	ccall?: (
		ident: string,
		returnType: string | null,
		argTypes: string[],
		args: unknown[]
	) => unknown;
	locateFile: (path: string) => string;
	preRun: Array<() => void>;
	// Emscripten lifecycle hooks
	onAbort?: (what: unknown) => void;
	onRuntimeInitialized?: () => void;
	monitorRunDependencies?: (left: number) => void;
	// Preloaded data file provider - called by Emscripten's file packager
	getPreloadedPackage?: (name: string, size: number) => ArrayBuffer;
}

export type LoadingError =
	{ type: 'timeout' } | { type: 'script'; message: string } | { type: 'runtime'; message: string };

export interface PacmanWindow extends Window {
	Module?: PacmanModule;
	pacmanReady?: () => void;
	pacmanError?: (error: LoadingError) => void;
	// Called by the game via `run_script` when a run ends, carrying the final score,
	// level reached, and play duration.
	pacmanGameOver?: (score: number, levelCount: number, durationMs: number) => void;
	SDL_CANVAS_ID?: string;
}

export const getPacmanWindow = (): PacmanWindow => window as unknown as PacmanWindow;

/** A leaderboard row in the shape the game's FFI exports expect. */
export interface LeaderboardRow {
	name: string;
	score: number;
}

/** Pushes ranked leaderboard rows into the running game via the
 * `pacman_leaderboard_clear`/`pacman_leaderboard_push` FFI exports. A no-op if the
 * module isn't loaded yet or was built without `ccall` exported. */
export function pushLeaderboardToGame(
	entries: LeaderboardRow[],
	module: PacmanModule | undefined
): void {
	if (!module?.ccall) return;

	module.ccall('pacman_leaderboard_clear', null, [], []);
	for (const entry of entries) {
		module.ccall('pacman_leaderboard_push', null, ['string', 'number'], [entry.name, entry.score]);
	}
}
