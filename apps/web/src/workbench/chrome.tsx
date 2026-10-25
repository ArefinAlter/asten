/**
 * The window frame on the desktop: the title bar is the page's top row, with
 * minimize, maximize and close drawn at its right (as T3 Code and Ogun Studio
 * do), shown only when the window has no native frame.
 */
import { useCallback, useEffect, useState } from "react";
import { isTauri } from "../lib/asten-client";

type AppWindow = {
  minimize(): Promise<void>;
  toggleMaximize(): Promise<void>;
  close(): Promise<void>;
  isMaximized(): Promise<boolean>;
  isDecorated(): Promise<boolean>;
};

async function currentWindow(): Promise<AppWindow | null> {
  if (!isTauri()) return null;
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    return getCurrentWindow() as unknown as AppWindow;
  } catch {
    return null;
  }
}

/** Whether the page must draw its own window controls (an undecorated desktop window). */
export function useCustomChrome(): boolean {
  const [custom, setCustom] = useState(false);
  useEffect(() => {
    void currentWindow().then((w) =>
      w?.isDecorated().then(
        (d) => setCustom(!d),
        () => undefined,
      ),
    );
  }, []);
  return custom;
}

function Glyph({ name }: { name: "min" | "max" | "restore" | "close" }) {
  return (
    <svg
      viewBox="0 0 10 10"
      width="10"
      height="10"
      fill="none"
      stroke="currentColor"
      strokeWidth="1"
      aria-hidden="true"
    >
      {name === "min" && <path d="M0 5.5h10" />}
      {name === "max" && <rect x="0.5" y="0.5" width="9" height="9" rx="1" />}
      {name === "restore" && (
        <>
          <rect x="0.5" y="2.5" width="7" height="7" rx="1" />
          <path d="M2.5 2.5v-1a1 1 0 0 1 1-1h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-1" />
        </>
      )}
      {name === "close" && <path d="M0.5 0.5l9 9M9.5 0.5l-9 9" />}
    </svg>
  );
}

export function WindowControls() {
  const [maximized, setMaximized] = useState(false);
  const sync = useCallback(() => {
    void currentWindow().then((w) =>
      w?.isMaximized().then(setMaximized, () => undefined),
    );
  }, []);
  useEffect(() => {
    sync();
    window.addEventListener("resize", sync);
    return () => window.removeEventListener("resize", sync);
  }, [sync]);
  const act = (f: (w: AppWindow) => Promise<void>) => () => {
    void currentWindow().then((w) => (w ? f(w) : undefined));
  };
  return (
    <div className="winctl" role="group" aria-label="Window controls">
      <button
        type="button"
        aria-label="Minimize"
        title="Minimize"
        onClick={act((w) => w.minimize())}
      >
        <Glyph name="min" />
      </button>
      <button
        type="button"
        aria-label={maximized ? "Restore" : "Maximize"}
        title={maximized ? "Restore" : "Maximize"}
        onClick={act((w) => w.toggleMaximize())}
      >
        <Glyph name={maximized ? "restore" : "max"} />
      </button>
      <button
        type="button"
        className="close"
        aria-label="Close"
        title="Close"
        onClick={act((w) => w.close())}
      >
        <Glyph name="close" />
      </button>
    </div>
  );
}
