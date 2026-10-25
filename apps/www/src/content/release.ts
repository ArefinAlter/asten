/**
 * The current pre-alpha release: the one source for the download page, the
 * calls to action and the structured data. Sizes and checksums are filled in
 * from the published release; an empty value is simply not shown.
 */

export const release = {
  version: "0.0.1a",
  tag: "v0.0.1a",
  semver: "0.0.1-alpha",
  date: "2026-10-25",
  dateLabel: "25 October 2026",
  hostedUrl: "https://asten.103.191.50.32.sslip.io/",
  releaseUrl: "https://github.com/ArefinAlter/asten/releases/tag/v0.0.1a",
  checksumsUrl:
    "https://github.com/ArefinAlter/asten/releases/download/v0.0.1a/SHA256SUMS.txt",
} as const;

export type ReleasePlatform = "windows" | "linux";

export interface ReleaseAsset {
  platform: ReleasePlatform;
  /** What the file is, in a few words. */
  label: string;
  file: string;
  url: string;
  /** Size in bytes; 0 until filled in from the published release. */
  bytes: number;
  /** Lower-case hex SHA-256; empty until filled in. */
  sha256: string;
  /** How to run it, in one line. */
  note: string;
  /** The desktop app (true) or the local API on its own (false). */
  primary: boolean;
}

const base = `https://github.com/ArefinAlter/asten/releases/download/${release.tag}`;

/** Sizes and SHA-256 of the published files (from SHA256SUMS.txt). */
const published: Record<string, { bytes: number; sha256: string }> = {
  "Asten_0.0.1-alpha_amd64.AppImage": {
    bytes: 88132088,
    sha256: "14c1a4a57f053f7151cf43feb406879dc5914ffba8b0b2a1859734c8731fc27d",
  },
  "Asten_0.0.1-alpha_amd64.deb": {
    bytes: 10749034,
    sha256: "8c500ab9f920ac624633db8dd5c6b4ac7ccb397b129a6b738aefcc6d61526743",
  },
  "Asten_0.0.1-alpha_windows_x64_portable.zip": {
    bytes: 10421418,
    sha256: "3c9982a3a98d6520699abdd313f9aa33d9c948484d75fe2cd2e3f4c651e33d03",
  },
  "Asten_0.0.1-alpha_x64-setup.exe": {
    bytes: 7655059,
    sha256: "092d3f132b64ae564128343fe1529877563292de9b51b8271d8dc51733763fd3",
  },
  "asten-api_0.0.1-alpha_linux_x64.tar.gz": {
    bytes: 6523357,
    sha256: "fa19f99042a53b3b2734becaa36a85e93709a6f49746b4a51291df744938b023",
  },
  "asten-api_0.0.1-alpha_windows_x64.zip": {
    bytes: 6785080,
    sha256: "0acc3b4738b74b3a6fe28c682432d95b6f677948a55bf2416a49c528091c7a56",
  },
};

function asset(
  platform: ReleasePlatform,
  label: string,
  file: string,
  note: string,
  primary: boolean,
): ReleaseAsset {
  const known = published[file];
  return {
    platform,
    label,
    file,
    url: `${base}/${file}`,
    bytes: known?.bytes ?? 0,
    sha256: known?.sha256 ?? "",
    note,
    primary,
  };
}

export const releaseAssets: ReleaseAsset[] = [
  asset(
    "windows",
    "Windows x64 installer",
    "Asten_0.0.1-alpha_x64-setup.exe",
    "Unsigned: Windows SmartScreen warns before the first run; choose More info, then Run anyway. Installs for your user only.",
    true,
  ),
  asset(
    "windows",
    "Windows x64 portable",
    "Asten_0.0.1-alpha_windows_x64_portable.zip",
    "Unzip and run Asten.exe. Needs the WebView2 runtime, which Windows 10 and 11 usually have. Unsigned, so SmartScreen warns.",
    true,
  ),
  asset(
    "linux",
    "Linux x64 AppImage",
    "Asten_0.0.1-alpha_amd64.AppImage",
    "chmod +x the AppImage, then run it. Built on Ubuntu 22.04; needs WebKitGTK 4.1.",
    true,
  ),
  asset(
    "linux",
    "Linux x64 Debian package",
    "Asten_0.0.1-alpha_amd64.deb",
    "sudo apt install ./Asten_0.0.1-alpha_amd64.deb (Ubuntu 22.04 or newer, Debian 12 or newer).",
    true,
  ),
  asset(
    "windows",
    "Local API for Windows x64",
    "asten-api_0.0.1-alpha_windows_x64.zip",
    "The engine and HTTP API without the window, with the browser desk beside it: run it, open 127.0.0.1:4317, type the pairing code it prints.",
    false,
  ),
  asset(
    "linux",
    "Local API for Linux x64",
    "asten-api_0.0.1-alpha_linux_x64.tar.gz",
    "The engine and HTTP API without the window, with the browser desk beside it: run it, open 127.0.0.1:4317, type the pairing code it prints.",
    false,
  ),
];

/** Human-readable size, or an empty string when unknown. */
export function formatBytes(bytes: number): string {
  if (!bytes) return "";
  const mb = bytes / (1024 * 1024);
  return mb >= 1
    ? `${mb.toFixed(1)} MB`
    : `${Math.max(1, Math.round(bytes / 1024))} KB`;
}
