import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";

/**
 * The two hosted builds share one page shape: no inline script (strict CSP),
 * no external anything, noindex, no manifest or service worker.
 *
 * - `--mode hosted`: the hosted pre-alpha at its own origin, against the real
 *   API through the pre-alpha gateway (`services/asten-prealpha`).
 * - `--mode site`: the offline copy on the in-page sample backend, kept as a
 *   build option under /app/ (offline builds and tests).
 */
function hostedCopy(mode: "site" | "hosted"): Plugin {
  const description =
    mode === "hosted"
      ? "Asten, pre-alpha release: the writing workbench with its real engine, in a private sandbox that is deleted after 24 hours."
      : "Asten, pre-alpha release: the writing workbench on a sample project. Runs in the browser with no backend.";
  return {
    name: "asten-hosted-copy",
    transformIndexHtml(html) {
      const page = html
        .replace(/<script>[\s\S]*?<\/script>\s*/, "")
        .replace(/\s*<link rel="manifest"[^>]*>/, "")
        .replace(
          /\s*<meta name="(mobile-web-app-capable|apple-mobile-web-app-[a-z-]+)"[^>]*>/g,
          "",
        )
        .replace(/\s*<link rel="apple-touch-icon"[^>]*>/, "")
        .replace(
          /<meta\s+name="description"[\s\S]*?\/>/,
          `<meta name="robots" content="noindex, nofollow" />\n    <meta name="description" content="${description}" />`,
        )
        .replace("<title>Asten</title>", "<title>Asten Workbench</title>");
      return mode === "site"
        ? page.replace(/href="\/icons\//g, 'href="/app/icons/')
        : page;
    },
  };
}

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [
    react(),
    ...(mode === "site" || mode === "hosted" ? [hostedCopy(mode)] : []),
  ],
  base: mode === "site" ? "/app/" : "/",
  clearScreen: false,
  build:
    mode === "site" || mode === "hosted"
      ? {
          outDir: mode === "site" ? "dist-site" : "dist-hosted",
          emptyOutDir: true,
          assetsInlineLimit: 0,
        }
      : undefined,
  server: {
    port: 1420,
    strictPort: true,
    // `dev:hosted` talks to a local asten-prealpha gateway (4318), which
    // hosts the API per visitor; every other mode to the API itself.
    proxy:
      mode === "hosted"
        ? {
            "/api": "http://127.0.0.1:4318",
            "/svc": "http://127.0.0.1:4318",
          }
        : {
            "/api": "http://127.0.0.1:4317",
            "/health": "http://127.0.0.1:4317",
          },
  },
}));
