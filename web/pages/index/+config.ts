import type { Config } from "vike/types";

// Disable SSR for the game page since Emscripten requires a browser environment
export default {
  prerender: false, // Don't pre-render during build
  ssr: false, // Force client-side only rendering
} satisfies Config;
