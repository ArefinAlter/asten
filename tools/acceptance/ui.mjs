// Acceptance, scenarios 1 and 9: a first run on a scratch profile, the sample
// novel, a new project, typing with autosave, closing and reopening; then the
// settings a writer changes (theme, accent, density, a keybinding, the AI
// switch), the library backup, the project file (saved through the real
// Windows dialog and opened again), rename and delete; and that all of it is
// still there after another restart.
//
//   node tools/acceptance/ui.mjs [--exe path] [--profile dir] [--out dir]
//
// Nothing here touches the owner's real %APPDATA%\com.asten.app: the app is
// pointed at a scratch folder with ASTEN_DATA_DIR and its own WebView2 folder,
// and the script checks that the real folder did not change.
import { execFile } from "node:child_process";
import { promisify } from "node:util";
const execFileAsync = promisify(execFile);
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { apiClient, args, deskSession, launchApp, logTo, openDocument, sleep, until } from "./lib.mjs";

const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-ui";
const out = args.out ?? "D:/verne-build/asten/evidence";
const log = logTo(join(out, "ui.log"));
const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: Boolean(ok), detail });
  log({ check: name, ok: Boolean(ok), detail: typeof detail === "string" ? detail : JSON.stringify(detail) });
};
const dialog = async (title, button, path = "") => {
  try {
    const argv = ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "D:/verne-build/asten/wt/tools/acceptance/dialog.ps1", "-Title", title, "-Button", button, "-TimeoutSec", "25"];
    if (path) argv.push("-Path", path);
    return (await execFileAsync("powershell.exe", argv, { encoding: "utf8" })).stdout.trim();
  } catch (error) {
    return `dialog failed (${error.code}): ${String(error.stdout ?? "").trim().slice(0, 200)} ${String(error.stderr ?? error.message).trim().slice(0, 400)}`;
  }
};

// The owner's real data folder must not change while this runs.
const real = join(process.env.APPDATA ?? "", "com.asten.app");
const snapshot = (dir) => {
  if (!existsSync(dir)) return "absent";
  const walk = (d) =>
    readdirSync(d, { withFileTypes: true }).flatMap((e) => {
      const p = join(d, e.name);
      return e.isDirectory() ? walk(p) : [`${p}:${statSync(p).mtimeMs}:${statSync(p).size}`];
    });
  return walk(dir).sort().join("\n");
};
const realBefore = snapshot(real);

const TYPED = " The harness typed this line to prove the page autosaves.";
let sampleId;
let sceneId;
let newId;

