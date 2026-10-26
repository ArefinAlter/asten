// Shared helpers for the 0.0.1b acceptance runs.
//
// The real desktop app is started with a scratch profile (ASTEN_DATA_DIR and a
// WebView2 user-data folder inside it), with WebView2's remote debugging port
// on, and driven over the Chrome DevTools Protocol with Playwright. Nothing
// here reads or writes the real %APPDATA%\com.asten.app.
//
//   node tools/acceptance/<script>.mjs [--exe <path to asten-desktop.exe>] [--profile <dir>]
import { spawn, spawnSync } from "node:child_process";
import { mkdirSync, rmSync, existsSync, writeFileSync, appendFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join, resolve } from "node:path";

const require = createRequire(resolve(import.meta.dirname, "../../apps/web/package.json"));
export const { chromium } = require("@playwright/test");

export const args = Object.fromEntries(
  process.argv
    .slice(2)
    .join(" ")
    .split(/\s*--/)
    .filter(Boolean)
    .map((pair) => {
      const [key, ...rest] = pair.split(/\s+/);
      return [key, rest.join(" ") || "true"];
    }),
);

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

export function logTo(file) {
  mkdirSync(resolve(file, ".."), { recursive: true });
  writeFileSync(file, "");
  return (line) => {
    const text = typeof line === "string" ? line : JSON.stringify(line);
    console.log(text);
    appendFileSync(file, text + "\n");
  };
}

/**
 * End any program still holding a scratch profile from an earlier run (an app
 * that restarted itself, its WebView2 helpers), found by the profile's path in
 * the helpers' command lines and the apps that started them.
 */
