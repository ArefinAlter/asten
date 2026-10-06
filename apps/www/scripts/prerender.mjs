/**
 * Static prerender for the Asten website.
 *
 * Runs after `vite build` (client) and `vite build --ssr` (server). Renders every
 * route to HTML, then writes the crawl and share files: sitemap, robots, llms.txt,
 * llms-full.txt, RSS, security.txt, humans.txt, the manifest, icons, press-kit SVGs,
 * and an Open Graph card per page. Fails the build on a broken internal link.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { Resvg } from "@resvg/resvg-js";
import satori from "satori";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const dist = path.join(root, "dist");
const server = await import(pathToFileURL(path.join(root, "dist-server", "entry-server.js")).href);
const { render, routes, site, llmsText, llmsFullIntro, rssFeed, artMarkup, ogArtFor } = server;

const started = Date.now();
// Keep a pristine copy of the template so this script can be re-run on its own.
const templateCopy = path.join(root, "dist-server", "template.html");
const built = fs.readFileSync(path.join(dist, "index.html"), "utf8");
if (built.includes("<!--app-html-->")) fs.writeFileSync(templateCopy, built);
const template = fs.readFileSync(templateCopy, "utf8");

function write(rel, content) {
  const file = path.join(dist, rel);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

function fileFor(routePath) {
  if (routePath === "/") return "index.html";
  if (routePath === "/404") return "404.html";
  return `${routePath.slice(1)}.html`;
}

/* ───────── Font preloads ───────── */

const assets = fs.readdirSync(path.join(dist, "assets"));
const preloads = [/^fraunces-latin-standard-normal-.*\.woff2$/, /^literata-latin-wght-normal-.*\.woff2$/]
  .map((pattern) => assets.find((name) => pattern.test(name)))
  .filter(Boolean)
  .map((name) => `<link rel="preload" href="/assets/${name}" as="font" type="font/woff2" crossorigin />`)
  .join("\n    ");

/* ───────── HTML → Markdown-ish text for llms-full.txt ───────── */

function removeElements(html, test) {
  let out = html;
  for (;;) {
    const open = test.exec(out);
    test.lastIndex = 0;
    if (!open) return out;
    const tag = open[1];
    const start = open.index;
    if (/\/>$/.test(open[0])) {
      out = out.slice(0, start) + out.slice(start + open[0].length);
      continue;
    }
    const scanner = new RegExp(`<\\/?${tag}\\b[^>]*>`, "g");
    scanner.lastIndex = start + open[0].length;
    let depth = 1;
    let end = out.length;
    for (let match; (match = scanner.exec(out)); ) {
      if (match[0].startsWith("</")) depth -= 1;
      else if (!match[0].endsWith("/>")) depth += 1;
      if (depth === 0) {
        end = match.index + match[0].length;
        break;
      }
    }
    out = out.slice(0, start) + out.slice(end);
  }
}

function decode(text) {
  return text
    .replace(/&nbsp;/g, " ")
    .replace(/&quot;/g, '"')
    .replace(/&#x27;|&#39;/g, "'")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&");
}

function toText(html) {
  const main = html.slice(html.indexOf("<main"), html.lastIndexOf("</main>"));
  let s = removeElements(main, /<([a-z0-9]+)\b[^>]*\b(?:aria-hidden="true"|data-llms="skip")[^>]*>/i);
  s = s.replace(/<(script|style|svg|button|input|label)\b[\s\S]*?<\/\1>/gi, "");
  s = s.replace(/<a\b[^>]*href="([^"]+)"[^>]*>([\s\S]*?)<\/a>/gi, (_, href, inner) => {
    const label = inner.replace(/<[^>]+>/g, "").trim();
    if (!label) return "";
    const url = href.startsWith("/") ? `${site.origin}${href}` : href;
    return `[${label}](${url})`;
  });
  s = s
    .replace(/<br\s*\/?>/gi, "\n")
    .replace(/<h1\b[^>]*>/gi, "\n\n# ")
    .replace(/<h2\b[^>]*>/gi, "\n\n## ")
    .replace(/<h3\b[^>]*>/gi, "\n\n### ")
    .replace(/<\/h[1-6]>/gi, "\n\n")
    .replace(/<li\b[^>]*>/gi, "\n- ")
    .replace(/<dt\b[^>]*>/gi, "\n\n**")
    .replace(/<\/dt>/gi, "**\n")
    .replace(/<t[hd]\b[^>]*>/gi, " | ")
    .replace(/<\/?(p|div|section|article|header|footer|figure|ul|ol|dl|dd|table|thead|tbody|tr|blockquote|nav|aside|details|summary|figcaption|main)\b[^>]*>/gi, "\n")
    .replace(/<[^>]+>/g, "");
  return decode(s)
    .split("\n")
    .map((line) => line.replace(/[ \t]+/g, " ").trim())
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

