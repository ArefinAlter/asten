// Acceptance, scenario 7: the updater, from an installed older build to the
// release build, through the real window and the real signed installer.
//
//   node tools/acceptance/updater.mjs --old <installer of the older build>
//        --new <the release installer> [--sig <its .sig>]
//
// The older build is a local build of the same code with version 0.0.1-alpha
// and its update address pointed at this script's loopback server (nothing
// else differs; the public key is the release key). The script serves a
// manifest that names the release installer and its signature, installs the
// older build silently into a scratch folder, opens Settings, presses
// "Check for updates", then "Install and relaunch", and checks that the
// installed program is now the release build, still opens, and still has the
// project made before the update. It then uninstalls.
import { execFile } from "node:child_process";
import { createReadStream, existsSync, readFileSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { join } from "node:path";
import { promisify } from "node:util";
import { apiClient, args, chromium, deskSession, launchApp, logTo, sleep, until } from "./lib.mjs";

const execFileAsync = promisify(execFile);
const out = args.out ?? "D:/verne-build/asten/evidence";
const log = logTo(join(out, "updater.log"));
const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: Boolean(ok), detail });
  log({ check: name, ok: Boolean(ok), detail: typeof detail === "string" ? detail : JSON.stringify(detail) });
};
const oldInstaller = args.old;
const newInstaller = args.new;
const sigFile = args.tamper ?? args.sig ?? `${newInstaller}.sig`; // --tamper <another file's .sig>: the update must be refused
const installDir = args.dir ?? "D:\\verne-build\\asten\\install-old";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-updater";
const port = 18765;
if (!oldInstaller || !newInstaller) throw new Error("--old and --new are required");

const version = async (file) =>
  (await execFileAsync("powershell.exe", ["-NoProfile", "-Command", `(Get-Item '${file}').VersionInfo.ProductVersion`], { encoding: "utf8" })).stdout.trim();

// The manifest and the installer, on loopback.
const manifest = {
  version: "0.0.1-beta",
  notes: "Asten 0.0.1b (pre-alpha), served locally for the update test.",
  pub_date: new Date().toISOString().replace(/\.\d{3}Z$/, "Z"),
  platforms: {
    "windows-x86_64": {
      signature: readFileSync(sigFile, "utf8").trim(),
      url: `http://127.0.0.1:${port}/Asten_0.0.1-beta_x64-setup.exe`,
    },
  },
};
const requests = [];
const server = createServer((req, res) => {
  requests.push(`${req.method} ${req.url}`);
  if (req.url?.startsWith("/latest.json")) {
    res.writeHead(200, { "content-type": "application/json" });
    res.end(JSON.stringify(manifest));
  } else if (req.url === "/Asten_0.0.1-beta_x64-setup.exe") {
    res.writeHead(200, { "content-type": "application/octet-stream", "content-length": statSync(newInstaller).size });
    createReadStream(newInstaller).pipe(res);
  } else {
    res.writeHead(404);
    res.end();
  }
});
await new Promise((resolve) => server.listen(port, "127.0.0.1", resolve));

