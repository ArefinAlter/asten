import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@fontsource-variable/fraunces";
import "@fontsource-variable/literata";
import "@fontsource/ibm-plex-mono/400.css";
import "@fontsource/ibm-plex-mono/500.css";
import "@asten/brand/tokens.css";
import "./asten.css";
import "./workbench/workbench.css";
import { applyStoredTheme } from "./brand/ThemeToggle";
import { installLocalTransport } from "./lib/asten-client";
import { hydrateHostedState, isHosted } from "./lib/storage";
import { applyPrefs, getPrefs, reloadPrefs } from "./workbench/prefs";

applyStoredTheme();
applyPrefs(getPrefs());

/** The hosted pre-alpha copy runs on an in-page sample backend: no API, no network. */
const SAMPLE = import.meta.env.VITE_ASTEN_SAMPLE === "1";

// Installable web app: the shell is cached so a phone can reopen the desk
// without the network; the API is never cached. The desktop and mobile shells
// load from their bundle and need no worker; the hosted copy registers none.
if (
  import.meta.env.PROD &&
  !SAMPLE &&
  !isHosted() &&
  window.isSecureContext &&
  "serviceWorker" in navigator &&
  !("__TAURI_INTERNALS__" in window)
) {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/sw.js").catch(() => undefined);
  });
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 10_000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

async function start() {
  // The hosted pre-alpha keeps the desk's own state with the visitor's
  // sandbox; read it before the first render so nothing flickers back.
  if (isHosted()) {
    await hydrateHostedState();
    reloadPrefs();
    applyStoredTheme();
  }
  if (SAMPLE) {
    const { SampleBackend } = await import("./sample/backend");
    const backend = new SampleBackend();
    installLocalTransport({
      handle: (method, path, body) => backend.handle(method, path, body),
      subscribe: (onEvent) => backend.subscribe(onEvent),
    });
  }
  // The client is created when the desk's modules load, after the transport.
  const { default: App } = await import("./App.tsx");
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    </StrictMode>,
  );
}

void start();
