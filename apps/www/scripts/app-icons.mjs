#!/usr/bin/env node
/**
 * Asten's application icons, drawn from the Palette mark (docs/BRAND.md,
 * section 4) and nothing else.
 *
 *   node apps/www/scripts/app-icons.mjs            render and write everything
 *   node apps/www/scripts/app-icons.mjs --check    exit 1 if a file would change
 *
 * What it writes:
 *   apps/desktop/src-tauri/icon-source/asten-icon.svg   the master (1024 square)
 *   apps/desktop/src-tauri/icon-source/asten-icon.png   the same, rendered
 *   apps/desktop/src-tauri/icons/*                      Tauri's full set (`tauri icon`),
 *                                                       then the small PNGs and the
 *                                                       .ico frames drawn directly
 *   apps/desktop/src-tauri/icons/installer-*.bmp        the Windows installer's header and side picture
 *   apps/web/public/icons/*                             favicon, 180, 192, 512, maskable
 *   apps/addin-word/icon-32.png, icon-64.png
 *
 * The plate is the day page colour (#F5EEDF). The mark is the favicon geometry
 * of the Palette (heavier strokes, so it holds at 16 px) with the reed's nib.
 * Clear space is one well-diameter on every side of the slate (BRAND.md), which
 * puts the slate at 71% of the plate's height; at 32 px and below the slate is
 * drawn larger (88%, then 80% up to 64 px) so it stays legible. The maskable
 * icon keeps the whole slate inside the 80% circle every platform mask keeps.
 * There is no rounded plate: BRAND.md does not define one.
 */
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { Resvg } from "@resvg/resvg-js";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "..", "..", "..");
const check = process.argv.includes("--check");

const c = {
  page: "#F5EEDF",
  raised: "#FBF7EE",
  sunk: "#ECE2CC",
  ink: "#1E1A15",
  red: "#B0351F",
  ochre: "#C98A2E",
};

