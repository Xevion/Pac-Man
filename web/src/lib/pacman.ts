export interface PacmanModule {
	canvas: HTMLCanvasElement;
	// Restrict keyboard capture to this element (default: document)
	keyboardListeningElement?: HTMLElement;
	_start_game?: () => void;
	_stop_game?: () => void;
	_restart_game?: () => void;
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
	| { type: 'timeout' }
	| { type: 'script'; message: string }
	| { type: 'runtime'; message: string };

export interface PacmanWindow extends Window {
	Module?: PacmanModule;
	pacmanReady?: () => void;
	pacmanError?: (error: LoadingError) => void;
	SDL_CANVAS_ID?: string;
}

export const getPacmanWindow = (): PacmanWindow => window as unknown as PacmanWindow;