/* ───────── Render every route ───────── */

const known = new Set(routes.map((route) => route.path));
const links = new Map();
const fullText = [llmsFullIntro()];

for (const meta of routes) {
  const { html, head } = await render(meta.path);
  if (/<script(?![^>]*(?:src|type="application\/ld\+json")=?)[^>]*>/.test(html) || html.includes('<div hidden id="S:')) {
    console.error(`Inline script or streamed segment in ${meta.path}; the CSP would block it.`);
    process.exit(1);
  }
  const page = template
    .replace("<!--app-head-->", `${head}\n    ${preloads}`)
    .replace("<!--app-html-->", html);
  write(fileFor(meta.path), page);

  for (const match of html.matchAll(/href="(\/[^"#?]*)/g)) {
    const target = match[1].length > 1 ? match[1].replace(/\/$/, "") : "/";
    if (!links.has(target)) links.set(target, new Set());
    links.get(target).add(meta.path);
  }
  if (!meta.noindex) {
    fullText.push(`---\n\nURL: ${site.origin}${meta.path === "/" ? "/" : meta.path}\nTitle: ${meta.title}\nSummary: ${meta.description}\n\n${toText(html)}\n`);
  }
}

/* ───────── Crawl files ───────── */

const indexable = routes.filter((route) => !route.noindex);
const url = (p) => (p === "/" ? `${site.origin}/` : `${site.origin}${p}`);
const ogSlug = (p) => (p === "/" ? "home" : p.slice(1).replaceAll("/", "-"));

write(
  "sitemap.xml",
  [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:image="http://www.google.com/schemas/sitemap-image/1.1">',
    ...indexable.map((route) =>
      [
        "  <url>",
        `    <loc>${url(route.path)}</loc>`,
        `    <lastmod>${route.modified ?? site.updated}</lastmod>`,
        `    <changefreq>${route.kind === "legal" ? "yearly" : route.kind === "journal" ? "weekly" : "monthly"}</changefreq>`,
        `    <priority>${(route.priority ?? 0.5).toFixed(1)}</priority>`,
        "    <image:image>",
        `      <image:loc>${site.origin}/og/${ogSlug(route.path)}.png</image:loc>`,
        "    </image:image>",
        "  </url>",
      ].join("\n"),
    ),
    "</urlset>",
    "",
  ].join("\n"),
);

const answerBots = [
  "Googlebot",
  "Bingbot",
  "DuckDuckBot",
  "Applebot",
  "Applebot-Extended",
  "Google-Extended",
  "GPTBot",
  "OAI-SearchBot",
  "ChatGPT-User",
  "ClaudeBot",
  "Claude-User",
  "Claude-SearchBot",
  "anthropic-ai",
  "PerplexityBot",
  "Perplexity-User",
  "DuckAssistBot",
  "Amazonbot",
  "Meta-ExternalAgent",
  "MistralAI-User",
  "cohere-ai",
  "YouBot",
  "CCBot",
];

write(
  "robots.txt",
  [
    "# Asten: public marketing pages. Search and answer engines are welcome.",
    "# Writers’ manuscripts never touch this website.",
    "",
    "User-agent: *",
    "Allow: /",
    "Disallow: /404",
    "",
    ...answerBots.flatMap((bot) => [`User-agent: ${bot}`, "Allow: /", ""]),
    `Sitemap: ${site.origin}/sitemap.xml`,
    "",
  ].join("\n"),
);

write("llms.txt", llmsText());
write("llms-full.txt", fullText.join("\n"));
write("journal/rss.xml", rssFeed());
write(
  "humans.txt",
  [
    "/* TEAM */",
    `Publisher: ${site.publisher} (${site.legalName})`,
    `Site: ${site.publisherUrl}`,
    `Contact: ${site.email}`,
    "From: Dhaka, Bangladesh",
    "",
    "/* SITE */",
    `Last update: ${site.updated}`,
    "Language: English",
    "Standards: HTML, CSS, SVG, JSON-LD",
    "Components: React, React Router, Vite",
    "Fonts: Fraunces, Literata, IBM Plex Mono, Noto Sans Egyptian Hieroglyphs (SIL OFL)",
    "Art: original SVG drawings in the Egyptian flat canon",
    "",
  ].join("\n"),
);
write(
  ".well-known/security.txt",
  [
    `Contact: mailto:${site.email}`,
    "Expires: 2027-10-06T00:00:00.000Z",
    "Preferred-Languages: en",
    `Canonical: ${site.origin}/.well-known/security.txt`,
    `Policy: ${site.origin}/security`,
    "",
  ].join("\n"),
);
write(
  "manifest.webmanifest",
  `${JSON.stringify(
    {
      name: "Asten",
      short_name: "Asten",
      description: site.description,
      start_url: "/",
      scope: "/",
      display: "browser",
      background_color: "#f5eedf",
      theme_color: "#f5eedf",
      lang: "en",
      icons: [
        { src: "/icon-192.png", sizes: "192x192", type: "image/png", purpose: "any" },
        { src: "/icon-512.png", sizes: "512x512", type: "image/png", purpose: "any" },
        { src: "/icon-maskable-512.png", sizes: "512x512", type: "image/png", purpose: "maskable" },
      ],
    },
    null,
    2,
  )}\n`,
);

/* ───────── Art: inline colours so standalone renderers can draw it ───────── */

const day = {
  ink: "#1E1A15", red: "#B0351F", blue: "#2B62B0", lapis: "#1F3F77", green: "#2E6B4E", turq: "#3D9A94",
  ochre: "#C98A2E", gold: "#D9A631", flesh: "#A9532E", natron: "#FFFCF5", papyrus: "#EFE3C6", sand: "#DCC597",
  shadow: "#C9B58C", page: "#F5EEDF", raised: "#FBF7EE", sunk: "#ECE2CC", night: "#0F1A2E", star: "#E3B341",
};

function styleFor(classes, colors) {
  let fill;
  let stroke;
  let width;
  let extra = "";
  for (const name of classes) {
    if (name.startsWith("f-")) fill = name === "f-none" ? "none" : colors[name.slice(2)] ?? fill;
    if (name === "o" || name === "o-thin" || name === "o-hair" || name === "o-bold") {
      stroke = colors.ink;
      width ??= { o: 2, "o-thin": 1.2, "o-hair": 0.8, "o-bold": 2.8 }[name];
    }
    if (name.startsWith("s-")) stroke = colors[{ "s-line": "ink", "s-ink": "ink", "s-rule": "shadow" }[name] ?? name.slice(2)] ?? stroke;
    if (/^w\d+$/.test(name)) width = { w1: 1, w15: 1.5, w2: 2, w3: 3, w4: 4 }[name];
    if (name === "moon-glow") extra += "opacity:0.22;";
    if (name === "slate") ((fill = colors.raised), (stroke = colors.ink), (width = 2.2));
    if (name === "well-black" || name === "nib") fill = colors.ink;
    if (name === "well-red") fill = colors.red;
    if (name === "slot") ((fill = colors.sunk), (stroke = colors.ink), (width = 1.4));
    if (name === "reed") ((stroke = colors.ochre), (width = 2.6), (fill = "none"));
    if (name === "chamber-box") ((fill = colors.raised), (stroke = colors.ink), (width = 1.5));
    if (name === "chamber-label" || name === "pyramid-caption") ((fill = colors.ink), (extra += "font-family:monospace;font-size:10px;text-anchor:middle;"));
  }
  let style = extra;
  if (fill) style += `fill:${fill};`;
  if (stroke) style += `stroke:${stroke};`;
  if (width) style += `stroke-width:${width};`;
  return style;
}

function inlineArt(svg, colors = day) {
  return svg
    .replace(/\sstyle="[^"]*var\([^"]*"/g, "")
    .replace(/<svg\b([^>]*)>/, (_, attrs) => `<svg xmlns="http://www.w3.org/2000/svg"${attrs.replace(/\sclass="[^"]*"/, "")} style="stroke-linecap:round;stroke-linejoin:round">`)
    .replace(/\sclass="([^"]*)"/g, (_, names) => {
      const style = styleFor(names.split(/\s+/), colors);
      return style ? ` style="${style}"` : "";
    });
}

