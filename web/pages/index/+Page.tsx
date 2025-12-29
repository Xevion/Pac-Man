import { useCallback, useEffect, useState } from "react";

export default function Page() {
  const [gameReady, setGameReady] = useState(false);
  const [gameStarted, setGameStarted] = useState(false);

  useEffect(() => {
    // Set up callback for when WASM signals it's ready
    (window as any).pacmanReady = () => {
      setGameReady(true);
    };

    if (!(window as any).Module) {
      const canvas = document.getElementById("canvas");

      (window as any).Module = {
        canvas: canvas,
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
        script.remove();
        delete (window as any).pacmanReady;
      };
    }
  }, []);

  const handleInteraction = useCallback(() => {
    if (gameReady && !gameStarted) {
      // Call the exported Rust function to start the game
      const module = (window as any).Module;
      if (module && module._start_game) {
        module._start_game();
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
