import type { CompileArtifact } from "@asten/contracts";
import { saveBytes } from "./files";

/**
 * Save a compiled file. In the desktop app the writer picks the place in the
 * system's save dialog; in a browser the file is a local blob the browser
 * downloads. No network either way.
 */
export function downloadArtifact(artifact: CompileArtifact) {
  const bytes =
    artifact.encoding === "base64"
      ? Uint8Array.from(atob(artifact.body), (c) => c.charCodeAt(0))
      : new TextEncoder().encode(artifact.body);
  return saveBytes(artifact.filename, bytes, artifact.mime);
}

export function relativeTime(iso: string, now = Date.now()): string {
  const seconds = Math.round((now - Date.parse(iso)) / 1000);
  if (!Number.isFinite(seconds)) return "";
  if (seconds < 45) return "just now";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} min ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} h ago`;
  const days = Math.round(hours / 24);
  if (days < 30) return `${days} d ago`;
  return new Date(iso).toLocaleDateString("en-GB", {
    day: "numeric",
    month: "short",
    year: "numeric",
  });
}

/** Only links the page may follow: https, mailto, and the desk's own hash routes. */
export function safeHref(href: string): string | null {
  const trimmed = href.trim();
  if (/^https:\/\/[^\s]+$/i.test(trimmed)) return trimmed;
  if (/^mailto:[^\s]+$/i.test(trimmed)) return trimmed;
  if (/^#\/[^\s]*$/.test(trimmed)) return trimmed;
  return null;
}