const dataUri = (svg) => `data:image/svg+xml;base64,${Buffer.from(svg).toString("base64")}`;

function markSvg(colors, size = { w: 32, h: 44 }) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 44" width="${size.w}" height="${size.h}">
  <rect x="3.5" y="2.5" width="25" height="39" rx="5.5" fill="${colors.raised}" stroke="${colors.ink}" stroke-width="2.2"/>
  <circle cx="11.2" cy="11" r="3.9" fill="${colors.ink}"/>
  <circle cx="20.8" cy="11" r="3.9" fill="${colors.red}"/>
  <rect x="13.6" y="18.5" width="4.8" height="19.5" rx="2.4" fill="${colors.sunk}" stroke="${colors.ink}" stroke-width="1.4"/>
  <path d="M16 16.5 V32.4" stroke="${colors.ochre}" stroke-width="2.6" stroke-linecap="round"/>
  <path d="M14.9 32 L17.1 32 L16.4 36.2 L15.6 36.2 Z" fill="${colors.ink}"/>
</svg>`;
}

const night = { ...day, ink: "#F1E7D2", red: "#E8694C", raised: "#15233D", sunk: "#0B1424" };

function png(svg, width) {
  return new Resvg(svg, { fitTo: { mode: "width", value: width }, font: { loadSystemFonts: false } }).render().asPng();
}

/* ───────── Icons ───────── */

write(
  "favicon.svg",
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

function iconSvg(size, padding, background = day.page) {
  const inner = size - padding * 2;
  const markH = inner;
  const markW = (markH * 32) / 44;
  const x = (size - markW) / 2;
  const y = (size - markH) / 2;
  const mark = markSvg(day).replace(/<svg[^>]*>/, "").replace("</svg>", "");
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">
  <rect width="${size}" height="${size}" fill="${background}"/>
  <g transform="translate(${x} ${y}) scale(${markH / 44})">${mark}</g>
</svg>`;
}

