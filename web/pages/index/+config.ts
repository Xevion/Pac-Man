import type { Config } from "vike/types";

// Disable SSR for the game page since Emscripten requires a browser environment
// Prerender enabled to generate index.html for deployment, while ssr: false ensures client-side WASM loading
export default {
  prerender: true, // Generate static HTML shell for deployment
  ssr: false, // Force client-side only rendering (required for Emscripten/WASM)
} satisfies Config;
