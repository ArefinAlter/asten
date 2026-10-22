import type { Update } from "@tauri-apps/plugin-updater";
import { useState } from "react";
import { isTauri } from "../lib/asten-client";

/** One row of "This desk": a label and a path the writer can copy. */
export function DeskPath({
  label,
  path,
  missing,
}: {
  label: string;
  path?: string | null;
  missing?: string;
}) {
  return (
    <li className="row">
      <strong>{label}</strong>
      {path ? (
        <small className="mono desk-path" title={path}>
          {path}
          <button
            type="button"
            className="linkish"
            onClick={() => {
              void navigator.clipboard?.writeText(path).catch(() => undefined);
            }}
          >
            copy
          </button>
        </small>
      ) : (
        <small className="hint">{missing}</small>
      )}
    </li>
  );
}

type Phase =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "latest" }
  | { kind: "available"; update: Update }
  | { kind: "installing"; received: number; total: number | null }
  | { kind: "failed"; message: string };

/**
 * The update check, in the desktop shell only. It asks the release page when
 * the writer presses the button and never on its own; a found update installs
 * and relaunches only after a second, explicit press.
 */
export function UpdateCheck({
  host,
  version,
}: {
  host: string;
  version: string;
}) {
  const [phase, setPhase] = useState<Phase>({ kind: "idle" });
  // Phones update through their store; the updater plugin is desktop-only.
  if (!isTauri() || /^(android|ios)\b/.test(host)) return null;

  async function check() {
    setPhase({ kind: "checking" });
    try {
      const updater = await import("@tauri-apps/plugin-updater");
      const update = await updater.check();
      setPhase(update ? { kind: "available", update } : { kind: "latest" });
    } catch (error) {
      setPhase({ kind: "failed", message: describe(error) });
    }
  }

  async function install(update: Update) {
    let received = 0;
    let total: number | null = null;
    setPhase({ kind: "installing", received, total });
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? null;
        } else if (event.event === "Progress") {
          received += event.data.chunkLength;
        }
        setPhase({ kind: "installing", received, total });
      });
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (error) {
      setPhase({ kind: "failed", message: describe(error) });
    }
  }

  return (
    <li className="row">
      <strong>Updates</strong>
      <small className="desk-path">
        {phase.kind === "idle" && (
          <button
            type="button"
            className="linkish"
            onClick={() => void check()}
          >
            Check for updates
          </button>
        )}
        {phase.kind === "checking" && (
          <span className="hint">Asking the release page…</span>
        )}
        {phase.kind === "latest" && (
          <>
            <span>You have the latest ({version}).</span>{" "}
            <button
              type="button"
              className="linkish"
              onClick={() => void check()}
            >
              check again
            </button>
          </>
        )}
        {phase.kind === "available" && (
          <>
            <span>Version {phase.update.version} available.</span>{" "}
            <button
              type="button"
              className="linkish"
              onClick={() => void install(phase.update)}
            >
              Install and relaunch
            </button>
          </>
        )}
        {phase.kind === "installing" && (
          <span className="hint">
            Installing… {progress(phase.received, phase.total)}
          </span>
        )}
        {phase.kind === "failed" && (
          <>
            <span className="hint">Could not check: {phase.message}</span>{" "}
            <button
              type="button"
              className="linkish"
              onClick={() => void check()}
            >
              try again
            </button>
          </>
        )}
      </small>
    </li>
  );
}

function progress(received: number, total: number | null): string {
  if (total && total > 0) {
    return `${Math.min(100, Math.round((received / total) * 100))}%`;
  }
  return received > 0 ? `${Math.round(received / 1024 / 1024)} MB` : "";
}

function describe(error: unknown): string {
  if (error instanceof Error) return error.message;
  return typeof error === "string" ? error : "something went wrong";
}