write("apple-touch-icon.png", png(iconSvg(180, 22), 180));
write("icon-192.png", png(iconSvg(192, 20), 192));
write("icon-512.png", png(iconSvg(512, 56), 512));
write("icon-maskable-512.png", png(iconSvg(512, 110), 512));

// favicon.ico: a single 32×32 PNG inside an ICO container.
{
  const image = png(iconSvg(32, 2), 32);
  const header = Buffer.alloc(22);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(1, 4);
  header.writeUInt8(32, 6);
  header.writeUInt8(32, 7);
  header.writeUInt8(0, 8);
  header.writeUInt8(0, 9);
  header.writeUInt16LE(1, 10);
  header.writeUInt16LE(32, 12);
  header.writeUInt32LE(image.length, 14);
  header.writeUInt32LE(22, 18);
  write("favicon.ico", Buffer.concat([header, image]));
}

/* ───────── Satori: OG cards and press-kit lockups ───────── */

const fontDir = (pkg) => path.join(root, "node_modules", "@fontsource", pkg, "files");
const fonts = [
  { name: "Fraunces", data: fs.readFileSync(path.join(fontDir("fraunces"), "fraunces-latin-600-normal.woff")), weight: 600, style: "normal" },
  { name: "Literata", data: fs.readFileSync(path.join(fontDir("literata"), "literata-latin-400-normal.woff")), weight: 400, style: "normal" },
  { name: "Plex", data: fs.readFileSync(path.join(fontDir("ibm-plex-mono"), "ibm-plex-mono-latin-500-normal.woff")), weight: 500, style: "normal" },
];

