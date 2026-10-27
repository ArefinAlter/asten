// Probe: import a long public-domain novel as one plain-text document with the
// current importer, then time the engine. Used to see what the importer does
// with real text before the importer is improved.
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { apiClient, args, deskSession, launchApp, logTo, stats } from "./lib.mjs";

const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-probe";
const out = args.out ?? "D:/verne-build/asten/evidence";
const book = args.book ?? "D:/verne-build/asten/scratch/gutenberg/pg1342.txt";
const log = logTo(join(out, "probe-long.log"));

const raw = readFileSync(book, "utf8");
const start = raw.indexOf("*** START OF");
const end = raw.indexOf("*** END OF");
const text = raw.slice(raw.indexOf("\n", start) + 1, end).trim();

const app = await launchApp({ exe, profile });
try {
  const session = await deskSession(app.page);
  const api = apiClient(session.origin, session.token);
  const project = await api.post("/projects", { name: "Probe", kind: "novel", description: null });
  let t = performance.now();
  const doc = await api.post(`/projects/${project.id}/import`, {
    filename: "pride.txt",
    format: "plain",
    body: text,
    parent_id: null,
    kind: "manuscript",
  });
  log({ importMs: Math.round(performance.now() - t), words: text.split(/\s+/).length, docId: doc.id });
  const got = await api.get(`/projects/${project.id}/documents/${doc.id}`);
  const blocks = got.content_json.content;
  log({ blocks: blocks.length, firstBlock: JSON.stringify(blocks[0]).slice(0, 200), sample: JSON.stringify(blocks[40]).slice(0, 300) });
  const times = [];
  for (let i = 0; i < 3; i++) {
    t = performance.now();
    const d = await api.get(`/projects/${project.id}/story/diagnostics`);
    times.push(Math.round(performance.now() - t));
    log({ diag: d.diagnostics.length, timings: d.timings_ms, cached: d.cached_blocks, total: d.total_blocks });
  }
  log({ diagnosticsMs: times });
} finally {
  await app.close();
}
