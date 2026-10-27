#!/usr/bin/env node
// Collect a platform's release assets under their published names, from a
// finished `tauri build` and `cargo build --release -p asten-api`.
//
//   node scripts/release-assets.mjs --platform windows-x64 --out release-assets
//   node scripts/release-assets.mjs --platform linux-x64 --out release-assets [--target <dir>]
//
// Windows: the NSIS installer (and its updater signature), a portable zip of
// the desktop executable, and asten-api in a zip. Linux: the .deb, the
// AppImage (and its signature), and asten-api in a .tar.gz. The version comes
// from the Tauri config.
import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const args = Object.fromEntries(
  process.argv
    .slice(2)
    .join(" ")
    .split(/\s*--/)
    .filter(Boolean)
    .map((pair) => pair.split(/\s+/)),
);
const platform = args.platform;
const out = resolve(args.out ?? "release-assets");
const target = resolve(args.target ?? "target");
const root = resolve(import.meta.dirname, "..");
const version = JSON.parse(
  readFileSync(join(root, "apps/desktop/src-tauri/tauri.conf.json"), "utf8"),
).version;
mkdirSync(out, { recursive: true });

const release = join(target, "release");
const bundle = join(release, "bundle");

function only(dir, test) {
  const found = existsSync(dir)
    ? readdirSync(dir).filter(test).filter((file) => file.includes(`_${version}_`))
    : [];
  if (found.length !== 1)
    throw new Error(`expected one match in ${dir}, found ${found.join(", ") || "none"}`);
  return join(dir, found[0]);
}

function copy(from, name) {
  copyFileSync(from, join(out, name));
  console.log(name);
}

function runtimeNote(windows) {
  const run = windows
    ? '  PowerShell:  $env:ASTEN_SERVE_DESK="desk"; .\\asten-api.exe'
    : "  ASTEN_SERVE_DESK=desk ./asten-api";
  return `asten-api ${version}: the Asten engine and HTTP API without the desktop window,
with the browser desk beside it in desk/.

${run}

Then open http://127.0.0.1:4317 and type the six-digit pairing code the API
prints. It listens on loopback only and keeps its SQLite file in
data/asten.db (ASTEN_DATABASE_PATH changes that). The Word and Docs guests
pair with it the same way. Environment variables: see the README at
https://github.com/ArefinAlter/asten. This build is unsigned.
`;
}

const desk = join(root, "apps/web/dist");

/** A temporary folder with the given files, plus the desk for the runtime. */
function stage(files, { withDesk = false, windows = false } = {}) {
  const dir = mkdtempSync(join(tmpdir(), "asten-assets-"));
  for (const [from, name] of files) copyFileSync(from, join(dir, name));
  writeFileSync(join(dir, "README.txt"), runtimeNote(windows));
  if (withDesk) {
    if (!existsSync(join(desk, "index.html")))
      throw new Error("build the desk first: pnpm --filter @asten/web build");
    cpSync(desk, join(dir, "desk"), { recursive: true });
  }
  return dir;
}

if (platform === "windows-x64") {
  const nsis = join(bundle, "nsis");
  const setup = only(nsis, (f) => f.endsWith("-setup.exe"));
  copy(setup, `Asten_${version}_x64-setup.exe`);
  if (existsSync(`${setup}.sig`)) copy(`${setup}.sig`, `Asten_${version}_x64-setup.exe.sig`);
  // bsdtar, which writes zip archives with -a; Git's GNU tar does not.
  const tar = "C:/Windows/System32/tar.exe";
  const portable = stage([[join(release, "asten-desktop.exe"), "Asten.exe"]]);
  writeFileSync(
    join(portable, "README.txt"),
    `Asten ${version}, portable. Run Asten.exe. It needs the Microsoft Edge WebView2 runtime (part of Windows 10 and 11). Data lives in %APPDATA%\\com.asten.app. This build is unsigned: SmartScreen warns before the first run.\n`,
  );
  const portableZip = join(out, `Asten_${version}_windows_x64_portable.zip`);
  execFileSync(tar, ["-a", "-c", "-f", portableZip, "-C", portable, "Asten.exe", "README.txt"]);
  console.log(`Asten_${version}_windows_x64_portable.zip`);
  const runtime = stage([[join(release, "asten-api.exe"), "asten-api.exe"]], {
    withDesk: true,
    windows: true,
  });
  const runtimeZip = join(out, `asten-api_${version}_windows_x64.zip`);
  execFileSync(tar, ["-a", "-c", "-f", runtimeZip, "-C", runtime, "asten-api.exe", "README.txt", "desk"]);
  console.log(`asten-api_${version}_windows_x64.zip`);
  rmSync(portable, { recursive: true, force: true });
  rmSync(runtime, { recursive: true, force: true });
} else if (platform === "linux-x64") {
  copy(only(join(bundle, "deb"), (f) => f.endsWith(".deb")), `Asten_${version}_amd64.deb`);
  const appimage = only(join(bundle, "appimage"), (f) => f.endsWith(".AppImage"));
  copy(appimage, `Asten_${version}_amd64.AppImage`);
  if (existsSync(`${appimage}.sig`)) copy(`${appimage}.sig`, `Asten_${version}_amd64.AppImage.sig`);
  const runtime = stage([[join(release, "asten-api"), "asten-api"]], { withDesk: true });
  execFileSync("chmod", ["+x", join(runtime, "asten-api")]);
  execFileSync("tar", [
    "-czf",
    join(out, `asten-api_${version}_linux_x64.tar.gz`),
    "-C",
    runtime,
    "asten-api",
    "README.txt",
    "desk",
  ]);
  console.log(`asten-api_${version}_linux_x64.tar.gz`);
  rmSync(runtime, { recursive: true, force: true });
} else {
  throw new Error("--platform must be windows-x64 or linux-x64");
}