// Satori treats an array as several children, so pass a lone child on its own.
const h = (type, style, children = []) => ({
  type,
  props: { style, children: children.length === 0 ? undefined : children.length === 1 ? children[0] : children },
});
const img = (src, width, height, style = {}) => ({ type: "img", props: { src, width, height, style } });

const blocksFrieze = dataUri(
  `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="16" viewBox="0 0 1200 16">${Array.from({ length: 29 }, (_, i) => {
    const x = i * 42;
    return `<rect x="${x + 1}" y="2" width="11" height="12" fill="${day.blue}"/><rect x="${x + 15}" y="2" width="11" height="12" fill="${day.red}"/><rect x="${x + 29}" y="2" width="11" height="12" fill="${day.green}"/>`;
  }).join("")}<path d="M0 1 H1200 M0 15 H1200" stroke="${day.ink}" stroke-width="1"/></svg>`,
);

function artSize(key) {
  const svg = artMarkup(key);
  const viewBox = /viewBox="([^"]+)"/.exec(svg)?.[1].split(/\s+/).map(Number) ?? [0, 0, 100, 100];
  return { svg: inlineArt(svg), w: viewBox[2], h: viewBox[3] };
}

const PANEL = { x: 706, y: 64, w: 430, h: 470 };

async function ogCard(meta) {
  const art = artSize(ogArtFor(meta));
  const box = { w: 360, h: 390 };
  const scale = Math.min(box.w / art.w, box.h / art.h);
  const aw = art.w * scale;
  const ah = art.h * scale;
  const size = meta.og.headline.length;
  const headline = size <= 38 ? 74 : size <= 64 ? 60 : 50;
  const tree = h(
    "div",
    { width: 1200, height: 630, display: "flex", flexDirection: "column", position: "relative", background: day.page, fontFamily: "Literata" },
    [
      img(blocksFrieze, 1200, 16),
      h("div", { display: "flex", width: 600, height: 500, marginTop: 48, marginLeft: 72, flexDirection: "column", justifyContent: "space-between" }, [
        h("div", { display: "flex", alignItems: "center", gap: 14, fontFamily: "Plex", fontSize: 22, letterSpacing: 3, textTransform: "uppercase", color: day.red }, [
          h("div", { width: 12, height: 12, background: day.red, transform: "rotate(45deg)" }),
          h("div", { display: "flex" }, [meta.og.eyebrow]),
        ]),
        h("div", { display: "flex", fontFamily: "Fraunces", fontSize: headline, lineHeight: 1.04, letterSpacing: -1.5, color: day.ink }, [meta.og.headline]),
        h("div", { display: "flex", alignItems: "center", gap: 14 }, [
          img(dataUri(markSvg(day)), 30, 41),
          h("div", { display: "flex", fontFamily: "Fraunces", fontSize: 36, color: day.ink }, ["Asten"]),
          h("div", { width: 2, height: 30, background: "#BFAB82", marginLeft: 10, marginRight: 10 }),
          h("div", { display: "flex", fontFamily: "Plex", fontSize: 19, color: "#6E6253" }, [site.origin.replace("https://", "")]),
        ]),
      ]),
      h("div", {
        position: "absolute",
        left: PANEL.x,
        top: PANEL.y,
        width: PANEL.w,
        height: PANEL.h,
        background: day.sunk,
        border: `3px solid ${day.ink}`,
        borderRadius: "215px 215px 14px 14px",
      }),
      h("div", { position: "absolute", left: 0, right: 0, bottom: 18, height: 10, borderTop: "3px solid #BFAB82", borderBottom: "1px solid #BFAB82" }),
    ],
  );
  const base = await satori(tree, { width: 1200, height: 630, fonts });
  const x = PANEL.x + (PANEL.w - aw) / 2;
  const y = PANEL.y + PANEL.h - 30 - ah;
  const nested = art.svg.replace("<svg ", `<svg x="${x.toFixed(1)}" y="${y.toFixed(1)}" width="${aw.toFixed(1)}" height="${ah.toFixed(1)}" `);
  return png(base.replace(/<\/svg>\s*$/, `${nested}</svg>`), 1200);
}

