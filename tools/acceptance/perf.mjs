// Acceptance, scenario 2: the numbers on a real novel, through the real app.
//
//   node tools/acceptance/perf.mjs [--exe path] [--book path]
//
// Imports a public-domain novel (Project Gutenberg, kept in the scratch folder,
// not in the repository) as a manuscript split into chapters, then times: the
// import; the first full check (cold) and repeated checks (warm); and the cost
// of one edit: saving a changed paragraph, then the check that follows it.
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { apiClient, args, deskSession, launchApp, logTo, stats } from "./lib.mjs";

const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-perf";
const out = args.out ?? "D:/verne-build/asten/evidence";
const book = args.book ?? "D:/verne-build/asten/scratch/gutenberg/pg1342.txt";
const log = logTo(join(out, "perf.log"));

const raw = readFileSync(book, "utf8");
const body = raw.slice(raw.indexOf("\n", raw.indexOf("*** START OF")) + 1, raw.indexOf("*** END OF")).trim();
const words = body.split(/\s+/).length;

const app = await launchApp({ exe, profile });
try {
  const session = await deskSession(app.page);
  const api = apiClient(session.origin, session.token);
  const project = await api.post("/projects", { name: "Performance", kind: "novel", description: null });
  const p = `/projects/${project.id}`;
  let t = performance.now();
  const root = await api.post(`${p}/import`, { filename: "pride.txt", format: "plain", body, parent_id: null, kind: "manuscript", encoding: "utf-8", split: "auto" });
  const importMs = Math.round(performance.now() - t);
  const docs = await api.get(`${p}/documents?fields=summary`);
  const chapters = docs.filter((d) => d.parent_id === root.id);
  log({ words, importMs, chapters: chapters.length });

  // Settle a few facts so the check has something to weigh.
  await api.post(`${p}/entities`, { kind: "character", name: "Elizabeth", aliases: ["Lizzy"], notes: "" });
  await api.post(`${p}/entities`, { kind: "character", name: "Darcy", aliases: [], notes: "" });
  await api.post(`${p}/facts`, { entity_key: "elizabeth", predicate: "hair_color", value: "brown", status: "confirmed" });
  await api.post(`${p}/facts`, { entity_key: "darcy", predicate: "hair_color", value: "black", status: "confirmed" });

  const cold = [];
  const t0 = performance.now();
  const first = await api.get(`${p}/story/diagnostics`);
  cold.push({ wallMs: Math.round(performance.now() - t0), engine: first.timings_ms, marks: first.diagnostics.length, blocks: first.total_blocks });
  log({ firstFullCheck: cold[0] });

  const warm = [];
  for (let i = 0; i < 5; i++) {
    t = performance.now();
    const d = await api.get(`${p}/story/diagnostics`);
    warm.push(Math.round(performance.now() - t));
    void d;
  }
  log({ warmChecksMs: warm, warm: stats(warm) });

  // One edit: change a paragraph in a middle chapter, save, then check again.
  const saves = [];
  const afters = [];
  for (let i = 0; i < 5; i++) {
    const target = chapters[10 + i];
    const doc = await api.get(`${p}/documents/${target.id}`);
    const content = structuredClone(doc.content_json);
    const para = content.content.find((b) => b.type === "paragraph" && b.content?.[0]?.text);
    para.content[0].text += ` Elizabeth shook out her black hair (${i}).`;
    t = performance.now();
    await api.put(`${p}/documents/${target.id}`, { content_json: content, expected_head_seq: doc.head_seq, actor: "author" });
    saves.push(Math.round(performance.now() - t));
    t = performance.now();
    const d = await api.get(`${p}/story/diagnostics`);
    afters.push(Math.round(performance.now() - t));
    void d;
  }
  log({ saveOneEditMs: saves, saveStats: stats(saves), checkAfterEditMs: afters, checkStats: stats(afters) });
  const sizeBytes = await app.page.evaluate(() => window.__TAURI_INTERNALS__.invoke("library_info").then((i) => i.database_bytes));
  log({ libraryBytes: sizeBytes });
} finally {
  await app.close();
}
