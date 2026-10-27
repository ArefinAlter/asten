/**
 * Saving and picking files. In the desktop app the system's own dialogs choose
 * the place and a command in the shell writes the file (the page has no
 * file-system permission); in a browser a local blob is downloaded.
 */
import { isTauri } from "../lib/asten-client";
import { log } from "./output";

function toBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunk = 0x8000;
  for (let index = 0; index < bytes.length; index += chunk)
    binary += String.fromCharCode(...bytes.subarray(index, index + chunk));
  return btoa(binary);
}

function extensionOf(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
}

/** Save bytes under a suggested name. Resolves to the path chosen, or null if none. */
export async function saveBytes(
  filename: string,
  bytes: Uint8Array,
  mime = "application/octet-stream",
): Promise<string | null> {
  if (isTauri()) {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const extension = extensionOf(filename);
    const path = await save({
      defaultPath: filename,
      filters: extension
        ? [{ name: extension.toUpperCase(), extensions: [extension] }]
        : [],
    });
    if (!path) return null;
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("save_file", { path, base64: toBase64(bytes) });
    log("Desk", `Saved ${path}`);
    return path;
  }
  const url = URL.createObjectURL(
    new Blob([bytes.slice().buffer], { type: mime }),
  );
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.append(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 2000);
  return filename;
}

/** Ask the writer for a file (desktop: the system dialog; elsewhere: null, the page uses an input). */
export async function pickFile(
  extensions: string[],
  name: string,
): Promise<{ path: string } | null> {
  if (!isTauri()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const path = await open({
    multiple: false,
    directory: false,
    filters: [{ name, extensions }],
  });
  return typeof path === "string" ? { path } : null;
}
