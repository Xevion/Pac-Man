import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import vike from "vike/plugin";
import { defineConfig, Plugin } from "vite";
import path from "path";
import { execSync } from "child_process";

/**
 * Vite plugin that injects the Pacman version hash at build time.
 * Uses git commit hash in production/dev, falls back to timestamp if git unavailable.
 */
function pacmanVersionPlugin(): Plugin {
  let version: string;

  function getVersion(mode: string): string {
    // Development mode uses fixed "dev" string
    if (mode === "development") {
      return "dev";
    }

    // Try to get git commit hash
    try {
      const hash = execSync("git rev-parse --short HEAD", {
        encoding: "utf8",
        stdio: ["pipe", "pipe", "pipe"],
      }).trim();
      
      if (hash) {
        return hash;
      }
    } catch {
      // Git not available or command failed
    }

    // Fallback to timestamp
    return Date.now().toString(36);
  }

  return {
    name: "pacman-version",
    config(_, { mode }) {
      version = getVersion(mode);
      console.log(`[pacman-version] Using version: ${version}`);

      return {
        define: {
          "import.meta.env.VITE_PACMAN_VERSION": JSON.stringify(version),
        },
      };
    },
  };
}

export default defineConfig({
  plugins: [pacmanVersionPlugin(), vike(), react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "."),
    },
    dedupe: ["react", "react-dom"],
  },
  build: {
    target: "es2022",
  },
  server: {
    // Proxy API requests to the backend server during local development
    // In production, both frontend and API are served from the same origin
    proxy: {
      "/api": {
        target: process.env.VITE_API_TARGET || "http://localhost:3001",
        changeOrigin: true,
      },
    },
  },
});
