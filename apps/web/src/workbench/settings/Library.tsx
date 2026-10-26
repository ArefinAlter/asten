import { useCallback, useEffect, useState } from "react";
import { isTauri } from "../../lib/asten-client";
import { Card, SettingRow } from "./parts";

interface BackupInfo {
  name: string;
  bytes: number;
  modified: string;
  kind: "automatic" | "before_restore";
}

interface LibraryInfo {
  database_path: string;
  database_bytes: number;
  backups_path: string;
  backups: BackupInfo[];
  restore_pending: boolean;
  last_restore: { ok: boolean; message: string; at: string } | null;
}

const size = (bytes: number) =>
  bytes > 1024 * 1024
    ? `${(bytes / 1024 / 1024).toFixed(1)} MB`
    : `${Math.max(1, Math.round(bytes / 1024))} KB`;

async function call<T>(command: string, args?: Record<string, unknown>) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

function describe(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Something went wrong";
}

/**
 * The library as a file (desktop): back it up, keep a copy somewhere else,
 * and put a copy back. Putting one back takes effect at the next start, and
 * the library it replaces is kept.
 */
export function Library() {
  const [info, setInfo] = useState<LibraryInfo>();
  const [message, setMessage] = useState<string>();
  const [busy, setBusy] = useState(false);
  const refresh = useCallback(async () => {
    try {
      setInfo(await call<LibraryInfo>("library_info"));
    } catch {
      setInfo(undefined);
    }
  }, []);
  useEffect(() => {
    if (isTauri()) void refresh();
  }, [refresh]);
  if (!isTauri() || !info) return null;

  const run = async (work: () => Promise<string | undefined>) => {
    setBusy(true);
    setMessage(undefined);
    try {
      setMessage(await work());
    } catch (error) {
      setMessage(describe(error));
    } finally {
      setBusy(false);
      await refresh();
    }
  };

  const restore = async (
    source: { kind: "backup"; name: string } | { kind: "file"; path: string },
    label: string,
  ) => {
    const { confirm } = await import("@tauri-apps/plugin-dialog");
    const go = await confirm(
      `Replace your library with ${label}? Asten restarts to do it. The library it replaces is kept in the backups folder, so nothing is lost.`,
      {
        title: "Restore the library",
        kind: "warning",
        okLabel: "Restore and restart",
      },
    );
    if (!go) return undefined;
    await call("library_restore", { source });
    const { relaunch } = await import("@tauri-apps/plugin-process");
    await relaunch();
    return "Restarting…";
  };

  return (
    <Card title="Your library">
      {info.last_restore && (
        <SettingRow
          title={info.last_restore.ok ? "Restored" : "Not restored"}
          note={info.last_restore.message}
        >
          <button
            type="button"
            className="wb-btn wb-btn--small"
            onClick={() => void call("library_dismiss_report").then(refresh)}
          >
            Dismiss
          </button>
        </SettingRow>
      )}
      <SettingRow
        title="The file"
        note={`${info.database_path} · ${size(info.database_bytes)}. Every project, page, fact and version lives in this one file.`}
      />
      <SettingRow
        title="Back up now"
        note={`A consistent copy in ${info.backups_path}. Asten also takes one when it opens (at most every six hours) and keeps the newest seven.`}
      >
        <button
          type="button"
          className="wb-btn"
          disabled={busy}
          onClick={() =>
            void run(async () => {
              const made = await call<BackupInfo>("library_backup_now");
              return `Backed up as ${made.name} (${size(made.bytes)}).`;
            })
          }
        >
          Back up now
        </button>
      </SettingRow>
      <SettingRow
        title="Keep a copy somewhere else"
        note="Writes the whole library, with its history, to a file you choose: a USB stick, a synced folder."
      >
        <button
          type="button"
          className="wb-btn"
          disabled={busy}
          onClick={() =>
            void run(async () => {
              const { save } = await import("@tauri-apps/plugin-dialog");
              const path = await save({
                defaultPath: "asten-library.db",
                filters: [{ name: "Asten library", extensions: ["db"] }],
              });
              if (!path) return undefined;
              const bytes = await call<number>("library_export_copy", { path });
              return `Saved ${size(bytes)} to ${path}.`;
            })
          }
        >
          Save a copy…
        </button>
      </SettingRow>
      <SettingRow
        title="Restore from a file"
        note="Choose a library you saved. It is checked first; if it is not a library Asten can open, nothing changes."
      >
        <button
          type="button"
          className="wb-btn"
          disabled={busy}
          onClick={() =>
            void run(async () => {
              const { open } = await import("@tauri-apps/plugin-dialog");
              const path = await open({
                multiple: false,
                directory: false,
                filters: [{ name: "Asten library", extensions: ["db"] }],
              });
              if (typeof path !== "string") return undefined;
              return restore({ kind: "file", path }, `the file ${path}`);
            })
          }
        >
          Restore from a file…
        </button>
      </SettingRow>
      {info.backups.map((backup) => (
        <SettingRow
          key={backup.name}
          title={backup.name}
          note={`${new Date(backup.modified).toLocaleString()} · ${size(backup.bytes)}${backup.kind === "before_restore" ? " · kept when a restore replaced a library" : ""}`}
        >
          <button
            type="button"
            className="wb-btn wb-btn--small"
            disabled={busy}
            onClick={() =>
              void run(() =>
                restore({ kind: "backup", name: backup.name }, backup.name),
              )
            }
          >
            Restore
          </button>
        </SettingRow>
      ))}
      {message && (
        <p className="muted small" role="status">
          {message}
        </p>
      )}
    </Card>
  );
}
