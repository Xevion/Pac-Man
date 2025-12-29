import type { Config } from "vike/types";

export default {
  prerender: true, // Generate static HTML for deployment
  ssr: false, // Force client-side only rendering
} satisfies Config;
