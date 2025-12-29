import type { OnPageTransitionStartAsync } from "vike/types";
import { getPacmanWindow } from "@/lib/pacman";
import { setPendingNavigation } from "@/lib/navigation";

// Must match --transition-duration in layouts/tailwind.css
const TRANSITION_DURATION = 200;

export const onPageTransitionStart: OnPageTransitionStartAsync = async (pageContext) => {
  console.log("Page transition start");
  setPendingNavigation(pageContext.urlPathname);
  document.querySelector("body")?.classList.add("page-is-transitioning");

  // Only stop the game when navigating AWAY FROM the game page
  // Don't stop when navigating between other pages (e.g., /leaderboard <-> /download)
  if (window.location.pathname === "/") {
    const win = getPacmanWindow();
    if (win.Module?._stop_game) {
      try {
        console.log("Stopping game loop for page transition");
        win.Module._stop_game();
      } catch (error) {
        console.warn("Failed to stop game (game may have already crashed):", error);
      }
    }
  }

  // Wait for fade-out animation to complete before page content changes
  await new Promise((resolve) => setTimeout(resolve, TRANSITION_DURATION));
};
