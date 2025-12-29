import type { OnPageTransitionStartAsync } from "vike/types";
import { getPacmanWindow } from "@/lib/pacman";

export const onPageTransitionStart: OnPageTransitionStartAsync = async () => {
  console.log("Page transition start");
  document.querySelector("body")?.classList.add("page-is-transitioning");

  // Stop the game loop when navigating away from the game page
  const win = getPacmanWindow();
  if (win.Module?._stop_game) {
    console.log("Stopping game loop for page transition");
    win.Module._stop_game();
  }
};