export function releaseProfile(profile) {
  const needle = profile.replace(/\//g, "\\").replace(/'/g, "''");
  const script = `
    $all = Get-CimInstance Win32_Process
    $helpers = $all | Where-Object { $_.CommandLine -and $_.CommandLine.IndexOf('${needle}', [System.StringComparison]::OrdinalIgnoreCase) -ge 0 }
    $parents = $helpers | ForEach-Object { $_.ParentProcessId } | Sort-Object -Unique
    foreach ($id in $parents) { $p = $all | Where-Object { $_.ProcessId -eq $id -and $_.Name -eq 'asten-desktop.exe' }; if ($p) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue } }
    foreach ($h in $helpers) { Stop-Process -Id $h.ProcessId -Force -ErrorAction SilentlyContinue }`;
  try {
    spawnSync("powershell.exe", ["-NoProfile", "-Command", script], { stdio: "ignore" });
  } catch {}
}

/** Start the app with a scratch profile; resolves with handles. */
export async function launchApp({ exe, profile, port = 9333, env = {}, fresh = true }) {
  releaseProfile(profile);
  if (fresh && existsSync(profile)) {
    for (let attempt = 0; attempt < 5; attempt++) {
      try {
        rmSync(profile, { recursive: true, force: true });
        break;
      } catch (error) {
        if (attempt === 4) throw error;
        releaseProfile(profile);
        await sleep(1500);
      }
    }
  }
  const data = join(profile, "data");
  const webview = join(profile, "webview");
  const roaming = join(profile, "AppData", "Roaming");
  const local = join(profile, "AppData", "Local");
  for (const dir of [data, webview, roaming, local]) mkdirSync(dir, { recursive: true });
  const child = spawn(exe, [], {
    env: {
      ...process.env,
      ASTEN_DATA_DIR: data,
      WEBVIEW2_USER_DATA_FOLDER: webview,
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
      APPDATA: roaming,
      LOCALAPPDATA: local,
      ASTEN_KEY_STORE: process.env.ASTEN_KEY_STORE ?? "memory",
      ELECTRON_RUN_AS_NODE: "",
      ...env,
    },
    stdio: "ignore",
    windowsHide: false,
  });
  const exited = new Promise((r) => child.on("exit", r));
  // Wait for the debugging endpoint.
  let browser;
  for (let i = 0; i < 120; i++) {
    try {
      browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
      break;
    } catch {
      await sleep(500);
    }
  }
  if (!browser) {
    child.kill();
    throw new Error("the app's debugging port never opened");
  }
  let page;
  for (let i = 0; i < 60 && !page; i++) {
    page = browser.contexts().flatMap((c) => c.pages())[0];
    if (!page) await sleep(250);
  }
  await page.waitForLoadState("domcontentloaded");
  return {
    child,
    browser,
    page,
    data,
    profile,
    exited,
    async close() {
      try {
        await browser.close();
      } catch {}
      // The window and its WebView2 helpers: end the whole tree, then let the
      // operating system release the profile's files.
      try {
        spawnSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" });
      } catch {}
      await Promise.race([exited, sleep(5000)]);
      await sleep(1500);
    },
  };
}

/** The embedded API's origin and the desk token, read through the app's own command. */
export async function deskSession(page) {
  for (let attempt = 0; attempt < 40; attempt++) {
    try {
      return await page.evaluate(async () => {
        const internals = window.__TAURI_INTERNALS__;
        if (!internals) throw new Error("not ready");
        return internals.invoke("desk_session");
      });
    } catch {
      await sleep(500);
    }
  }
  throw new Error("the app never answered desk_session");
}

/** A small client for the embedded API, using the desk token. */
export function apiClient(origin, token) {
  const base = `${origin}/api/v1`;
  async function call(method, path, body, raw = false) {
    const response = await fetch(`${base}${path}`, {
      method,
      headers: {
        authorization: `Bearer ${token}`,
        ...(body !== undefined && !raw ? { "content-type": "application/json" } : {}),
      },
      body: body === undefined ? undefined : raw ? body : JSON.stringify(body),
    });
    if (raw === "binary") return Buffer.from(await response.arrayBuffer());
    const text = await response.text();
    let json;
    try {
      json = text ? JSON.parse(text) : undefined;
    } catch {
      json = undefined;
    }
    if (!response.ok) {
      const error = new Error(`${method} ${path} -> ${response.status} ${text.slice(0, 300)}`);
      error.status = response.status;
      throw error;
    }
    return json;
  }
  return {
    base,
    get: (path) => call("GET", path),
    post: (path, body) => call("POST", path, body ?? {}),
    put: (path, body) => call("PUT", path, body ?? {}),
    del: (path) => call("DELETE", path),
    call,
    async binary(path, body) {
      const response = await fetch(`${base}${path}`, {
        method: "POST",
        headers: { authorization: `Bearer ${token}`, "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      if (!response.ok) throw new Error(`${path} -> ${response.status} ${(await response.text()).slice(0, 300)}`);
      return Buffer.from(await response.arrayBuffer());
    },
  };
}

export function percentile(values, p) {
  const sorted = [...values].sort((a, b) => a - b);
  if (!sorted.length) return 0;
  return sorted[Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length))];
}

export function stats(values) {
  return {
    n: values.length,
    min: Math.min(...values),
    median: percentile(values, 50),
    p95: percentile(values, 95),
    max: Math.max(...values),
    mean: values.reduce((a, b) => a + b, 0) / (values.length || 1),
  };
}

/** Wait for a condition polled every 200 ms. */
export async function until(check, { timeout = 15000, label = "condition" } = {}) {
  const start = Date.now();
  for (;;) {
    const value = await check();
    if (value) return value;
    if (Date.now() - start > timeout) throw new Error(`timed out waiting for ${label}`);
    await sleep(200);
  }
}

/**
 * Open a document in the editor and wait until it is on screen. The window can
 * still be settling after launch (it navigates once while the desk pairs), which
 * drops a route set too early, so the route is set again until the editor shows.
 * The page is reloaded first so that projects made through the API are listed.
 */
export async function openDocument(page, projectId, documentId, { timeout = 60000 } = {}) {
  const hash = `#/p/${projectId}/write/${documentId}`;
  const end = Date.now() + timeout;
  // The window lists projects when it loads; one made through the API since then is not in it yet.
  await page.reload({ waitUntil: "load" }).catch(() => undefined);
  await sleep(1500);
  while (Date.now() < end) {
    await page.evaluate((h) => { if (location.hash !== h) location.hash = h; }, hash).catch(() => undefined);
    const shown = await page.locator(".ProseMirror").first().waitFor({ state: "visible", timeout: 6000 }).then(() => true, () => false);
    if (shown) return;
  }
  throw new Error(`the editor never showed ${documentId}`);
}
