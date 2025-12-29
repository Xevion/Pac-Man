export interface PacmanModule {
  canvas: HTMLCanvasElement;
  _start_game?: () => void;
  _stop_game?: () => void;
  _restart_game?: () => void;
  locateFile: (path: string) => string;
  preRun: unknown[];
}

export interface PacmanWindow extends Window {
  Module?: PacmanModule;
  pacmanReady?: () => void;
  SDL_CANVAS_ID?: string;
}

export const getPacmanWindow = (): PacmanWindow => window as unknown as PacmanWindow;
