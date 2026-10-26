// Acceptance, scenario 9: backup and restore of the library through the real
// interface and the real Windows confirmation dialog.
//
//   node tools/acceptance/restore.mjs
//
// Makes a project, backs the library up, makes a second project, restores the
// backup from Settings (confirming in the native dialog), lets the app restart
// itself, and checks that the second project is gone, that the library it
// replaced was kept, and that the app reports the restore.
import { execFile } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { promisify } from "node:util";
import { apiClient, args, chromium, deskSession, launchApp, logTo, sleep, until } from "./lib.mjs";

const execFileAsync = promisify(execFile);
const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-restore";
const out = args.out ?? "D:/verne-build/asten/evidence";
const log = logTo(join(out, "restore.log"));
const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: Boolean(ok), detail });
  log({ check: name, ok: Boolean(ok), detail: typeof detail === "string" ? detail : JSON.stringify(detail) });
};
const dialog = async (title, keys) => {
  try {
    const argv = ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "D:/verne-build/asten/wt/tools/acceptance/dialog.ps1", "-Title", title, "-Keys", keys, "-TimeoutSec", "25"];
    return (await execFileAsync("powershell.exe", argv, { encoding: "utf8" })).stdout.trim();
  } catch (error) {
    return `dialog failed (${error.code}): ${String(error.stdout ?? "").trim().slice(0, 200)} ${String(error.stderr ?? error.message).trim().slice(0, 300)}`;
  }
};

let app = await launchApp({ exe, profile, fresh: true });
let before;
try {
  const { page } = app;
  const session = await deskSession(page);
  const api = apiClient(session.origin, session.token);
  await api.post("/projects", { name: "Kept by the backup", kind: "novel", description: null });
  const made = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke("library_backup_now"));
  check("a backup is made", Boolean(made?.name), made?.name);
  await api.post("/projects", { name: "Made after the backup", kind: "novel", description: null });
  check("two projects exist before the restore", (await api.get("/projects")).length === 2);

  // Settings, Storage and versions, the backup's own Restore button.
  await page.reload({ waitUntil: "load" });
  await sleep(2000);
  const first = (await api.get("/projects"))[0];
  await page.evaluate(([p]) => { location.hash = `#/p/${p}/settings`; }, [first.id]);
  await page.locator(".settings-nav").waitFor({ timeout: 20000 });
  await page.locator(".settings-nav button", { hasText: "Storage and versions" }).first().click();
  const row = page.locator(".setting-row", { hasText: made.name });
  await row.waitFor({ timeout: 15000 });
  await page.screenshot({ path: join(out, "restore-1-before.png") });
  before = app;
  // First answer no (Escape): nothing may change.
  const cancelled = row.getByRole("button", { name: "Restore" }).click();
  const no = await dialog("Restore the library", "{ESC}");
  await cancelled.catch(() => undefined);
  await sleep(2500);
  check("the app asks before it replaces the library; Escape cancels and nothing changes", /"found":true/.test(no) && (await api.get("/projects")).length === 2, no.slice(0, 160));
  // Then answer yes (Enter on the default button, "Restore and restart").
  const click = row.getByRole("button", { name: "Restore" }).click();
  const said = await dialog("Restore the library", "{ENTER}");
  log({ confirmDialog: said });
  check("confirming restores and restarts", /"found":true/.test(said), said.slice(0, 200));
  await click.catch(() => undefined);
} catch (error) {
  log({ error: String(error).slice(0, 300) });
}

// The app restarts itself; attach to the new process on the same debugging port.
await Promise.race([before?.exited, sleep(20000)]);
let browser;
for (let i = 0; i < 120 && !browser; i++) {
  try {
    browser = await chromium.connectOverCDP("http://127.0.0.1:9333");
  } catch {
    await sleep(500);
  }
}
check("the app restarted itself", Boolean(browser));
if (browser) {
  let page;
  for (let i = 0; i < 60 && !page; i++) {
    page = browser.contexts().flatMap((c) => c.pages())[0];
    if (!page) await sleep(250);
  }
  await page.waitForLoadState("domcontentloaded");
  const session = await deskSession(page);
  const api = apiClient(session.origin, session.token);
  const names = (await api.get("/projects")).map((p) => p.name);
  check("the library is the backed-up one: the later project is gone", names.length === 1 && names[0] === "Kept by the backup", names.join(", "));
  const info = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke("library_info"));
  check("the app reports the restore", info.last_restore?.ok === true, JSON.stringify(info.last_restore ?? null).slice(0, 200));
  const kept = readdirSync(info.backups_path).filter((f) => f.startsWith("before-restore-"));
  check("the library it replaced was kept", kept.length >= 1, kept.join(", "));
  await page.screenshot({ path: join(out, "restore-2-after.png") });
  await browser.close().catch(() => undefined);
  // End the new process tree (it is not the child the harness started).
  await execFileAsync("powershell.exe", ["-NoProfile", "-Command", `Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*verne-build*asten*scratch*profile-restore*' -or ($_.Name -eq 'asten-desktop.exe' -and $_.ExecutablePath -like '*verne-build*') } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }`]).catch(() => undefined);
  await sleep(2000);
}
if (!existsSync(profile)) check("the scratch profile exists", false);
const failed = results.filter((r) => !r.ok);
log({ summary: `${results.length - failed.length} of ${results.length} checks passed`, failed: failed.map((f) => `${f.name}: ${f.detail}`) });
process.exit(failed.length ? 1 : 0);
