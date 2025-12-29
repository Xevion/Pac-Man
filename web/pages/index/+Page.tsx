import { useCallback, useEffect, useRef, useState } from "react";
import { getPacmanWindow, LoadingError } from "@/lib/pacman";

const LOADING_FADE_DURATION = 300;
const LOADING_TIMEOUT_MS = 15000;

export default function Page() {
  const [gameReady, setGameReady] = useState(false);
  const [gameStarted, setGameStarted] = useState(false);
  const [loadingVisible, setLoadingVisible] = useState(true);
  const [loadError, setLoadError] = useState<LoadingError | null>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Fade out loading overlay when game becomes ready
  useEffect(() => {
    if (gameReady && loadingVisible) {
      const timer = setTimeout(() => {
        setLoadingVisible(false);
      }, LOADING_FADE_DURATION);
      return () => clearTimeout(timer);
    }
  }, [gameReady, loadingVisible]);

  // Clear timeout when game is ready or error occurs
  useEffect(() => {
    if (gameReady || loadError) {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }
    }
  }, [gameReady, loadError]);

  useEffect(() => {
    const win = getPacmanWindow();

    // Always set up the ready callback (restart_game will call it too)
    win.pacmanReady = () => {
      setGameReady(true);
    };

    // Error callback for WASM runtime errors
    win.pacmanError = (error: LoadingError) => {
      console.error("Pacman error:", error);
      setLoadError(error);
    };

    const module = win.Module;

    // If Module already exists (returning after navigation),
    // the onPageTransitionEnd hook handles calling restart_game
    if (module?._restart_game) {
      setGameStarted(false);
      // Don't delete pacmanReady here - restart_game needs it
      return;
    }

    // First time initialization
    const canvas = document.getElementById("canvas") as HTMLCanvasElement | null;
    if (!canvas) {
      console.error("Canvas element not found");
      setLoadError({ type: "runtime", message: "Canvas element not found" });
      return;
    }

    win.Module = {
      canvas,
      locateFile: (path: string) => {
        return path.startsWith("/") ? path : `/${path}`;
      },
      preRun: [],
      // Emscripten calls this on fatal errors (abort/trap/etc)
      onAbort: (what: unknown) => {
        const message = typeof what === "string" ? what : "WebAssembly execution aborted";
        console.error("WASM abort:", what);
        setLoadError({ type: "runtime", message });
      },
    };

    const script = document.createElement("script");
    script.src = "/pacman.js";
    script.async = false;

    // Handle script load errors
    script.onerror = () => {
      setLoadError({ type: "script", message: "Failed to load game script" });
    };

    document.body.appendChild(script);

    // Set up loading timeout - the separate effect clears this if game loads successfully
    timeoutRef.current = setTimeout(() => {
      setLoadError((prev) => prev ?? { type: "timeout" });
    }, LOADING_TIMEOUT_MS);

    return () => {
      delete win.pacmanReady;
      delete win.pacmanError;
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const handleInteraction = useCallback(() => {
    if (gameReady && !gameStarted) {
      const win = getPacmanWindow();
      if (win.Module?._start_game) {
        win.Module._start_game();
        setGameStarted(true);
      }
    }
  }, [gameReady, gameStarted]);

  // Handle keyboard interaction
  useEffect(() => {
    if (!gameReady || gameStarted) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      handleInteraction();
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [gameReady, gameStarted, handleInteraction]);

  return (
    <div className="flex justify-center items-center h-full pt-4">
      <div
        className="relative block aspect-[5/6]"
        style={{
          height: "min(calc(100vh - 96px), calc((100vw - 32px) * 6 / 5))",
        }}
        onClick={handleInteraction}
      >
        <canvas id="canvas" className="w-full h-full" />

        {/* Loading overlay - CSS animation continues during main thread blocking */}
        {loadingVisible && (
          <div
            className="absolute inset-0 flex flex-col items-center justify-center bg-black/80 transition-opacity"
            style={{
              transitionDuration: `${LOADING_FADE_DURATION}ms`,
              opacity: gameReady ? 0 : 1,
            }}
          >
            {loadError ? (
              <>
                <div className="error-indicator" />
                <span className="text-red-500 text-2xl mt-4 font-semibold">
                  {loadError.type === "timeout"
                    ? "Loading timed out"
                    : loadError.type === "script"
                      ? "Failed to load"
                      : "Error occurred"}
                </span>
                <span className="text-gray-400 text-sm mt-2 max-w-xs text-center">
                  {loadError.type === "timeout"
                    ? "The game took too long to load. Please refresh the page."
                    : loadError.type === "script"
                      ? "Could not load game files. Check your connection and refresh."
                      : loadError.message}
                </span>
                <button
                  onClick={() => window.location.reload()}
                  className="mt-4 px-4 py-2 bg-yellow-400 text-black font-semibold rounded hover:bg-yellow-300 transition-colors"
                >
                  Reload
                </button>
              </>
            ) : (
              <>
                <div className="loading-spinner" />
                <span className="text-yellow-400 text-2xl mt-4">Loading...</span>
              </>
            )}
          </div>
        )}

        {/* Click to Start overlay */}
        {gameReady && !gameStarted && (
          <div className="absolute inset-0 flex items-center justify-center bg-black/60 cursor-pointer">
            <span className="text-yellow-400 text-5xl font-bold">Click to Start</span>
          </div>
        )}
      </div>
    </div>
  );
}