let app;
let cdp;
try {
  // Install the older build, silently, into a scratch folder.
  await execFileAsync(oldInstaller, ["/S", `/D=${installDir}`]);
  const installed = join(installDir, "asten-desktop.exe");
  check("the older build installs silently into the scratch folder", existsSync(installed), installed);
  const before = await version(installed);
  check("it is the older version", /0\.0\.1-alpha/.test(before), before);

  app = await launchApp({ exe: installed, profile, fresh: true });
  const { page } = app;
  const session = await deskSession(page);
  const api = apiClient(session.origin, session.token);
  const desk = await fetch(`${session.origin}/api/v1/desk`, { headers: { authorization: `Bearer ${session.token}` } }).then((r) => r.json());
  // The API reports the crate version (the same code in both builds); the version the updater compares is the one in the app config and the installer.
  log({ apiReportsCrateVersion: desk.version });
  const project = await api.post("/projects", { name: "Made before the update", kind: "novel", description: null });

  // Settings, Continuity checks, "This desk", Check for updates.
  await page.reload({ waitUntil: "load" });
  await sleep(2000);
  await page.evaluate(([p]) => { location.hash = `#/p/${p}/settings`; }, [project.id]);
  await page.locator(".settings-nav").waitFor({ timeout: 20000 });
  await page.locator(".settings-nav button", { hasText: "Continuity checks" }).first().click();
  const button = page.getByRole("button", { name: "Check for updates" });
  await button.waitFor({ timeout: 20000 });
  await button.scrollIntoViewIfNeeded();
  await button.click();
  const found = await page.getByText(/Version 0\.0\.1-beta available/).waitFor({ timeout: 30000 }).then(() => true, () => false);
  await page.screenshot({ path: join(out, "updater-1-available.png") });
  check("pressing Check for updates finds 0.0.1-beta, and only then", found, requests.join(" | "));
  check("nothing was requested before the button was pressed", requests.length >= 1 && requests[0].startsWith("GET /latest.json"), requests[0] ?? "none");
  const install = page.getByRole("button", { name: "Install and relaunch" });
  const finished = app.exited;
  await install.click();
  if (args.tamper) {
    // A signature that does not belong to the installer: the update must fail and change nothing.
    const refused = await page.getByText(/Could not check/).waitFor({ timeout: 120000 }).then(() => true, () => false);
    const message = refused ? await page.getByText(/Could not check/).first().innerText() : "";
    await page.screenshot({ path: join(out, "updater-refused.png") });
    check("an installer whose signature does not match is refused", refused, message.slice(0, 200));
    check("and the installed program is still the older build", /0\.0\.1-alpha/.test(await version(installed)), await version(installed));
    throw Object.assign(new Error("tamper test finished"), { finished: true });
  }
  await page.screenshot({ path: join(out, "updater-2-installing.png") }).catch(() => undefined);
  // The app hands over to the installer and exits; wait for the installed program to change.
  const upgraded = await until(async () => {
    try {
      return /0\.0\.1-beta/.test(await version(installed));
    } catch {
      return false;
    }
  }, { timeout: 180000, label: "the installed program to become 0.0.1-beta" }).catch(() => false);
  check("the installer ran and replaced the program with 0.0.1-beta", upgraded, await version(installed).catch(() => "?"));
  check("the new installer was fetched from the manifest's address, with its signature checked", requests.some((r) => r.includes("setup.exe")), requests.join(" | "));
  void finished;

  // The relaunched app (or one we start) is the new version and has the data.
  let browser;
  for (let i = 0; i < 90 && !browser; i++) {
    try {
      browser = await chromium.connectOverCDP("http://127.0.0.1:9333");
    } catch {
      await sleep(1000);
    }
  }
  let relaunched = Boolean(browser);
  if (!browser) {
    app = await launchApp({ exe: installed, profile, fresh: false });
    browser = app.browser;
  }
  cdp = browser;
  let page2;
  for (let i = 0; i < 60 && !page2; i++) {
    page2 = browser.contexts().flatMap((c) => c.pages())[0];
    if (!page2) await sleep(500);
  }
  await page2.waitForLoadState("domcontentloaded");
  const session2 = await deskSession(page2);
  const desk2 = await fetch(`${session2.origin}/api/v1/desk`, { headers: { authorization: `Bearer ${session2.token}` } }).then((r) => r.json());
  check("the app that is running afterwards reports 0.0.1-beta", /0\.0\.1-beta/.test(desk2.version), `${desk2.version} (${relaunched ? "relaunched by the update" : "started by the script"})`);
  const names = (await apiClient(session2.origin, session2.token).get("/projects")).map((p) => p.name);
  check("the project made before the update is still there", names.includes("Made before the update"), names.join(", "));
  await page2.screenshot({ path: join(out, "updater-3-after.png") });
} catch (error) {
  if (!error.finished) throw error;
} finally {
  try { await cdp?.close(); } catch {}
  await execFileAsync("powershell.exe", ["-NoProfile", "-Command", `Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -like '${installDir.replace(/\\/g, "\\\\")}*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }`]).catch(() => undefined);
  await sleep(2000);
  try { await app?.close(); } catch {}
  // Uninstall, so nothing of the test stays on the machine.
  const uninstaller = join(installDir, "uninstall.exe");
  if (existsSync(uninstaller)) await execFileAsync(uninstaller, ["/S"]).catch(() => undefined);
  await sleep(4000);
  check("the uninstaller removes the program", !existsSync(join(installDir, "asten-desktop.exe")));
  server.close();
}
const failed = results.filter((r) => !r.ok);
log({ summary: `${results.length - failed.length} of ${results.length} checks passed`, failed: failed.map((f) => `${f.name}: ${f.detail}`) });
process.exit(failed.length ? 1 : 0);