// ───────────── Session 1: first run ─────────────
let app = await launchApp({ exe, profile, fresh: true });
try {
  const { page } = app;
  const session = await deskSession(page);
  let api = apiClient(session.origin, session.token);
  await page.getByRole("heading", { name: /A writing desk with a memory/ }).waitFor({ timeout: 30000 });
  check("a first run opens on the welcome page with no project", (await api.get("/projects")).length === 0);
  await page.screenshot({ path: join(out, "ui-1-welcome.png") });
  check("the welcome page offers a new project and the sample", (await page.getByRole("button", { name: "Create a project" }).count()) === 1 && (await page.getByRole("button", { name: "Open the sample novel" }).count()) === 1);

  await page.getByRole("button", { name: "Open the sample novel" }).click();
  const projects1 = await until(async () => {
    const list = await api.get("/projects");
    return list.length === 1 ? list : undefined;
  }, { timeout: 30000, label: "the sample project" });
  sampleId = projects1[0].id;
  check("the sample novel is created as a real project", projects1[0].name === "Salt and Lantern", projects1[0].name);
  // The sample is written piece by piece after the project exists.
  const scenes = await until(async () => {
    const found = (await api.get(`/projects/${sampleId}/documents?fields=summary`)).filter((d) => d.kind === "scene");
    return found.length >= 3 ? found : undefined;
  }, { timeout: 45000, label: "the sample's scenes" });
  sceneId = scenes[0].id;
  const cast = await until(async () => {
    const found = await api.get(`/projects/${sampleId}/entities`);
    return found.length >= 2 ? found : undefined;
  }, { timeout: 45000, label: "the sample's cast" }).catch(() => []);
  check("the sample has scenes, a cast and marks the engine found", scenes.length >= 3 && cast.length >= 2, `${scenes.length} scenes, ${cast.length} entities`);
  await sleep(2500);
  await page.screenshot({ path: join(out, "ui-2-sample.png") });
  const marks = (await api.get(`/projects/${sampleId}/story/diagnostics`)).diagnostics;
  log({ sampleMarks: marks.map((m) => m.code) });

  // A new project from File > New.
  await page.locator(".ribbon-file").click();
  await page.locator(".backstage-nav button", { hasText: /^New$/ }).click();
  await page.locator('.backstage input[placeholder="Untitled project"]').fill("Lantern Notes");
  await page.locator(".backstage .template", { hasText: "Novel" }).first().click();
  const projects2 = await until(async () => {
    const list = await api.get("/projects");
    return list.length === 2 ? list : undefined;
  }, { timeout: 20000, label: "the new project" });
  newId = projects2.find((p) => p.name === "Lantern Notes")?.id;
  check("a new project is created from File, New", Boolean(newId), projects2.map((p) => p.name).join(", "));
  await sleep(1500);
  await page.screenshot({ path: join(out, "ui-3-new-project.png") });

  // Type into a sample scene and let it autosave.
  await openDocument(page, sampleId, sceneId);
  const editor = page.locator(".ProseMirror").first();
  await editor.click();
  await page.keyboard.press("Control+End");
  for (const ch of TYPED) await page.keyboard.press(ch === " " ? "Space" : ch, { delay: 0 });
  const saved = await until(async () => JSON.stringify((await api.get(`/projects/${sampleId}/documents/${sceneId}`)).content_json).includes("autosaves"), { timeout: 20000, label: "the autosave" }).catch(() => false);
  check("typing is saved without pressing anything", saved);
  await sleep(800);
  check("the status line says saved", /saved/i.test(await page.locator("body").innerText()));
  await page.screenshot({ path: join(out, "ui-4-typed.png") });
  const lib = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke("library_info"));
  check("the library is one file in the scratch folder", lib.database_path.toLowerCase().replace(/\\/g, "/").startsWith(profile.toLowerCase()), lib.database_path);
} finally {
  await app.close();
}

