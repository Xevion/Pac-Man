import type { OnPageTransitionEndAsync } from "vike/types";
import { getPacmanWindow } from "@/lib/pacman";

export const onPageTransitionEnd: OnPageTransitionEndAsync = async (
  pageContext
) => {
  console.log("Page transition end");
  document.querySelector("body")?.classList.remove("page-is-transitioning");

  // Restart the game loop when returning to the game page
  if (pageContext.urlPathname === "/") {
    const win = getPacmanWindow();
    const module = win.Module;

    if (module?._restart_game) {
      const canvas = document.getElementById("canvas") as HTMLCanvasElement | null;
      if (!canvas) {
        console.error("Canvas element not found during game restart");
        return;
      }

      // Update canvas reference BEFORE restart - App::new() will read from Module.canvas
      module.canvas = canvas;
      // SDL2's Emscripten backend reads this for canvas lookup
      win.SDL_CANVAS_ID = "#canvas";

      try {
        console.log("Restarting game with fresh App instance");
        module._restart_game();
      } catch (error) {
        console.error("Failed to restart game:", error);
      }
    }
  }
};
