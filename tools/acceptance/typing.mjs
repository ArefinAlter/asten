// Acceptance: typing in the real window on a real novel, with the continuity
// engine running as the words arrive.
//
//   node tools/acceptance/typing.mjs [--whole-book]
//
// Measures, in the page: the time from each keydown to the second animation
// frame after it (the key's pixels are on screen), plus the browser's own Event
// Timing entries; then types a sentence that contradicts a settled fact and
// times how long until the mark exists on the server and appears in the window.
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { apiClient, args, deskSession, launchApp, logTo, openDocument, sleep, stats, until } from "./lib.mjs";

const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-typing";
const out = args.out ?? "D:/verne-build/asten/evidence";
const book = args.book ?? "D:/verne-build/asten/scratch/gutenberg/pg1342.txt";
const whole = Boolean(args["whole-book"]);
const log = logTo(join(out, whole ? "typing-whole-book.log" : "typing.log"));
const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: Boolean(ok), detail });
  log({ check: name, ok: Boolean(ok), detail: typeof detail === "string" ? detail : JSON.stringify(detail) });
};

const raw = readFileSync(book, "utf8");
const body = raw.slice(raw.indexOf("\n", raw.indexOf("*** START OF")) + 1, raw.indexOf("*** END OF")).trim();

const app = await launchApp({ exe, profile });
try {
  const { page } = app;
  const session = await deskSession(page);
  const api = apiClient(session.origin, session.token);
  const project = await api.post("/projects", { name: "Typing test", kind: "novel", description: null });
  // Whole-book mode: the entire novel as ONE scene inside a manuscript, the worst case for the editor.
  const shell = whole
    ? await api.post(`/projects/${project.id}/import`, { filename: "The book", format: "plain", body: "x", parent_id: null, kind: "manuscript", encoding: "utf-8", split: "none" })
    : null;
  const root = await api.post(`/projects/${project.id}/import`, {
    filename: "pride.txt", format: "plain", body, parent_id: shell ? shell.id : null, kind: whole ? "scene" : "manuscript", encoding: "utf-8", split: whole ? "none" : "auto",
  });
  const docs = await api.get(`/projects/${project.id}/documents?fields=summary`);
  const target = whole ? root : docs.filter((d) => d.parent_id === root.id).sort((a, b) => a.position - b.position)[4];
  const targetDoc = docs.find((d) => d.id === target.id);
  log({ document: targetDoc.title, words: targetDoc.words });
  await api.post(`/projects/${project.id}/entities`, { kind: "character", name: "Elizabeth", aliases: ["Lizzy"], notes: "" });
  await api.post(`/projects/${project.id}/facts`, { entity_key: "elizabeth", predicate: "eye_color", value: "brown", status: "confirmed" });

  const editor = page.locator(".ProseMirror").first();
  const opened = performance.now();
  await openDocument(page, project.id, target.id);
  await until(async () => (await editor.innerText()).length > 200, { timeout: 60000, label: "the page text" });
  log({ pageOpenMs: Math.round(performance.now() - opened) });
  await sleep(1500);
  await page.screenshot({ path: join(out, whole ? "typing-whole-book-open.png" : "typing-open.png") });

  // Probes: key to second frame, and the browser's event timing.
  await page.evaluate(() => {
    window.__lat = [];
    window.__evt = [];
    document.addEventListener(
      "keydown",
      () => {
        const t0 = performance.now();
        requestAnimationFrame(() => requestAnimationFrame(() => window.__lat.push(performance.now() - t0)));
      },
      true,
    );
    try {
      new PerformanceObserver((list) => {
        for (const e of list.getEntries()) if (e.name === "keydown" || e.name === "input") window.__evt.push({ name: e.name, d: e.duration });
      }).observe({ type: "event", durationThreshold: 16, buffered: true });
    } catch {}
  });

  await editor.click();
  await page.keyboard.press("Control+End");
  await page.keyboard.press("Enter");
  const typed = "She had been walking for some time before she noticed that the lane had narrowed to a footpath, and that the hedges on either side had closed over her head. ";
  const samples = [];
  const started = performance.now();
  for (let round = 0; round < 3; round++) {
    for (const ch of typed) {
      await page.keyboard.press(ch === " " ? "Space" : ch, { delay: 0 });
      await sleep(25);
    }
  }
  const typingMs = performance.now() - started;
  await sleep(500);
  const latencies = await page.evaluate(() => window.__lat);
  const events = await page.evaluate(() => window.__evt);
  const keys = latencies.length;
  const s = stats(latencies);
  log({ keystrokes: keys, wallMs: Math.round(typingMs), keyToFrameMs: s, slowEvents: events.length, worstEventMs: events.reduce((m, e) => Math.max(m, e.d), 0) });
  check("every keystroke reached the screen", keys >= typed.length * 3 * 0.98, `${keys} of ${typed.length * 3}`);
  check("typing stays responsive: median key-to-frame under 50 ms, p95 under 100 ms", s.median < 50 && s.p95 < 100, JSON.stringify(s));

  // A sentence that contradicts a settled fact, then wait for the mark.
  const clash = "Elizabeth lifted her head, and her blue eyes were bright with anger.";
  const before = Date.now();
  for (const ch of clash) {
    await page.keyboard.press(ch === " " ? "Space" : ch, { delay: 0 });
    await sleep(20);
  }
  const lastKey = performance.now();
  const onServer = await until(async () => {
    const d = await api.get(`/projects/${project.id}/story/diagnostics`);
    return d.diagnostics.find((m) => m.code === "CANON-E013");
  }, { timeout: 30000, label: "the mark on the server" });
  const serverMs = Math.round(performance.now() - lastKey);
  log({ markOnServerAfterLastKeyMs: serverMs, mark: onServer.title });
  check("the mark exists on the server within a few seconds of the last key", serverMs < 5000, `${serverMs} ms`);
  await until(async () => (await page.locator("text=CANON-E013").count()) > 0, { timeout: 30000, label: "the mark in the window" });
  const windowMs = Math.round(performance.now() - lastKey);
  log({ markInWindowAfterLastKeyMs: windowMs });
  check("the mark appears in the window", windowMs < 8000, `${windowMs} ms`);
  await sleep(500);
  await page.screenshot({ path: join(out, whole ? "typing-whole-book-mark.png" : "typing-mark.png") });
  log({ totalSince: Date.now() - before });
} finally {
  await app.close();
}
const failed = results.filter((r) => !r.ok);
log({ summary: `${results.length - failed.length} of ${results.length} checks passed`, failed: failed.map((f) => `${f.name}: ${f.detail}`) });
process.exit(failed.length ? 1 : 0);
