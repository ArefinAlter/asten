#!/usr/bin/env node
/*
 * WCAG 2.1 contrast audit for the desk's brand tokens.
 *
 * Reads packages/brand/src/tokens.css, resolves both themes (day from `:root`,
 * night from `:root[data-theme="night"]`), and reports the contrast ratio of
 * every text and UI pair the desk actually paints. Thresholds follow WCAG 2.1
 * AA: 4.5:1 for body text (anything under 18.66px bold / 24px regular, which
 * is all of the desk's chrome), 3:1 for large text and for UI parts such as
 * focus rings and borders.
 *
 * Where a token pair fails, the desk fixes it at the component level in
 * apps/web/src/asten.css rather than by editing the brand tokens. Those
 * overrides are listed in OVERRIDES below so the "after" column shows what
 * the desk ships.
 *
 *   node scripts/contrast-audit.mjs          table
 *   node scripts/contrast-audit.mjs --json   machine-readable
 *
 * Exits 1 when any pair fails after overrides.
 */

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const tokensPath = resolve(here, "../packages/brand/src/tokens.css");
const css = readFileSync(tokensPath, "utf8");

/** Pull `--name: value;` declarations out of the block that follows `selector`. */
function block(selector) {
  const start = css.indexOf(selector);
  if (start < 0) throw new Error(`No block for ${selector} in tokens.css`);
  const open = css.indexOf("{", start);
  let depth = 0;
  let end = open;
  for (; end < css.length; end += 1) {
    if (css[end] === "{") depth += 1;
    if (css[end] === "}") {
      depth -= 1;
      if (depth === 0) break;
    }
  }
  const body = css.slice(open + 1, end);
  const vars = {};
  for (const match of body.matchAll(/--([\w-]+)\s*:\s*([^;]+);/g)) {
    vars[match[1]] = match[2].trim();
  }
  return vars;
}

const dayVars = block(":root {");
const nightVars = { ...dayVars, ...block(':root[data-theme="night"]') };

/** Resolve var() references to a literal colour string. */
function resolveVar(vars, value, depth = 0) {
  if (depth > 10) throw new Error(`var() loop in ${value}`);
  const match = /^var\(--([\w-]+)\)$/.exec(value.trim());
  if (!match) return value.trim();
  const next = vars[match[1]];
  if (next === undefined) throw new Error(`Unknown token --${match[1]}`);
  return resolveVar(vars, next, depth + 1);
}

/** Parse #rgb, #rrggbb, or `rgb(r g b / a%)` into [r, g, b, a] in 0..1. */
function parseColor(text) {
  const hex = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(text);
  if (hex) {
    let digits = hex[1];
    if (digits.length === 3)
      digits = digits
        .split("")
        .map((d) => d + d)
        .join("");
    const n = Number.parseInt(digits, 16);
    return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255, 1];
  }
  const rgb =
    /^rgb\(\s*(\d+)\s+(\d+)\s+(\d+)\s*(?:\/\s*([\d.]+)(%?)\s*)?\)$/i.exec(text);
  if (rgb) {
    let alpha = 1;
    if (rgb[4] !== undefined) {
      alpha = Number.parseFloat(rgb[4]);
      if (rgb[5] === "%") alpha /= 100;
    }
    return [
      Number(rgb[1]) / 255,
      Number(rgb[2]) / 255,
      Number(rgb[3]) / 255,
      alpha,
    ];
  }
  throw new Error(`Cannot parse colour ${text}`);
}

/** Composite a colour with alpha over an opaque base. */
function over(top, base) {
  const a = top[3];
  return [
    top[0] * a + base[0] * (1 - a),
    top[1] * a + base[1] * (1 - a),
    top[2] * a + base[2] * (1 - a),
    1,
  ];
}

