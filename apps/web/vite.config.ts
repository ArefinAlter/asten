import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";

/**
 * The hosted pre-alpha copy (`--mode site`): served by the public site under
 * /app/, on sample data, under the site's CSP (no inline script, no external
 * anything), marked noindex, with no manifest or service worker.
 */
function hostedCopy(): Plugin {
  return {
    name: "asten-hosted-copy",
    transformIndexHtml(html) {
      return html
        .replace(/<script>[\s\S]*?<\/script>\s*/, "")
        .replace(/\s*<link rel="manifest"[^>]*>/, "")
        .replace(
          /\s*<meta name="(mobile-web-app-capable|apple-mobile-web-app-[a-z-]+)"[^>]*>/g,
          "",
        )
        .replace(/\s*<link rel="apple-touch-icon"[^>]*>/, "")
        .replace(/href="\/icons\//g, 'href="/app/icons/')
        .replace(
          /<meta\s+name="description"[\s\S]*?\/>/,
          '<meta name="robots" content="noindex, nofollow" />\n    <meta name="description" content="Asten, pre-alpha release: the writing workbench on a sample project. Runs in the browser with no backend." />',
        )
        .replace("<title>Asten</title>", "<title>Asten Workbench</title>");
    },
  };
}

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [react(), ...(mode === "site" ? [hostedCopy()] : [])],
  base: mode === "site" ? "/app/" : "/",
  clearScreen: false,
  build:
    mode === "site"
      ? { outDir: "dist-site", emptyOutDir: true, assetsInlineLimit: 0 }
      : undefined,
  server: {
    port: 1420,
    strictPort: true,
    proxy: {
      "/api": "http://127.0.0.1:4317",
      "/health": "http://127.0.0.1:4317",
    },
  },
}));
