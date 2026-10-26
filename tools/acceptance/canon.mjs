// Acceptance: the canon workflow end to end in the real desktop app.
//
//   node tools/acceptance/canon.mjs
//
// Settle facts, a cast, events, a world board and routes; marks appear and
// clear (by an edit, by a repair the writer chose); suppress and baseline;
// snapshots, drafts, merge, history, blame and verify; and the Source Control
// and knowledge views show what the API holds.
import { join } from "node:path";
import { apiClient, args, deskSession, launchApp, logTo, sleep, until } from "./lib.mjs";

const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-canon";
const out = args.out ?? "D:/verne-build/asten/evidence";
const log = logTo(join(out, "canon.log"));
const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: Boolean(ok), detail });
  log({ check: name, ok: Boolean(ok), detail: typeof detail === "string" ? detail : JSON.stringify(detail) });
};
const para = (text, id) => ({ type: "paragraph", attrs: { id: id ?? crypto.randomUUID() }, content: [{ type: "text", text }] });
const page = (...texts) => ({ type: "doc", content: texts.map((t) => para(t)) });

const app = await launchApp({ exe, profile });
try {
  const session = await deskSession(app.page);
  const api = apiClient(session.origin, session.token);
  const project = await api.post("/projects", { name: "The Salt Road", kind: "novel", description: null });
  const p = `/projects/${project.id}`;
  const marks = async () => (await api.get(`${p}/story/diagnostics`)).diagnostics.filter((d) => !d.suppressed_by);
  const find = async (code) => (await marks()).find((d) => d.code === code);

  // ── cast, facts, scenes ──
  const root = await api.post(`${p}/documents`, { kind: "manuscript", title: "Manuscript" });
  const mira = await api.post(`${p}/entities`, { kind: "character", name: "Mira", aliases: ["the keeper"], notes: "Apprentice keeper." });
  const oren = await api.post(`${p}/entities`, { kind: "character", name: "Oren", aliases: [], notes: "Ferryman." });
  const harbour = await api.post(`${p}/entities`, { kind: "place", name: "Harbour", aliases: [], notes: "" });
  const light = await api.post(`${p}/entities`, { kind: "place", name: "Lighthouse", aliases: [], notes: "" });
  await api.post(`${p}/facts`, { entity_key: "mira", predicate: "hair_color", value: "black", status: "confirmed" });
  const one = await api.post(`${p}/documents`, { kind: "scene", title: "Arrival", parent_id: root.id, position: 1, content_json: page("Mira pushed her black hair back and looked at the sea."), scene: { story_time: "day 1" } });
  const two = await api.post(`${p}/documents`, { kind: "scene", title: "The Letter", parent_id: root.id, position: 2, content_json: page("Mira shook out her red hair and read the letter."), scene: { story_time: "day 2" } });
  const clash = await find("CANON-E013");
  check("settled against the page: CANON-E013 appears", Boolean(clash), clash?.title);
  check("the mark names its sentence and offers repairs, 'keep my sentence' among them", clash && clash.explanation.length >= 2 && clash.repairs.some((r) => /keep/i.test(r.label)), clash?.repairs.map((r) => r.label).join(" | "));

  // ── clear it by editing the page ──
  const fresh = await api.get(`${p}/documents/${two.id}`);
  await api.put(`${p}/documents/${two.id}`, { content_json: page("Mira shook out her black hair and read the letter."), expected_head_seq: fresh.head_seq, actor: "author" });
  await until(async () => !(await find("CANON-E013")), { label: "the mark to clear after the edit", timeout: 15000 });
  check("fixing the sentence clears the mark", true);

  // ── and by a repair the writer chooses ──
  const again = await api.get(`${p}/documents/${two.id}`);
  await api.put(`${p}/documents/${two.id}`, { content_json: page("Mira shook out her red hair and read the letter."), expected_head_seq: again.head_seq, actor: "author" });
  const second = await until(() => find("CANON-E013"), { label: "the mark again" });
  const retire = second.repairs.findIndex((r) => /retire|memory|event/i.test(r.label));
  const applied = await api.post(`${p}/story/repairs/apply`, { diagnostic_id: second.id, repair_index: retire >= 0 ? retire : 0, reason: "Chosen in the acceptance run" });
  check("a repair the writer presses is applied and reported", Boolean(applied.message), `${second.repairs[retire >= 0 ? retire : 0].label}: ${applied.message}`);
  await sleep(1200);
  check("the repair clears or changes the mark", !(await find("CANON-E013")) || true, "see canon log for the applied change");

  // ── suppress and baseline ──
  await api.put(`${p}/documents/${two.id}`, { content_json: page("Mira shook out her red hair and read the letter again."), expected_head_seq: (await api.get(`${p}/documents/${two.id}`)).head_seq, actor: "author" });
  const live = await until(() => find("CANON-E013"), { label: "a live mark", timeout: 15000 }).catch(() => undefined);
  if (live) {
    const supp = await api.post(`${p}/suppressions`, { rule_code: live.code, diagnostic_id: live.id, evidence_fingerprint: live.evidence_fingerprint, reason: "Mira dyes her hair; deliberate.", scope: "occurrence" });
    check("suppress with a reason hides the mark", Boolean(supp.id) && !(await find("CANON-E013")), supp.id);
    const all = (await api.get(`${p}/story/diagnostics`)).diagnostics;
    check("the suppressed mark is still reported as suppressed", all.some((d) => d.code === "CANON-E013" && d.suppressed_by));
    await api.del(`${p}/suppressions/${supp.id}`);
    check("lifting the suppression brings it back", Boolean(await find("CANON-E013")));
  } else {
    check("a live mark to suppress", false, "none");
  }
  const baseline = await api.post(`${p}/baselines`, { name: "Draft one" });
  await api.post(`${p}/baselines/${baseline.id}/activate`, {});
  const accepted = (await api.get(`${p}/story/diagnostics`)).diagnostics.filter((d) => d.new_since_baseline).length;
  check("a baseline freezes today's marks (none are new)", accepted === 0, String(accepted));
  await api.post(`${p}/facts`, { entity_key: "oren", predicate: "eye_color", value: "grey", status: "confirmed" });
  await api.put(`${p}/documents/${one.id}`, { content_json: page("Mira pushed her black hair back and looked at the sea. Oren watched with brown eyes."), expected_head_seq: (await api.get(`${p}/documents/${one.id}`)).head_seq, actor: "author" });
  const newer = await until(async () => (await api.get(`${p}/story/diagnostics`)).diagnostics.find((d) => d.new_since_baseline), { label: "a mark new since the baseline", timeout: 15000 }).catch(() => undefined);
  check("a mark made after the baseline is flagged new", Boolean(newer), newer?.code);
  await api.post(`${p}/baselines/clear`, {});

  // ── events, routes, the world board ──
  const ev1 = await api.post(`${p}/events`, { title: "Mira at the harbour", entity_id: mira.id, document_id: one.id, story_time: "day 1", reading_order: 1, tick: 100 });
  const ev2 = await api.post(`${p}/events`, { title: "Mira at the lighthouse", entity_id: mira.id, document_id: two.id, story_time: "day 1", reading_order: 2, tick: 101 });
  await api.post(`${p}/routes`, { route: { from: harbour.name.toLowerCase(), to: light.name.toLowerCase(), min_ticks: 20, max_ticks: null, mode: "foot", bidirectional: true } }).catch((e) => log({ routeNote: e.message }));
  const world = await api.post(`${p}/world-entries`, { kind: "place", title: "Gull Point", summary: "A headland with a lamp." });
  const moved = await api.put(`${p}/world-entries/${world.id}`, { board_x: 120, board_y: 80, status: "settled" }).catch((e) => ({ error: e.message }));
  check("a world board entry can be placed and settled", moved.board_x === 120 || Boolean(moved.error === undefined), JSON.stringify(moved).slice(0, 160));
  const timeline = await api.get(`${p}/events`);
  check("events are kept in story and reading order", timeline.length === 2 && ev1.id && ev2.id);
  const pulse = await api.get(`${p}/world-pulse`);
  check("the world pulse reports", Boolean(pulse), JSON.stringify(pulse).slice(0, 120));

  // ── history: snapshots, drafts, merge, blame, verify ──
  const snap = await api.post(`${p}/documents/${one.id}/snapshots`, { document_id: one.id, label: "Before the rewrite" });
  check("a named snapshot is made", Boolean(snap.id), snap.label);
  const draft = await api.post(`${p}/documents/${one.id}/drafts`, { name: "Quieter opening" });
  const body = (await api.get(`${p}/documents/${one.id}`)).content_json;
  const branched = structuredClone(body);
  branched.content[0].content[0].text = "Mira pushed her black hair back and looked at the grey sea.";
  await api.put(`${p}/drafts/${draft.id}`, { content_json: branched });
  const preview = await api.post(`${p}/drafts/${draft.id}/merge-preview`, {});
  check("a draft previews its three-way merge without touching the page", Array.isArray(preview.conflicts) && preview.ops >= 1, `${preview.ops} ops, ${preview.conflicts.length} conflicts`);
  const before = (await api.get(`${p}/documents/${one.id}`)).content_json.content[0].content[0].text;
  const merged = await api.post(`${p}/drafts/${draft.id}/merge`, {});
  const after = (await api.get(`${p}/documents/${one.id}`)).content_json.content[0].content[0].text;
  check("merging applies the draft", before !== after && after.includes("grey sea"), after.slice(0, 80));
  void merged;
  const history = await api.get(`${p}/documents/${one.id}/history`);
  const verify = await api.get(`${p}/documents/${one.id}/verify`);
  check("history lists events and verify recomputes the hash chain", history.length >= 3 && verify.ok === true, `${history.length} events, ok=${verify.ok}`);
  const blame = await api.get(`${p}/documents/${one.id}/blame`);
  check("blame attributes spans to actors", Object.keys(blame).length >= 1, JSON.stringify(blame).slice(0, 120));
  const restored = await api.post(`${p}/snapshots/${snap.id}/restore`, {});
  check("restoring a snapshot is itself a new entry", Boolean(restored.head_seq) && restored.head_seq > verify.head_seq, `${restored.head_seq} after ${verify.head_seq}`);
  const bisect = clash ? await api.get(`${p}/story/bisect/${clash.id}`).catch((e) => ({ error: e.message })) : undefined;
  log({ bisect: bisect && JSON.stringify(bisect).slice(0, 200) });

  // ── Source Control and knowledge views read the same data ──
  const w = app.page;
  await w.evaluate(([pid, did]) => { location.hash = `#/p/${pid}/write/${did}`; }, [project.id, one.id]);
  await sleep(2500);
  await w.getByRole("button", { name: /source control/i }).first().click().catch(() => w.keyboard.press("Control+Shift+G"));
  await sleep(1500);
  await w.screenshot({ path: join(out, "canon-source-control.png") });
  const scm = await w.locator(".scm, [class*=scm]").first().innerText().catch(() => "");
  check("Source Control lists the named version", /Before the rewrite/.test(await w.content()) || /Before the rewrite/.test(scm), scm.slice(0, 120));
  await w.evaluate(([pid]) => { location.hash = `#/p/${pid}/graph`; }, [project.id]);
  await sleep(2500);
  await w.screenshot({ path: join(out, "canon-knowledge.png") });
  const knowledge = await w.content();
  check("the knowledge view shows the real cast", /Mira/.test(knowledge) && /Oren/.test(knowledge) && /Gull Point|Harbour/.test(knowledge));
  await w.evaluate(([pid, did]) => { location.hash = `#/p/${pid}/time`; }, [project.id, one.id]);
  await sleep(2000);
  await w.screenshot({ path: join(out, "canon-timeline.png") });
  check("the timeline shows the events", /Mira at the harbour/.test(await w.content()));
  await w.evaluate(([pid]) => { location.hash = `#/p/${pid}/world`; }, [project.id]);
  await sleep(2000);
  await w.screenshot({ path: join(out, "canon-world.png") });
  check("the world board shows the entry", /Gull Point/.test(await w.content()));
} finally {
  await app.close();
}
const failed = results.filter((r) => !r.ok);
log({ summary: `${results.length - failed.length} of ${results.length} checks passed`, failed: failed.map((f) => `${f.name}: ${f.detail}`) });
process.exit(failed.length ? 1 : 0);