function luminance([r, g, b]) {
  const lin = (c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

function ratio(fg, bg) {
  const l1 = luminance(fg);
  const l2 = luminance(bg);
  const [light, dark] = l1 > l2 ? [l1, l2] : [l2, l1];
  return (light + 0.05) / (dark + 0.05);
}

function toHex([r, g, b]) {
  const h = (c) =>
    Math.round(c * 255)
      .toString(16)
      .padStart(2, "0");
  return `#${h(r)}${h(g)}${h(b)}`;
}

/**
 * Colour spec: a token name (`ink`), a literal (`#fffaf2`), a stack
 * `wash over papyrus` for a translucent token composited on a base, or
 * `mix(ink-3 80%, papyrus-raised)` for CSS color-mix in sRGB.
 */
function colorOf(vars, spec) {
  const trimmed = spec.trim();
  const mix = /^mix\((.+?)\s+(\d+)%\s*,\s*(.+)\)$/.exec(trimmed);
  if (mix) {
    const a = colorOf(vars, mix[1]);
    const b = colorOf(vars, mix[3]);
    const p = Number(mix[2]) / 100;
    return [
      a[0] * p + b[0] * (1 - p),
      a[1] * p + b[1] * (1 - p),
      a[2] * p + b[2] * (1 - p),
      1,
    ];
  }
  const parts = trimmed.split(" over ").map((part) => part.trim());
  const head = parts[0];
  const literal = head.startsWith("#") || head.startsWith("rgb");
  let color = parseColor(resolveVar(vars, literal ? head : `var(--${head})`));
  for (const base of parts.slice(1)) {
    color = over(color, colorOf(vars, base));
  }
  return color;
}

// What the desk paints. `need` is the AA threshold for how the pair is used.
const PAIRS = [
  { use: "Body text", fg: "ink", bg: "papyrus", need: 4.5 },
  { use: "Body text on the page", fg: "ink", bg: "papyrus-raised", need: 4.5 },
  { use: "Body text, sunk panels", fg: "ink", bg: "papyrus-sunk", need: 4.5 },
  { use: "Secondary text (lede, crumb)", fg: "ink-2", bg: "papyrus", need: 4.5 },
  { use: "Secondary text on the page", fg: "ink-2", bg: "papyrus-raised", need: 4.5 },
  { use: "Activity labels, 9px mono", fg: "ink-2", bg: "papyrus-sunk", need: 4.5 },
  { use: "Muted mono labels (pane headings, eyebrow, hints)", fg: "ink-3", bg: "papyrus", need: 4.5 },
  { use: "Muted mono labels on the page", fg: "ink-3", bg: "papyrus-raised", need: 4.5 },
  { use: "Muted labels in a selected binder row", fg: "ink-3", bg: "papyrus-sunk", need: 4.5 },
  { use: "Rubric marks and errors", fg: "rubric", bg: "papyrus", need: 4.5 },
  { use: "Rubric in a selected binder row", fg: "rubric", bg: "papyrus-sunk", need: 4.5 },
  { use: "Code cartouche on its wash", fg: "rubric", bg: "rubric-wash over papyrus-raised", need: 4.5 },
  { use: "Status bar text", fg: "papyrus", bg: "ink", need: 4.5 },
  { use: "Settled (malachite)", fg: "malachite", bg: "papyrus", need: 4.5 },
  { use: "Links and linkish buttons (lapis)", fg: "lapis", bg: "papyrus", need: 4.5 },
  { use: "Links on the page", fg: "lapis", bg: "papyrus-raised", need: 4.5 },
  { use: "Primary button label", fg: "#fffaf2", bg: "rubric", need: 4.5 },
  { use: "Primary button label, hover", fg: "#fffaf2", bg: "rubric-deep", need: 4.5 },
  { use: "Warn text", fg: "warn", bg: "papyrus", need: 4.5 },
  { use: "Focus ring on chrome", fg: "lapis", bg: "papyrus", need: 3, ui: true },
  { use: "Focus ring on the page", fg: "lapis", bg: "papyrus-raised", need: 3, ui: true },
  { use: "Focus ring on sunk panels", fg: "lapis", bg: "papyrus-sunk", need: 3, ui: true },
  { use: "Text field borders (inside edge)", fg: "rule", bg: "papyrus-raised", need: 3, ui: true },
  { use: "Text field borders (outside edge)", fg: "rule", bg: "papyrus", need: 3, ui: true },
  { use: "Button borders", fg: "rule", bg: "papyrus", need: 3, ui: true },
  { use: "Theme toggle (gold sun)", fg: "gold", bg: "papyrus", need: 3, ui: true },
  { use: "Offline dot in the status bar", fg: "ink-3", bg: "ink", need: 3, ui: true },
];

// Component-level fixes shipped in apps/web/src/asten.css for pairs that fail
// on tokens alone. Keyed by `${theme}|${use}`; the value is the shipped pair.
// `accepted` marks a pair that needs no fix because the colour is not the
// only thing carrying the information.
const FIELD_BORDER = "mix(ink-3 80%, papyrus-raised)";
const OVERRIDES = {
  "night|Primary button label": {
    fg: "papyrus",
    bg: "rubric",
    where: ".btn--primary (night) sets --btn-fg to var(--papyrus)",
  },
  "night|Primary button label, hover": {
    fg: "papyrus",
    bg: "rubric-deep",
    where: ".btn--primary (night) sets --btn-fg to var(--papyrus)",
  },
  "night|Code cartouche on its wash": {
    fg: "rubric-deep",
    bg: "rubric-wash over papyrus-raised",
    where: ".chip--code (night) uses var(--rubric-deep)",
  },
  "night|Warn text": {
    fg: "gold",
    bg: "papyrus",
    where: ".warn (night) uses var(--gold)",
  },
  "day|Text field borders (inside edge)": {
    fg: FIELD_BORDER,
    bg: "papyrus-raised",
    where: "input, select, textarea: border-color color-mix(ink-3 80%, papyrus-raised)",
  },
  "day|Text field borders (outside edge)": {
    fg: FIELD_BORDER,
    bg: "papyrus",
    where: "same rule",
  },
  "night|Text field borders (inside edge)": {
    fg: FIELD_BORDER,
    bg: "papyrus-raised",
    where: "same rule",
  },
  "night|Text field borders (outside edge)": {
    fg: FIELD_BORDER,
    bg: "papyrus",
    where: "same rule",
  },
  "day|Button borders": {
    fg: "rule",
    bg: "papyrus",
    where: "accepted: the label identifies a button; WCAG 1.4.11 asks nothing of its border",
    accepted: true,
  },
  "night|Button borders": {
    fg: "rule",
    bg: "papyrus",
    where: "accepted: as above",
    accepted: true,
  },
  "day|Theme toggle (gold sun)": {
    fg: "ink",
    bg: "papyrus",
    where: ".theme-toggle .tt-disk gets a 1.2px ink stroke; the outline identifies the control, the gold is pigment",
  },
  "day|Offline dot in the status bar": {
    fg: "ink-3",
    bg: "ink",
    where: "accepted: the words live/offline carry the state; the dot is decoration",
    accepted: true,
  },
  "night|Offline dot in the status bar": {
    fg: "ink-3",
    bg: "ink",
    where: "accepted: as above",
    accepted: true,
  },
};

const rows = [];
for (const [theme, vars] of [
  ["day", dayVars],
  ["night", nightVars],
]) {
  for (const pair of PAIRS) {
    const fg = colorOf(vars, pair.fg);
    const bg = colorOf(vars, pair.bg);
    const before = ratio(fg, bg);
    const override = OVERRIDES[`${theme}|${pair.use}`];
    let after = before;
    let afterPair = null;
    if (override) {
      const ofg = colorOf(vars, override.fg);
      const obg = colorOf(vars, override.bg);
      after = ratio(ofg, obg);
      afterPair = `${toHex(ofg)} on ${toHex(obg)}`;
    }
    rows.push({
      theme,
      use: pair.use,
      pair: `${toHex(fg)} on ${toHex(bg)}`,
      need: pair.need,
      before: Number(before.toFixed(2)),
      after: Number(after.toFixed(2)),
      afterPair,
      pass: after >= pair.need || Boolean(override?.accepted),
      where: override?.where,
    });
  }
}

if (process.argv.includes("--json")) {
  console.log(JSON.stringify(rows, null, 2));
} else {
  const pad = (text, width) => String(text).padEnd(width);
  console.log(
    `${pad("theme", 6)}${pad("use", 52)}${pad("pair", 20)}${pad("need", 6)}${pad("before", 8)}${pad("after", 8)}result`,
  );
  console.log("-".repeat(108));
  for (const row of rows) {
    const result = row.pass
      ? row.before >= row.need
        ? "pass"
        : row.after >= row.need
          ? "fixed"
          : "accepted"
      : "FAIL";
    console.log(
      `${pad(row.theme, 6)}${pad(row.use, 52)}${pad(row.pair, 20)}${pad(`${row.need}:1`, 6)}${pad(row.before.toFixed(2), 8)}${pad(row.before === row.after ? "" : row.after.toFixed(2), 8)}${result}`,
    );
    if (row.where) console.log(`      ${row.afterPair ? `${row.afterPair}  ` : ""}${row.where}`);
  }
  const failing = rows.filter((row) => !row.pass);
  console.log("-".repeat(108));
  console.log(
    failing.length
      ? `${failing.length} pair${failing.length === 1 ? "" : "s"} below AA after overrides.`
      : "Every pair meets WCAG 2.1 AA after overrides.",
  );
}

process.exitCode = rows.some((row) => !row.pass) ? 1 : 0;
