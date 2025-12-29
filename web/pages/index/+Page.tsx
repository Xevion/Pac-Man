import { useCallback, useEffect, useState } from "react";
import { getPacmanWindow } from "@/lib/pacman";

export default function Page() {
  const [gameReady, setGameReady] = useState(false);
  const [gameStarted, setGameStarted] = useState(false);

  useEffect(() => {
    const win = getPacmanWindow();
    
    // Always set up the ready callback (restart_game will call it too)
    win.pacmanReady = () => {
      setGameReady(true);
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
      return;
    }

    win.Module = {
      canvas,
      locateFile: (path: string) => {
        return path.startsWith("/") ? path : `/${path}`;
      },
      preRun: [],
    };

    const script = document.createElement("script");
    script.src = "/pacman.js";
    script.async = false;
    document.body.appendChild(script);

    return () => {
      delete win.pacmanReady;
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
    <div className="mt-4 flex justify-center h-[calc(100vh-120px)]">
      <div
        className="relative block border-1 border-yellow-400/50 aspect-[5/6] h-[min(calc(100vh-120px),_calc(95vw_*_6/5))] w-auto"
        style={{
          boxShadow:
            "0 0 12px rgba(250, 204, 21, 0.35), 0 0 2px rgba(255, 255, 255, 0.25)",
        }}
        onClick={handleInteraction}
      >
        <canvas id="canvas" className="w-full h-full" />

        {/* Click to Start overlay */}
        {gameReady && !gameStarted && (
          <div className="absolute inset-0 flex items-center justify-center bg-black/60 cursor-pointer">
            <span className="text-yellow-400 text-5xl font-bold">
              Click to Start
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