let ogCount = 0;
for (const meta of routes) {
  try {
    write(`og/${ogSlug(meta.path)}.png`, await ogCard(meta));
    ogCount += 1;
  } catch (error) {
    console.warn(`  ! OG card failed for ${meta.path}: ${error.message}`);
  }
}

async function lockup(colors, background) {
  const tree = h("div", { width: 560, height: 180, display: "flex", alignItems: "center", gap: 22, padding: "0 40px", background: background ?? "transparent" }, [
    img(dataUri(markSvg(colors)), 74, 102),
    h("div", { fontFamily: "Fraunces", fontSize: 112, letterSpacing: -2, color: colors.ink, lineHeight: 1 }, ["Asten"]),
  ]);
  return satori(tree, { width: 560, height: 180, fonts });
}

write("brand/asten-mark.svg", markSvg(day, { w: 320, h: 440 }));
write("brand/asten-mark-night.svg", markSvg(night, { w: 320, h: 440 }));
write("brand/asten-logo.svg", await lockup(day));
write("brand/asten-logo-night.svg", await lockup(night, day.night));
write(
  "brand/asten-cartouche.svg",
  await satori(
    h("div", { width: 640, height: 240, display: "flex", alignItems: "center", justifyContent: "center" }, [
      h("div", { display: "flex", alignItems: "center", gap: 20, padding: "26px 64px 26px 52px", border: `6px solid ${day.ink}`, borderRadius: 999, background: day.raised }, [
        img(dataUri(markSvg(day)), 56, 77),
        h("div", { fontFamily: "Fraunces", fontSize: 92, letterSpacing: -2, color: day.ink, lineHeight: 1 }, ["Asten"]),
      ]),
      h("div", { width: 12, height: 130, background: day.ink, borderRadius: 6, marginLeft: -2 }),
    ]),
    { width: 640, height: 240, fonts },
  ),
);

/* ───────── Link check ───────── */

const staticFiles = new Set(["/llms.txt", "/llms-full.txt", "/sitemap.xml", "/journal/rss.xml", "/robots.txt", "/icon-512.png", "/verne-mark.png", "/favicon.svg"]);
const broken = [];
for (const [target, sources] of links) {
  if (known.has(target) || staticFiles.has(target) || target.startsWith("/assets/") || target.startsWith("/og/") || target.startsWith("/brand/")) continue;
  broken.push(`${target}  (from ${[...sources].join(", ")})`);
}

console.log(`Prerendered ${routes.length} pages, ${ogCount} OG cards, crawl files, icons, and brand files in ${Date.now() - started} ms.`);
if (broken.length > 0) {
  console.error(`Broken internal links:\n  ${broken.join("\n  ")}`);
  process.exit(1);
}
