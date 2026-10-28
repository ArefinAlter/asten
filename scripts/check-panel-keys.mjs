// Build with: pnpm --filter @asten/web exec vite build --mode sample --outDir dist-sample
// Then run: node scripts/check-panel-keys.mjs
import { createServer } from "node:http";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const root = resolve(
  process.env.ASTEN_PANEL_ASSETS ?? join(repo, "apps/web/dist-sample"),
);
assert(
  existsSync(join(root, "index.html")),
  "Build the sample frontend before running the panel check.",
);
const { chromium, expect } = createRequire(
  process.env.ASTEN_PLAYWRIGHT_PACKAGE ?? join(repo, "apps/web/package.json"),
)("@playwright/test");
const types = {
  ".js": "text/javascript",
  ".css": "text/css",
  ".html": "text/html",
  ".svg": "image/svg+xml",
  ".woff2": "font/woff2",
  ".png": "image/png",
};
const server = createServer((request, response) => {
  let file = resolve(
    root,
    "." + new URL(request.url, "http://localhost").pathname,
  );
  if (
    !file.startsWith(root + "/") &&
    !file.startsWith(root + "\\") &&
    file !== root
  ) {
    response.writeHead(403).end();
    return;
  }
  if (!existsSync(file) || statSync(file).isDirectory())
    file = join(root, "index.html");
  response.setHeader(
    "content-type",
    types[extname(file)] ?? "application/octet-stream",
  );
  response.end(readFileSync(file));
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
let browser;
const results = [],
  errors = [];
try {
  const channel =
    process.env.ASTEN_BROWSER_CHANNEL ??
    (process.platform === "win32" ? "msedge" : undefined);
  browser = await chromium.launch({
    headless: true,
    ...(channel ? { channel } : {}),
  });
  const page = await browser.newPage({
    viewport: { width: 1920, height: 900 },
  });
  page.on("pageerror", (error) => errors.push(String(error)));
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  await expect(page.locator(".sidebar-resizer")).toBeVisible();
  for (const [panel, grip, key] of [
    [".wb-sidebar", ".sidebar-resizer", "ArrowRight"],
    [".secondary", ".secondary-resizer", "ArrowLeft"],
  ]) {
    // Large saved preferences must first be fitted to the smaller workspace.
    await page.setViewportSize({ width: 1920, height: 900 });
    await page.locator(".sidebar-resizer").focus();
    await page.keyboard.press("End");
    await page.locator(".secondary-resizer").focus();
    await page.keyboard.press("End");
    await page.setViewportSize({ width: 1024, height: 600 });
    await expect
      .poll(async () => (await page.locator(".wb-center").boundingBox()).width)
      .toBeGreaterThanOrEqual(320);
    const before = (await page.locator(panel).boundingBox()).width;
    const expected = Math.round(
      Math.min(
        before + 10,
        Number(await page.locator(grip).getAttribute("aria-valuemax")),
      ),
    );
    await page.locator(grip).focus();
    await page.keyboard.press(key);
    await expect(page.locator(grip)).toHaveAttribute(
      "aria-valuenow",
      String(expected),
    );
    await expect
      .poll(async () => (await page.locator(panel).boundingBox()).width)
      .toBeGreaterThanOrEqual(before);
    const after = (await page.locator(panel).boundingBox()).width;
    const center = (await page.locator(".wb-center").boundingBox()).width;
    assert(center >= 320);
    results.push({ panel, before, after, center });
  }
  assert.deepEqual(errors, []);
  console.log("Panel keyboard regression passed", JSON.stringify(results));
} finally {
  if (process.env.ASTEN_PANEL_EVIDENCE) {
    const path = resolve(process.env.ASTEN_PANEL_EVIDENCE);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(
      path,
      JSON.stringify({ results, pageErrors: errors }, null, 2),
    );
  }
  await browser?.close();
  await new Promise((resolve) => server.close(resolve));
}
