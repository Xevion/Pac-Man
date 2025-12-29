export interface PacmanModule {
  canvas: HTMLCanvasElement;
  _start_game?: () => void;
  _stop_game?: () => void;
  _restart_game?: () => void;
  locateFile: (path: string) => string;
  preRun: unknown[];
  // Emscripten error hooks
  onAbort?: (what: unknown) => void;
  onRuntimeInitialized?: () => void;
}

export type LoadingError =
  | { type: "timeout" }
  | { type: "script"; message: string }
  | { type: "runtime"; message: string };

export interface PacmanWindow extends Window {
  Module?: PacmanModule;
  pacmanReady?: () => void;
  pacmanError?: (error: LoadingError) => void;
  SDL_CANVAS_ID?: string;
}

export const getPacmanWindow = (): PacmanWindow => window as unknown as PacmanWindow;
