/**
 * The current Windows alpha release: the one source for the download page, the
 * calls to action and the structured data. File sizes come from the published
 * release assets; the SHA-256 of every file is in the release's SHA256SUMS.txt.
 */

export const release = {
  version: "0.1.0-alpha.1",
  tag: "v0.1.0-alpha.1",
  semver: "0.1.0-alpha.1",
  date: "2026-10-29",
  dateLabel: "29 October 2026",
  hostedUrl: "https://asten.103.191.50.32.sslip.io/",
  releaseUrl: "https://github.com/ArefinAlter/asten/releases/tag/v0.1.0-alpha.1",
  checksumsFile: "SHA256SUMS.txt",
  checksumsUrl:
    "https://github.com/ArefinAlter/asten/releases/download/v0.1.0-alpha.1/SHA256SUMS.txt",
} as const;

export type ReleasePlatform = "windows" | "linux";

export interface ReleaseFile {
  /** What the file is, in a few words. */
  kind: string;
  file: string;
  url: string;
  /** Size in bytes, from the published release. */
  bytes: number;
}

export interface PlatformDownloads {
  os: ReleasePlatform;
  name: string;
  /** One line: what the system needs. */
  requirement: string;
  primary: ReleaseFile;
  more: ReleaseFile[];
}

const base = `https://github.com/ArefinAlter/asten/releases/download/${release.tag}`;
const file = (kind: string, name: string, bytes: number, releaseBase = base): ReleaseFile => ({ kind, file: name, url: `${releaseBase}/${name}`, bytes });
const previousLinux = "https://github.com/ArefinAlter/asten/releases/download/v0.0.1c";

export const platformDownloads: PlatformDownloads[] = [
  {
    os: "windows",
    name: "Windows · 0.1.0-alpha.1",
    requirement: "Windows 10 or 11, x64. Uses the WebView2 runtime; the installer fetches it if it is missing.",
    primary: file("Installer", "Asten_0.1.0-alpha.1_x64-setup.exe", 8979906),
    more: [
      file("Portable .zip (Asten.exe)", "Asten_0.1.0-alpha.1_windows_x64_portable.zip", 12364797),
      file("asten-api, the local API without the window, .zip", "asten-api_0.1.0-alpha.1_windows_x64.zip", 7236536),
    ],
  },
  {
    os: "linux",
    name: "Linux · previous 0.0.1c",
    requirement: "x64, glibc 2.35 and WebKitGTK 4.1. AppImage on most distributions, .deb for Debian 12 and Ubuntu 22.04 or newer.",
    primary: file("Desktop app", "Asten_0.0.1-beta.1_amd64.AppImage", 90151416, previousLinux),
    more: [
      file("Debian package, .deb", "Asten_0.0.1-beta.1_amd64.deb", 13054964, previousLinux),
      file("asten-api, the local API without the window, .tar.gz", "asten-api_0.0.1-beta.1_linux_x64.tar.gz", 6929462, previousLinux),
    ],
  },
];

/** Size in decimal megabytes: 7655059 -> "7.7 MB". */
export function formatBytes(bytes: number): string {
  if (!bytes) return "";
  return bytes >= 1e6 ? `${(bytes / 1e6).toFixed(1)} MB` : `${Math.max(1, Math.round(bytes / 1e3))} KB`;
}