/** The Palette on its plate. `slate` is the slate's height as a share of the plate. */
function plate(size, slate) {
  const unit = (size * slate) / 39; // the slate is 39 units tall
  const tx = size / 2 - 16 * unit;
  const ty = size / 2 - 22 * unit;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">
  <rect width="${size}" height="${size}" fill="${c.page}"/>
  <g transform="translate(${tx.toFixed(3)} ${ty.toFixed(3)}) scale(${unit.toFixed(5)})">
    <rect x="3.5" y="2.5" width="25" height="39" rx="5.5" fill="${c.raised}" stroke="${c.ink}" stroke-width="2.6"/>
    <circle cx="11.2" cy="11" r="4.4" fill="${c.ink}"/>
    <circle cx="20.8" cy="11" r="4.4" fill="${c.red}"/>
    <rect x="13.4" y="18.5" width="5.2" height="19.5" rx="2.6" fill="${c.sunk}" stroke="${c.ink}" stroke-width="1.6"/>
    <path d="M16 16.5 V32.4" stroke="${c.ochre}" stroke-width="2.8" stroke-linecap="round"/>
    <path d="M14.9 32 L17.1 32 L16.4 36.2 L15.6 36.2 Z" fill="${c.ink}"/>
  </g>
</svg>
`;
}

const slateFor = (size) => (size <= 32 ? 0.88 : size <= 64 ? 0.8 : 0.71);
const render = (svg, size) =>
  new Resvg(svg, { fitTo: { mode: "width", value: size }, font: { loadSystemFonts: false } }).render().asPng();
const drawn = (size) => render(plate(size, slateFor(size)), size);

/** An .ico holding PNG frames. */
function ico(sizes) {
  const frames = sizes.map((s) => ({ s, png: drawn(s) }));
  const head = Buffer.alloc(6 + 16 * frames.length);
  head.writeUInt16LE(0, 0);
  head.writeUInt16LE(1, 2);
  head.writeUInt16LE(frames.length, 4);
  let offset = head.length;
  frames.forEach(({ s, png }, i) => {
    const at = 6 + 16 * i;
    head.writeUInt8(s >= 256 ? 0 : s, at);
    head.writeUInt8(s >= 256 ? 0 : s, at + 1);
    head.writeUInt8(0, at + 2);
    head.writeUInt8(0, at + 3);
    head.writeUInt16LE(1, at + 4);
    head.writeUInt16LE(32, at + 6);
    head.writeUInt32LE(png.length, at + 8);
    head.writeUInt32LE(offset, at + 12);
    offset += png.length;
  });
  return Buffer.concat([head, ...frames.map((f) => f.png)]);
}

let drift = 0;
function write(file, data) {
  const target = path.join(repo, file);
  if (check) {
    const same = fs.existsSync(target) && Buffer.compare(fs.readFileSync(target), Buffer.from(data)) === 0;
    if (!same) {
      drift += 1;
      console.error(`would change: ${file}`);
    }
    return;
  }
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, data);
}

const source = "apps/desktop/src-tauri/icon-source";
const icons = "apps/desktop/src-tauri/icons";
const master = plate(1024, 0.71);
write(`${source}/asten-icon.svg`, master);
write(`${source}/asten-icon.png`, render(master, 1024));

if (!check) {
  // Tauri's own generator writes the whole set (icns, ico, Square*Logo, StoreLogo,
  // Android, iOS); the small sizes are then redrawn directly, which is crisper
  // than shrinking the master.
  const desktop = path.join(repo, "apps", "desktop");
  const cli = path.join(desktop, "node_modules", "@tauri-apps", "cli", "tauri.js");
  execFileSync(process.execPath, [cli, "icon", path.join(repo, source, "asten-icon.png"), "-o", path.join(repo, icons)], {
    cwd: desktop,
    stdio: "inherit",
  });
}
for (const s of [32, 64, 128]) write(`${icons}/${s}x${s}.png`, drawn(s));
write(`${icons}/128x128@2x.png`, drawn(256));
write(`${icons}/icon.png`, render(master, 512));
write(`${icons}/icon.ico`, ico([16, 24, 32, 48, 64, 128, 256]));

// The Windows installer's pictures (NSIS wants 24-bit bitmaps): the mark on the
// day page colour, in the header (150 x 57) and beside the welcome page (164 x 314).
function bmp(width, height, svg) {
  const { pixels } = new Resvg(svg, { fitTo: { mode: "width", value: width }, font: { loadSystemFonts: false } }).render();
  const row = Math.ceil((width * 3) / 4) * 4;
  const body = Buffer.alloc(row * height);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const at = (y * width + x) * 4;
      const to = (height - 1 - y) * row + x * 3; // bottom-up, BGR
      body[to] = pixels[at + 2];
      body[to + 1] = pixels[at + 1];
      body[to + 2] = pixels[at];
    }
  }
  const head = Buffer.alloc(54);
  head.write("BM", 0);
  head.writeUInt32LE(54 + body.length, 2);
  head.writeUInt32LE(54, 10);
  head.writeUInt32LE(40, 14);
  head.writeInt32LE(width, 18);
  head.writeInt32LE(height, 22);
  head.writeUInt16LE(1, 26);
  head.writeUInt16LE(24, 28);
  head.writeUInt32LE(body.length, 34);
  head.writeInt32LE(2835, 38);
  head.writeInt32LE(2835, 42);
  return Buffer.concat([head, body]);
}
/** The mark, `slate` units per pixel scale `unit`, centred at (cx, cy) on a plate of the given size. */
function picture(width, height, slateHeight, cx, cy) {
  const unit = slateHeight / 39;
  const tx = cx - 16 * unit;
  const ty = cy - 22 * unit;
  return plate(1, 1)
    .replace(/<svg[^>]*>/, `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">`)
    .replace(/<rect width="1" height="1"[^>]*\/>/, `<rect width="${width}" height="${height}" fill="${c.page}"/>`)
    .replace(/translate\([^)]*\) scale\([^)]*\)/, `translate(${tx.toFixed(3)} ${ty.toFixed(3)}) scale(${unit.toFixed(5)})`);
}
write(`${icons}/installer-header.bmp`, bmp(150, 57, picture(150, 57, 46, 150 - 30, 28.5)));
write(`${icons}/installer-sidebar.bmp`, bmp(164, 314, picture(164, 314, 112, 82, 120)));

// The browser desk and the installed web app.
const web = "apps/web/public/icons";
write(`${web}/asten-180.png`, render(plate(180, 0.71), 180));
write(`${web}/asten-192.png`, render(plate(192, 0.71), 192));
write(`${web}/asten-512.png`, render(plate(512, 0.71), 512));
write(`${web}/asten-maskable-512.png`, render(plate(512, 0.6), 512));
write(
  `${web}/favicon.svg`,
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 44">
  <style>
    .s{fill:#FBF7EE;stroke:#1E1A15}.b{fill:#1E1A15}.r{fill:#B0351F}.t{fill:#ECE2CC;stroke:#1E1A15}
    @media (prefers-color-scheme: dark){.s{fill:#15233D;stroke:#F1E7D2}.b{fill:#F1E7D2}.r{fill:#E8694C}.t{fill:#0B1424;stroke:#F1E7D2}}
  </style>
  <rect class="s" x="3.5" y="2.5" width="25" height="39" rx="5.5" stroke-width="2.6"/>
  <circle class="b" cx="11.2" cy="11" r="4.4"/>
  <circle class="r" cx="20.8" cy="11" r="4.4"/>
  <rect class="t" x="13.4" y="18.5" width="5.2" height="19.5" rx="2.6" stroke-width="1.6"/>
  <path d="M16 16.5 V32.4" stroke="#C98A2E" stroke-width="2.8" stroke-linecap="round"/>
</svg>
`,
);

// The Word add-in's ribbon icons.
write("apps/addin-word/icon-32.png", drawn(32));
write("apps/addin-word/icon-64.png", drawn(64));

if (check && drift) process.exit(1);
console.log(check ? "icons are up to date" : "icons written");