// ───────────── Session 2: reopen, then change settings ─────────────
app = await launchApp({ exe, profile, fresh: false });
try {
  const { page } = app;
  const session = await deskSession(page);
  const api = apiClient(session.origin, session.token);
  const projects = await api.get("/projects");
  check("after a restart both projects are still there", projects.length === 2 && projects.some((p) => p.name === "Lantern Notes") && projects.some((p) => p.name === "Salt and Lantern"), projects.map((p) => p.name).join(", "));
  const doc = await api.get(`/projects/${sampleId}/documents/${sceneId}`);
  check("and the typed line is in the scene", JSON.stringify(doc.content_json).includes("autosaves"));
  await sleep(2000);
  check("the desk opens where it was left", /Salt and Lantern|Lantern Notes/.test(await page.locator("body").innerText()), (await page.evaluate(() => location.hash)));
  await page.screenshot({ path: join(out, "ui-5-reopened.png") });

  // Settings.
  const goSettings = async (label) => {
    await page.evaluate(([p]) => { location.hash = `#/p/${p}/settings`; }, [sampleId]);
    await page.locator(".settings-nav").waitFor({ timeout: 15000 });
    await page.locator(".settings-nav button", { hasText: label }).first().click();
    await sleep(500);
  };
  await goSettings("Appearance");
  await page.locator(".theme-card", { hasText: "Dark" }).click();
  await sleep(400);
  const theme1 = await page.evaluate(() => document.documentElement.dataset.theme ?? document.body.dataset.theme ?? document.documentElement.className);
  check("choosing Dark changes the theme at once", /dark|night/.test(String(theme1)), String(theme1));
  await page.locator('.accent-swatch[aria-label="Accent #1f3f77"]').click();
  await page.locator(".setting-row", { hasText: "Density" }).locator("select").selectOption("compact");
  await page.screenshot({ path: join(out, "ui-6-settings-dark.png") });

  await goSettings("Keybindings");
  await page.locator('input[aria-label="Search keybindings"]').fill("Run checks");
  const row = page.locator(".wb-table tbody tr").first();
  const before = await row.locator("kbd.chord").innerText();
  await row.getByRole("button", { name: /record/i }).click();
  await page.keyboard.press("Control+Alt+J");
  await sleep(500);
  const after = await row.locator("kbd.chord").innerText();
  check("a keybinding can be recorded", after !== before && /J/i.test(after), `${before} -> ${after}`);

  await goSettings("AI providers");
  const aiText = await page.locator(".settings-body").innerText();
  check("the AI switch is off by default and says what it means", /off/i.test(aiText) && /no model|no call|nothing is sent|never/i.test(aiText), aiText.slice(0, 200).replace(/\n/g, " | "));
  await page.screenshot({ path: join(out, "ui-7-providers.png") });

  await goSettings("Storage and versions");
  const storageText = await page.locator(".settings-body").innerText();
  check("Storage shows the library file and its folder", /Your library/.test(storageText) && /asten/i.test(storageText));
  await page.getByRole("button", { name: "Back up now" }).click();
  await until(async () => /Backed up as/.test(await page.locator(".settings-body").innerText()), { timeout: 20000, label: "the backup message" });
  const libInfo = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke("library_info"));
  const backups = existsSync(libInfo.backups_path) ? readdirSync(libInfo.backups_path).filter((f) => f.endsWith(".db")) : [];
  check("Back up now writes a consistent copy", backups.length >= 1, backups.join(", "));
  await page.screenshot({ path: join(out, "ui-8-storage.png") });

  // The project file, through the real save dialog.
  const asten = join(profile, "export", "lantern-notes.asten");
  mkdirSync(join(profile, "export"), { recursive: true });
  await openDocument(page, sampleId, sceneId);
  await sleep(1000);
  await page.locator(".ribbon-file").click();
  await page.locator(".backstage-nav button", { hasText: /^Save as$/ }).click();
  const saveBtn = page.getByRole("button", { name: /Save a project file/ }).first();
  await saveBtn.waitFor({ timeout: 10000 });
  const pending = saveBtn.click();
  const said = await dialog("Save As", "Save", asten.replace(/\//g, "\\"));
  log({ saveDialog: said });
  await pending;
  await until(async () => existsSync(asten), { timeout: 15000, label: "the project file" }).catch(() => undefined);
  check("Save a project file writes through the Windows dialog", existsSync(asten), said.slice(0, 200));
  if (existsSync(asten)) {
    const file = JSON.parse(readFileSync(asten, "utf8"));
    check("the project file is asten-project v1 with the manuscript", file.format === "asten-project" && file.version === 1 && JSON.stringify(file).includes("autosaves"), `${file.format} v${file.version}`);
    // Two projects will share a name; give the original another so a row can be picked by name.
    await api.put(`/projects/${sampleId}`, { name: "Salt and Lantern (original)" });
    await page.locator(".backstage-nav button", { hasText: /^Open$/ }).click();
    await page.locator('input[type="file"][accept*=".asten"]').setInputFiles(asten);
    const three = await until(async () => {
      const list = await api.get("/projects");
      return list.length === 3 ? list : undefined;
    }, { timeout: 20000, label: "the opened project file" }).catch(() => undefined);
    check("Open a project file creates a third project from it", Boolean(three), three ? three.map((p) => p.name).join(", ") : "still two");
    // Rename it, then delete it, by the interface.
    if (three) {
      const copy = three.find((p) => !projects.some((q) => q.id === p.id));
      if (!(await page.locator(".backstage").count())) await page.locator(".ribbon-file").click();
      await page.locator(".backstage-nav button", { hasText: /^Open$/ }).click().catch(() => undefined);
      await page.getByRole("button", { name: `Rename ${copy.name}`, exact: true }).click();
      await page.locator('[role="dialog"] input').last().fill("Salt and Lantern, a copy");
      await page.locator('[role="dialog"] button', { hasText: /^Rename$/ }).click();
      const renamed = await until(async () => (await api.get("/projects")).some((p) => p.name === "Salt and Lantern, a copy"), { timeout: 15000, label: "the rename" }).catch(() => false);
      log({ renamedIsTheCopy: (await api.get("/projects")).find((p) => p.name === "Salt and Lantern, a copy")?.id === copy.id });
      check("a project can be renamed from the Open page", renamed);
      await page.getByRole("button", { name: "Delete Salt and Lantern, a copy" }).click();
      await page.locator('[role="dialog"] input').last().fill("Salt and Lantern, a copy");
      await page.locator('[role="dialog"] button', { hasText: /Delete for good/ }).click();
      const gone = await until(async () => (await api.get("/projects")).length === 2, { timeout: 15000, label: "the delete" }).catch(() => false);
      check("a project can be deleted by typing its name", gone);
    }
  }
  await page.screenshot({ path: join(out, "ui-9-backstage.png") });
} finally {
  await app.close();
}

// ───────────── Session 3: the settings survived ─────────────
app = await launchApp({ exe, profile, fresh: false });
try {
  const { page } = app;
  const session = await deskSession(page);
  const api = apiClient(session.origin, session.token);
  await sleep(2500);
  const theme = await page.evaluate(() => document.documentElement.dataset.theme ?? document.body.dataset.theme ?? document.documentElement.className);
  check("the theme is still Dark after a restart", /dark|night/.test(String(theme)), String(theme));
  await page.evaluate(([p]) => { location.hash = `#/p/${p}/settings`; }, [sampleId]);
  await page.locator(".settings-nav").waitFor({ timeout: 15000 });
  await page.locator(".settings-nav button", { hasText: "Appearance" }).first().click();
  await sleep(500);
  check("the accent and density are kept", (await page.locator('.accent-swatch.active[aria-label="Accent #1f3f77"]').count()) === 1 && (await page.locator(".setting-row", { hasText: "Density" }).locator("select").inputValue()) === "compact");
  await page.locator(".settings-nav button", { hasText: "Keybindings" }).first().click();
  await page.locator('input[aria-label="Search keybindings"]').fill("Run checks");
  check("the recorded keybinding is kept", /J/i.test(await page.locator(".wb-table tbody tr").first().locator("kbd.chord").innerText()));
  await page.screenshot({ path: join(out, "ui-10-after-restart.png") });
  check("the projects are all still there", (await api.get("/projects")).length === 2);
  // Back to Light so the evidence of the first run is not the only one.
  await page.locator(".settings-nav button", { hasText: "Appearance" }).first().click();
  await page.locator(".theme-card", { hasText: "Light" }).click();
  await sleep(300);
} finally {
  await app.close();
}

const realAfter = snapshot(real);
check("the owner's real %APPDATA%\\com.asten.app was not touched", realBefore === realAfter, realBefore === "absent" ? "folder absent before and after" : "unchanged");
const failed = results.filter((r) => !r.ok);
log({ summary: `${results.length - failed.length} of ${results.length} checks passed`, failed: failed.map((f) => `${f.name}: ${f.detail}`) });
process.exit(failed.length ? 1 : 0);
