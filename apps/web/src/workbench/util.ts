import type { CompileArtifact } from "@asten/contracts";

/** Save a compiled file through the browser; a local blob, no network. */
export function downloadArtifact(artifact: CompileArtifact) {
  const bytes =
    artifact.encoding === "base64"
      ? Uint8Array.from(atob(artifact.body), (c) => c.charCodeAt(0))
      : new TextEncoder().encode(artifact.body);
  const url = URL.createObjectURL(new Blob([bytes], { type: artifact.mime }));
  const link = document.createElement("a");
  link.href = url;
  link.download = artifact.filename;
  document.body.append(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 2000);
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
