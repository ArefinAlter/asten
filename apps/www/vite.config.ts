import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The client bundle is built here; `scripts/prerender.mjs` then renders every
// route to static HTML with the server bundle, and writes the crawl files.
export default defineConfig(({ isSsrBuild }) => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 4320,
    strictPort: true,
  },
  build: {
    target: "es2022",
    assetsInlineLimit: 0,
    // Static files belong to the client build only.
    copyPublicDir: !isSsrBuild,
  },
}));
