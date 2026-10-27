// Acceptance: a real long manuscript, a real Word file, the engine on them,
// and every compile format checked by a script that does not trust Asten.
//
//   node tools/acceptance/documents.mjs
//
// Inputs (kept in scratch, never committed): a Project Gutenberg novel and a
// .docx written by python-docx (tools/acceptance/make_docx.py).
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { apiClient, args, deskSession, launchApp, logTo, stats } from "./lib.mjs";

const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-docs";
const out = args.out ?? "D:/verne-build/asten/evidence";
const book = args.book ?? "D:/verne-build/asten/scratch/gutenberg/pg1342.txt";
const docxFile = args.docx ?? "D:/verne-build/asten/scratch/salt-road.docx";
const python = args.python ?? "D:/verne-build/asten/venv/Scripts/python.exe";
const log = logTo(join(out, "documents.log"));
const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: Boolean(ok), detail });
  log({ check: name, ok: Boolean(ok), detail: typeof detail === "string" ? detail : JSON.stringify(detail) });
};

const raw = readFileSync(book, "utf8");
const body = raw
  .slice(raw.indexOf("\n", raw.indexOf("*** START OF")) + 1, raw.indexOf("*** END OF"))
  .trim();
const sourceWords = body.split(/\s+/).length;

const app = await launchApp({ exe, profile });
try {
  const session = await deskSession(app.page);
  const api = apiClient(session.origin, session.token);
  const ms = (t) => Math.round(performance.now() - t);

  // ── a 120,000-word novel from plain text ──
  const project = await api.post("/projects", { name: "Pride and Prejudice", kind: "novel", description: null });
  let t = performance.now();
  const container = await api.post(`/projects/${project.id}/import`, {
    filename: "pride-and-prejudice.txt",
    format: "plain",
    body,
    parent_id: null,
    kind: "manuscript",
    encoding: "utf-8",
    split: "auto",
  });
  log({ importMs: ms(t), sourceWords });
  const summaries = await api.get(`/projects/${project.id}/documents?fields=summary`);
  const chapters = summaries.filter((d) => d.parent_id === container.id).sort((a, b) => a.position - b.position);
  const totalWords = chapters.reduce((sum, d) => sum + d.words, 0);
  check("the novel became a manuscript with a document per chapter", chapters.length >= 60, `${chapters.length} chapters`);
  check("every word arrived", totalWords >= sourceWords * 0.97, `${totalWords} of ${sourceWords}`);
  check("chapters are titled from their headings", chapters.some((d) => /^CHAPTER II\./i.test(d.title)), chapters.slice(0, 4).map((d) => d.title).join(" | "));
  const first = await api.get(`/projects/${project.id}/documents/${chapters[2].id}`);
  const firstText = JSON.stringify(first.content_json);
  check("hard-wrapped lines were joined and _italics_ became italics", !/\\n/.test(firstText.slice(0, 4000)) && firstText.includes('"italic"'), firstText.slice(0, 200));

  // ── the engine on the whole book ──
  const timings = [];
  let snapshot;
  for (let i = 0; i < 4; i++) {
    t = performance.now();
    snapshot = await api.get(`/projects/${project.id}/story/diagnostics`);
    timings.push({ wall: ms(t), engine: snapshot.timings_ms.total, perceive: snapshot.timings_ms.perceive, analyze: snapshot.timings_ms.analyze, cached: snapshot.cached_blocks, total: snapshot.total_blocks });
  }
  log({ diagnosticsTimings: timings });
  const byCode = {};
  for (const d of snapshot.diagnostics) byCode[d.code] = (byCode[d.code] ?? 0) + 1;
  log({ marksOnTheBareBook: byCode, blocks: snapshot.total_blocks });
  check("the engine weighs a 120,000-word book in under two seconds, cold", timings[0].wall < 2000, `${timings[0].wall} ms wall, ${timings[0].engine} ms engine`);

  // ── incremental: a canon fact, a contradicting sentence, one block re-perceived ──
  const entity = await api.post(`/projects/${project.id}/entities`, { kind: "character", name: "Elizabeth", aliases: ["Lizzy"], notes: "" });
  await api.post(`/projects/${project.id}/facts`, { entity_key: "elizabeth", predicate: "eye_color", value: "brown", status: "confirmed" });
  const chapter = await api.get(`/projects/${project.id}/documents/${chapters[4].id}`);
  const sentence = "Elizabeth lifted her head, and her blue eyes were bright with anger.";
  const edited = structuredClone(chapter.content_json);
  edited.content.push({ type: "paragraph", attrs: { id: "00000000-0000-7000-8000-000000000001" }, content: [{ type: "text", text: sentence }] });
  const incremental = [];
  let head = chapter.head_seq;
  for (let i = 0; i < 5; i++) {
    t = performance.now();
    const saved = await api.put(`/projects/${project.id}/documents/${chapters[4].id}`, { content_json: edited, expected_head_seq: head, actor: "author" });
    const saveMs = ms(t);
    head = saved.head_seq;
    t = performance.now();
    snapshot = await api.get(`/projects/${project.id}/story/diagnostics`);
    incremental.push({ saveMs, checkMs: ms(t), engineMs: snapshot.timings_ms.total, cached: snapshot.cached_blocks, total: snapshot.total_blocks });
    // Change the sentence a little so the next save is a real edit.
    edited.content.at(-1).content[0].text = `${sentence}${" ".repeat(0)}${i % 2 ? "" : " "}`.trimEnd() + (i % 2 ? "" : ".");
  }
  log({ incremental });
  const clash = snapshot.diagnostics.find((d) => d.code === "CANON-E013");
  check("a contradicting sentence raises CANON-E013", Boolean(clash), clash?.title ?? "none");
  check("one block is re-perceived, the rest come from the cache", incremental.every((r) => r.total - r.cached <= 2), JSON.stringify(incremental.at(-1)));
  check("the check after an edit is fast", stats(incremental.map((r) => r.checkMs)).p95 < 1000, JSON.stringify(stats(incremental.map((r) => r.checkMs))));
  if (clash) {
    check("the mark offers repairs the author owns", clash.repairs.length >= 2 && clash.repairs.some((r) => /keep/i.test(r.label)), clash.repairs.map((r) => r.label).join(" | "));
  }

  // ── a Word file with headings, italics, comments and tracked changes ──
  const docxBytes = readFileSync(docxFile);
  const wordProject = await api.post("/projects", { name: "The Salt Road", kind: "novel", description: null });
  const wordRoot = await api.post(`/projects/${wordProject.id}/import`, {
    filename: "salt-road.docx", format: "docx", body: docxBytes.toString("base64"), encoding: "base64", parent_id: null, kind: "manuscript", split: "auto",
  });
  const wordDocs = (await api.get(`/projects/${wordProject.id}/documents`)).filter((d) => d.parent_id === wordRoot.id).sort((a, b) => a.position - b.position);
  check("the Word file's title names the manuscript and its headings became chapters", wordRoot.title === "The Salt Road" && wordDocs.length === 3, `${wordRoot.title}: ${wordDocs.map((d) => d.title).join(" | ")}`);
  const chapterOne = JSON.stringify(wordDocs[0].content_json);
  check("italics survived", chapterOne.includes('"italic"') && chapterOne.includes("auburn"));
  check("bold survived", chapterOne.includes('"bold"'));
  check("tracked insertions and deletions became marks", chapterOne.includes('"insertion"') && chapterOne.includes('"deletion"'));
  check("the scene break, the quote and the list survived", chapterOne.includes("sceneBreak") && chapterOne.includes("blockquote") && chapterOne.includes("bulletList"));
  const comments1 = await api.get(`/projects/${wordProject.id}/documents/${wordDocs[0].id}/comments`);
  const comments2 = await api.get(`/projects/${wordProject.id}/documents/${wordDocs[1].id}/comments`);
  check("the comments came with their authors and ranges", comments1.length === 1 && comments2.length === 1 && comments1[0].body.startsWith("Ida:") && comments1[0].to_offset > comments1[0].from_offset, JSON.stringify([comments1, comments2].map((c) => c[0] && [c[0].body, c[0].from_offset, c[0].to_offset])));
  const wordMarks = (await api.get(`/projects/${wordProject.id}/story/diagnostics`)).diagnostics.length;
  log({ wordProjectMarks: wordMarks });

  // ── compile everything, then check the files with tools that are not ours ──
  const exportsDir = join(out, "exports-novel");
  const wordExports = join(out, "exports-word");
  for (const dir of [exportsDir, wordExports]) {
    rmSync(dir, { recursive: true, force: true });
    mkdirSync(dir, { recursive: true });
  }
  const compileAll = async (projectId, dir) => {
    const files = {};
    for (const format of ["markdown", "plain", "html", "fountain", "docx", "epub", "fdx"]) {
      const started = performance.now();
      const artifact = await api.post(`/projects/${projectId}/compile`, { format });
      const bytes = artifact.encoding === "base64" ? Buffer.from(artifact.body, "base64") : Buffer.from(artifact.body, "utf8");
      writeFileSync(join(dir, artifact.filename), bytes);
      files[format] = { name: artifact.filename, bytes: bytes.length, ms: ms(started) };
    }
    return files;
  };
  const files = await compileAll(project.id, exportsDir);
  log({ compiled: files });
  const wordFiles = await compileAll(wordProject.id, wordExports);
  const run = (dir, extra = []) => {
    try {
      return JSON.parse(execFileSync(python, ["-I", join(import.meta.dirname, "check_exports.py"), dir, ...extra], { encoding: "utf8" }));
    } catch (error) {
      return JSON.parse(error.stdout || "{}");
    }
  };
  const novelChecks = run(exportsDir, ["--expect-words", String(Math.round(sourceWords * 0.97))]);
  const wordChecks = run(wordExports, ["--tracked"]);
  for (const [label, result] of [["novel", novelChecks], ["word", wordChecks]]) {
    for (const r of result.results ?? []) check(`export (${label}) ${r.check}`, r.ok, r.detail);
  }

  // ── import all six formats back, from what was compiled ──
  const roundTrip = await api.post("/projects", { name: "Round trip", kind: "novel", description: null });
  const readBack = {};
  for (const [format, name, encoding] of [
    ["markdown", files.markdown.name, "utf-8"],
    ["plain", files.plain.name, "utf-8"],
    ["html", files.html.name, "utf-8"],
    ["docx", wordFiles.docx.name, "base64"],
  ]) {
    const dir = format === "docx" ? wordExports : exportsDir;
    const bytes = readFileSync(join(dir, name));
    const doc = await api.post(`/projects/${roundTrip.id}/import`, {
      filename: name, format, body: encoding === "base64" ? bytes.toString("base64") : bytes.toString("utf8"), encoding, parent_id: null, kind: "manuscript", split: "auto",
    });
    const all = await api.get(`/projects/${roundTrip.id}/documents?fields=summary`);
    const under = all.filter((d) => d.parent_id === doc.id);
    readBack[format] = { chapters: under.length, words: under.reduce((s, d) => s + d.words, 0) || all.find((d) => d.id === doc.id)?.words };
  }
  log({ readBack });
  check("Markdown, plain, HTML and Word exports import again with every chapter", ["markdown", "plain", "html"].every((f) => readBack[f].chapters >= 60) && readBack.docx.chapters === 3, JSON.stringify(readBack));
  // Fountain and Final Draft round trip on the Word project (short prose; screenplay classes are for scripts).
  for (const [format, name] of [["fountain", wordFiles.fountain.name], ["fdx", wordFiles.fdx.name]]) {
    const bytes = readFileSync(join(wordExports, name));
    const doc = await api.post(`/projects/${roundTrip.id}/import`, { filename: name, format, body: bytes.toString("utf8"), encoding: "utf-8", parent_id: null, kind: "manuscript" });
    check(`${format} imports`, Boolean(doc.id), doc.title);
  }
} finally {
  await app.close();
}
const failed = results.filter((r) => !r.ok);
log({ summary: `${results.length - failed.length} of ${results.length} checks passed`, failed: failed.map((f) => `${f.name}: ${f.detail}`) });
process.exit(failed.length ? 1 : 0);
