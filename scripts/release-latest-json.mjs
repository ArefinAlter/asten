#!/usr/bin/env node
// Write the updater manifest the desktop app reads from the public site
// (apps/www/public/updates/latest.json), from a folder of release assets that
// includes the updater signatures (.sig) made with the release key.
//
//   node scripts/release-latest-json.mjs --assets release-assets --tag v0.0.1a
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const args = Object.fromEntries(
  process.argv
    .slice(2)
    .join(" ")
    .split(/\s*--/)
    .filter(Boolean)
    .map((pair) => pair.split(/\s+/)),
);
const root = resolve(import.meta.dirname, "..");
const assets = resolve(args.assets ?? "release-assets");
const tag = args.tag;
if (!tag) throw new Error("--tag is required, e.g. --tag v0.0.1a");
const out = resolve(args.out ?? join(root, "apps/www/public/updates/latest.json"));
const version = JSON.parse(
  readFileSync(join(root, "apps/desktop/src-tauri/tauri.conf.json"), "utf8"),
).version;
const base = `https://github.com/ArefinAlter/asten/releases/download/${tag}`;
const files = readdirSync(assets);

function entry(suffix) {
  const file = files.find((f) => f.endsWith(suffix));
  if (!file || !files.includes(`${file}.sig`)) return undefined;
  return {
    signature: readFileSync(join(assets, `${file}.sig`), "utf8").trim(),
    url: `${base}/${file}`,
  };
}

const platforms = {};
const windows = entry("_x64-setup.exe");
if (windows) platforms["windows-x86_64"] = windows;
const linux = entry("_amd64.AppImage");
if (linux) platforms["linux-x86_64"] = linux;
if (!Object.keys(platforms).length) throw new Error(`no signed assets in ${assets}`);

const manifest = {
  version,
  notes: `Asten ${tag.replace(/^v/, "")} (pre-alpha). Release notes: https://github.com/ArefinAlter/asten/releases/tag/${tag}`,
  pub_date: new Date().toISOString().replace(/\.\d{3}Z$/, "Z"),
  platforms,
};
writeFileSync(out, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`wrote ${out}: ${Object.keys(platforms).join(", ")}`);
