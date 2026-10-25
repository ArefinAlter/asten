import { describe, expect, it } from "vitest";
import type { AstenDiagnostic, DocumentSummary } from "@asten/contracts";
import {
  ACTIVE_STATE,
  EDITOR_COMMANDS,
  UNAVAILABLE,
  resolveCommand,
  runEditorCommand,
  type ChainLike,
} from "./ribbon/commands";
import { RIBBON_TABS, ribbonCommandIds } from "./ribbon/model";
import { INITIAL_LAYOUT, LIMITS, layoutReducer, visiblePanels } from "./layout";
import {
  problemRows,
  readingOrder,
  stepMark,
  countBySeverity,
} from "./problems";
import {
  TOOLS,
  classifyRequest,
  decideToolCall,
  mayReadTextOf,
} from "./agent/tools";
import {
  parseCommand,
  runCommand,
  tokenize,
  type ConsoleEngine,
} from "./terminal";
import {
  chordOf,
  conflicts,
  effectiveBindings,
  matchBinding,
  normalizeChord,
} from "./keys";
import {
  buildVault,
  backlinks,
  graphOf,
  parseWikilinks,
  splitWikilinks,
  DEFAULT_FILTER,
} from "./knowledge/vault";
import { seedBodies, step, DEFAULT_FORCES } from "./knowledge/force";
import { commitsOf, diffParagraphs, diffSequences, pageStatus } from "./scm";
import { extractivePoints, respond, type AgentContext } from "./agent/script";

// ───────────────────────── ribbon command mapping ─────────────────────────

function fakeChain() {
  const calls: unknown[][] = [];
  const chain: ChainLike = new Proxy({} as ChainLike, {
    get(_, prop: string) {
      if (prop === "run") return () => true;
      if (prop === "missingStep") return undefined;
      return (...args: unknown[]) => {
        calls.push([prop, ...args]);
        return chain;
      };
    },
  });
  return { editor: { chain: () => chain }, calls };
}

describe("ribbon commands", () => {
  it("maps Bold, headings, alignment and scene break to Tiptap chain steps", () => {
    const { editor, calls } = fakeChain();
    expect(runEditorCommand(editor, "editor.bold")).toBe(true);
    expect(runEditorCommand(editor, "editor.styleHeading2")).toBe(true);
    expect(runEditorCommand(editor, "editor.alignJustify")).toBe(true);
    expect(runEditorCommand(editor, "editor.sceneBreak")).toBe(true);
    expect(calls).toEqual([
      ["focus"],
      ["toggleBold"],
      ["focus"],
      ["toggleHeading", { level: 2 }],
      ["focus"],
      ["setTextAlign", "justify"],
      ["focus"],
      ["insertContent", { type: "sceneBreak" }],
    ]);
  });

  it("refuses an unknown command and a chain without the method", () => {
    const { editor } = fakeChain();
    expect(runEditorCommand(editor, "editor.nothing")).toBe(false);
    expect(runEditorCommand(editor, "editor.bold", [["missingStep"]])).toBe(
      false,
    );
  });

  it("resolves every ribbon control to editor, app or a stated reason", () => {
    for (const id of ribbonCommandIds()) {
      const resolved = resolveCommand(id);
      if (resolved.kind === "unavailable")
        expect(resolved.reason.length).toBeGreaterThan(10);
      else if (resolved.kind === "editor")
        expect(EDITOR_COMMANDS[id]).toBeDefined();
      else expect(resolved.action).toBe(id);
    }
    expect(resolveCommand("editor.table").kind).toBe("unavailable");
    expect(resolveCommand("app.compile").kind).toBe("app");
  });

  it("has every tab Word and Publisher lead with, and no prose-writing command", () => {
    expect(RIBBON_TABS.map((t) => t.label)).toEqual([
      "Home",
      "Insert",
      "Layout",
      "Page Design",
      "References",
      "Review",
      "View",
      "Help",
    ]);
    const ids = ribbonCommandIds().join(" ");
    expect(ids).not.toMatch(/generate|rewrite|continue|autocomplete/i);
    for (const key of Object.keys(ACTIVE_STATE))
      expect(EDITOR_COMMANDS[key]).toBeDefined();
    for (const key of Object.keys(UNAVAILABLE))
      expect(EDITOR_COMMANDS[key]).toBeUndefined();
  });
});

// ───────────────────────── panel state ─────────────────────────

describe("layout", () => {
  it("toggles panels the way VS Code does", () => {
    let s = layoutReducer(INITIAL_LAYOUT, { type: "togglePanel" });
    expect(s.panel).toBe(false);
    s = layoutReducer(s, { type: "showPanel", tab: "terminal" });
    expect(s).toMatchObject({ panel: true, panelTab: "terminal" });
    s = layoutReducer(s, { type: "showPane", pane: "manuscript" });
    expect(s.sidebar).toBe(false); // pressing the active pane closes it
    s = layoutReducer(s, { type: "showPane", pane: "scm" });
    expect(s).toMatchObject({ sidebar: true, sidePane: "scm" });
  });

  it("clamps sizes and un-maximises on resize", () => {
    let s = layoutReducer(INITIAL_LAYOUT, { type: "toggleMaximizePanel" });
    expect(s.panelMaximized).toBe(true);
    s = layoutReducer(s, { type: "resize", target: "panel", size: 5 });
    expect(s.panelHeight).toBe(LIMITS.panel.min);
    expect(s.panelMaximized).toBe(false);
    s = layoutReducer(s, { type: "resize", target: "sidebar", size: 9999 });
    expect(s.sidebarWidth).toBe(LIMITS.sidebar.max);
    s = layoutReducer(s, { type: "zoomBy", delta: 500 });
    expect(s.zoom).toBe(LIMITS.zoom.max);
  });

  it("hides everything but the page in focus mode, without forgetting it", () => {
    const s = layoutReducer(INITIAL_LAYOUT, {
      type: "pageMode",
      mode: "focus",
    });
    expect(visiblePanels(s)).toEqual({
      sidebar: false,
      panel: false,
      secondary: false,
      ribbon: false,
    });
    const back = layoutReducer(s, { type: "pageMode", mode: "print" });
    expect(visiblePanels(back)).toEqual({
      sidebar: true,
      panel: true,
      secondary: true,
      ribbon: true,
    });
  });

  it("collapses the ribbon (Ctrl+F1)", () => {
    expect(
      layoutReducer(INITIAL_LAYOUT, { type: "toggleRibbon" }).ribbonCollapsed,
    ).toBe(true);
  });
});

// ───────────────────────── problems ─────────────────────────

const doc = (
  id: string,
  parent: string | null,
  position: number,
  title = id,
): DocumentSummary => ({
  id,
  parent_id: parent,
  position,
  title,
  project_id: "p",
  kind: parent ? "scene" : "chapter",
  revision: 1,
  exclude_from_compile: false,
  scene: { context: { kind: "narration" }, introduces: [] },
  head_seq: 1,
  chain_hash: "",
  words: 0,
  created_at: "",
  updated_at: "",
});

const mark = (
  id: string,
  severity: AstenDiagnostic["severity"],
  document: string,
  start: number,
  extra: Partial<AstenDiagnostic> = {},
): AstenDiagnostic => ({
  id,
  code: `CANON-E${id}`,
  severity,
  tier: "must",
  probability: 1,
  title: `t${id}`,
  message: `m${id}`,
  primary: {
    document,
    block: `${document}-p1`,
    start,
    end: start + 1,
    quote: "",
    label: "",
  },
  related: [],
  explanation: [],
  culprit: [],
  repairs: [],
  provenance: { rules: [], packs: [], revision: "", engine_version: "" },
  stale: false,
  suppressed_by: null,
  new_since_baseline: false,
  entity: null,
  details: "",
  evidence_fingerprint: "",
  ...extra,
});

describe("problems", () => {
  const documents = [
    doc("ch2", null, 2),
    doc("ch1", null, 1),
    doc("b", "ch1", 2, "Scene B"),
    doc("a", "ch1", 1, "Scene A"),
    doc("c", "ch2", 1),
  ];

  it("orders pages depth-first by binder position", () => {
    expect(readingOrder(documents)).toEqual(["ch1", "a", "b", "ch2", "c"]);
  });

  it("sorts by severity, then reading order, then offset", () => {
    const rows = problemRows(
      [
        mark("1", "warning", "a", 5),
        mark("2", "error", "c", 0),
        mark("3", "warning", "a", 1),
        mark("4", "warning", "b", 0),
        mark("5", "information", "a", 0),
      ],
      documents,
    );
    expect(rows.map((r) => r.id)).toEqual(["2", "3", "1", "4", "5"]);
    expect(rows[1]).toMatchObject({
      scene: "Scene A",
      chapter: "ch1",
      line: 1,
    });
  });

  it("hides suppressed marks unless asked, and filters by text", () => {
    const marks = [
      mark("1", "error", "a", 0, { suppressed_by: "s" }),
      mark("2", "warning", "b", 0),
    ];
    expect(problemRows(marks, documents).map((r) => r.id)).toEqual(["2"]);
    expect(
      problemRows(marks, documents, { showSuppressed: true }),
    ).toHaveLength(2);
    expect(
      problemRows(marks, documents, { filter: "scene b" }).map((r) => r.id),
    ).toEqual(["2"]);
    expect(
      countBySeverity(problemRows(marks, documents, { showSuppressed: true })),
    ).toMatchObject({ error: 1, warning: 1 });
  });

  it("steps through marks with wrap-around (F8)", () => {
    const rows = problemRows(
      [mark("1", "error", "a", 0), mark("2", "warning", "b", 0)],
      documents,
    );
    expect(stepMark(rows, undefined, 1)?.id).toBe("1");
    expect(stepMark(rows, "2", 1)?.id).toBe("1");
    expect(stepMark(rows, "1", -1)?.id).toBe("2");
  });
});

// ───────────────────────── agent tool access ─────────────────────────

describe("agent tool access", () => {
  it("refuses every prose tool at every access", () => {
    for (const tool of [
      "manuscript.write",
      "manuscript.rewrite",
      "manuscript.continue",
    ]) {
      expect(decideToolCall(tool, "read").allowed).toBe(false);
      expect(decideToolCall(tool, "propose").allowed).toBe(false);
    }
    expect(
      TOOLS.filter((t) => t.needs === "never").every((t) => t.closedBecause),
    ).toBe(true);
  });

  it("keeps the table closed and settling canon human-only", () => {
    expect(decideToolCall("shell.exec", "propose")).toMatchObject({
      allowed: false,
    });
    expect(decideToolCall("facts.settle", "propose").allowed).toBe(false);
  });

  it("gates proposals on thread access and honours switched-off tools", () => {
    expect(decideToolCall("marks.list", "read").allowed).toBe(true);
    expect(decideToolCall("facts.propose", "read").allowed).toBe(false);
    expect(decideToolCall("facts.propose", "propose").allowed).toBe(true);
    expect(
      decideToolCall("marks.list", "propose", new Set(["marks.list"])).allowed,
    ).toBe(false);
  });

  it("never reads a manuscript scene's text to summarise it", () => {
    expect(mayReadTextOf("sources.summarise", "scene")).toBe(false);
    expect(mayReadTextOf("sources.summarise", "chapter")).toBe(false);
    expect(mayReadTextOf("sources.summarise", "source")).toBe(true);
  });

  it("recognises requests for prose before anything else", () => {
    for (const text of [
      "Write the next scene",
      "Rewrite this paragraph so it flows",
      "continue chapter 3",
      "polish my opening",
      "Can you draft the ending?",
    ])
      expect(classifyRequest(text).kind).toBe("prose");
    expect(classifyRequest("Explain CANON-E013")).toEqual({
      kind: "explain",
      code: "CANON-E013",
    });
    expect(classifyRequest("run the checks").kind).toBe("checks");
    expect(classifyRequest("Which proposals are pending?").kind).toBe("facts");
    expect(classifyRequest("summarise the research on tide tables").kind).toBe(
      "source",
    );
    expect(classifyRequest("what is settled about Mara?").kind).toBe("canon");
  });

  it("answers a prose request with a refused call and no text for the page", async () => {
    const ctx = {
      access: "propose",
      disabledTools: new Set<string>(),
      chips: [],
    } as unknown as AgentContext;
    const turn = await respond("Rewrite the crossing scene", ctx);
    expect(turn.calls).toHaveLength(1);
    expect(turn.calls[0]).toMatchObject({
      tool: "manuscript.rewrite",
      allowed: false,
    });
    expect(turn.reply).toMatch(/never write, rewrite or continue/);
  });

  it("summarises sources extractively", () => {
    expect(
      extractivePoints("First one. Second.\n\nOther [[Link]] here. More."),
    ).toEqual(["First one.", "Other Link here."]);
  });
});

// ───────────────────────── console ─────────────────────────

describe("console", () => {
  it("tokenizes quotes and parses flags", () => {
    expect(tokenize(`asten snapshot "a b" 'c'`)).toEqual([
      "asten",
      "snapshot",
      "a b",
      "c",
    ]);
    expect(parseCommand("asten compile --format epub")).toEqual({
      command: "compile",
      args: [],
      flags: { format: "epub" },
    });
    expect(parseCommand("asten compile --format=fdx")).toMatchObject({
      flags: { format: "fdx" },
    });
  });

  it("is not a system shell", () => {
    for (const line of ["ls", "rm -rf /", "powershell", "cmd /c dir"])
      expect(parseCommand(line)).toMatchObject({
        error: expect.stringMatching(/not a system shell/),
      });
  });

  const engine: ConsoleEngine = {
    projectName: () => "Sample",
    diagnostics: async () => ({
      revision: "abcdef0123",
      diagnostics: [mark("1", "error", "a", 0)],
    }),
    documents: async () => [doc("a", null, 1, "Scene A")],
    compile: async (format) => ({
      filename: `x.${format}`,
      mime: "text/plain",
      encoding: "utf-8",
      body: "hello",
    }),
    packs: async () => [
      {
        id: "1",
        pack_key: "scribe",
        name: "Scribe",
        version: "0.1.0",
        summary: "",
        enabled: true,
        manifest: null,
        created_at: "",
      },
    ],
    setPack: async () => {
      throw new Error("no");
    },
    facts: async () => [],
    entities: async () => [],
    snapshot: async () => ({ id: "s1" }),
    openDocumentId: () => undefined,
  };

  it("runs check, compile and pack list against the engine", async () => {
    const check = await runCommand("asten check", engine);
    expect(check.lines.map((l) => l.text).join("\n")).toMatch(
      /CANON-E1 .*Scene A[\s\S]*1 error, 0 warnings/,
    );
    const compiled = await runCommand("asten compile --format epub", engine);
    expect(compiled.artifact?.filename).toBe("x.epub");
    expect(
      (await runCommand("asten compile --format pdf", engine)).lines[1].kind,
    ).toBe("err");
    expect(
      (await runCommand("asten pack disable scribe", engine)).lines[1].text,
    ).toMatch(/Scribe stays on/);
    expect(
      (await runCommand("asten snapshot hi", engine)).lines[1].text,
    ).toMatch(/Open a page/);
    expect((await runCommand("clear", engine)).clear).toBe(true);
  });
});

// ───────────────────────── keybindings ─────────────────────────

describe("keybindings", () => {
  const ev = (
    key: string,
    mods: Partial<{ ctrl: boolean; alt: boolean; shift: boolean }> = {},
  ) => ({
    key,
    ctrlKey: Boolean(mods.ctrl),
    metaKey: false,
    altKey: Boolean(mods.alt),
    shiftKey: Boolean(mods.shift),
  });

  it("normalises chords and reads events", () => {
    expect(normalizeChord("shift+ctrl+p")).toBe("Ctrl+Shift+P");
    expect(chordOf(ev("b", { ctrl: true, alt: true }))).toBe("Ctrl+Alt+B");
    expect(chordOf(ev("Shift", { shift: true }))).toBeNull();
  });

  it("finds conflicts after an override", () => {
    const bindings = effectiveBindings({ "layout.panel": "Ctrl+K" });
    expect(conflicts(bindings).get("Ctrl+K")).toEqual([
      "palette.open",
      "layout.panel",
    ]);
    expect(conflicts(effectiveBindings({})).size).toBe(0);
  });

  it("leaves Ctrl+B to Bold while the caret is in the page", () => {
    const bindings = effectiveBindings({});
    expect(
      matchBinding(bindings, ev("b", { ctrl: true }), false)?.command,
    ).toBe("layout.sidebar");
    expect(
      matchBinding(bindings, ev("b", { ctrl: true }), true),
    ).toBeUndefined();
    expect(matchBinding(bindings, ev("j", { ctrl: true }), true)?.command).toBe(
      "layout.panel",
    );
    expect(
      matchBinding(bindings, ev("F1", { ctrl: true }), false)?.command,
    ).toBe("layout.ribbon");
  });
});

// ───────────────────────── knowledge and source control ─────────────────────────

const page = (
  id: string,
  kind: "wiki" | "note" | "source" | "scene",
  title: string,
  text: string,
) =>
  ({
    ...doc(id, null, 1, title),
    kind,
    content_json: {
      type: "doc",
      content: text.split("\n").map((t) => ({
        type: "paragraph",
        content: [{ type: "text", text: t }],
      })),
    },
  }) as never;

describe("knowledge", () => {
  it("parses wikilinks with aliases and headings", () => {
    expect(
      parseWikilinks("See [[Mara Vell|Mara]] and [[Harbour#North]].").map(
        (l) => [l.target, l.shown],
      ),
    ).toEqual([
      ["Mara Vell", "Mara"],
      ["Harbour", "Harbour"],
    ]);
    expect(splitWikilinks("a [[B]] c")).toEqual([
      { text: "a " },
      { link: "B", shown: "B" },
      { text: " c" },
    ]);
  });

  it("builds notes, merges a wiki page into its cast member, and finds backlinks", () => {
    const entities = [
      {
        id: "e1",
        project_id: "p",
        kind: "character",
        name: "Mara Vell",
        aliases: ["Mara"],
        notes: "",
        created_at: "",
        updated_at: "",
      },
    ] as never;
    const vault = buildVault(
      [
        page("w", "wiki", "Mara Vell", "Keeper at [[Gull Point]]."),
        page("n", "note", "Gull Point", "Kept by [[Mara]]. See [[Nowhere]]."),
        page("s", "scene", "Opening", "Mara climbed."),
      ],
      entities,
    );
    expect(vault.byId.get("entity:e1")?.documentId).toBe("w");
    expect(
      backlinks(vault, "entity:e1")
        .map((b) => b.note.title)
        .sort(),
    ).toEqual(["Gull Point", "Opening"]);
    expect(vault.byId.get("unresolved:nowhere")?.kind).toBe("unresolved");
    const local = graphOf(vault, DEFAULT_FILTER, { id: "doc:n", depth: 1 });
    expect(local.nodes.map((n) => n.title).sort()).toEqual([
      "Gull Point",
      "Mara Vell",
      "Nowhere",
    ]);
    expect(
      graphOf(vault, { ...DEFAULT_FILTER, showUnresolved: false }).nodes.some(
        (n) => n.kind === "unresolved",
      ),
    ).toBe(false);
  });

  it("lays the graph out deterministically and stays bounded", () => {
    const a = seedBodies(["x", "y", "z"]);
    const b = seedBodies(["x", "y", "z"]);
    for (let i = 0; i < 200; i += 1) {
      step(a, [{ from: "x", to: "y" }], DEFAULT_FORCES);
      step(b, [{ from: "x", to: "y" }], DEFAULT_FORCES);
    }
    expect(a).toEqual(b);
    expect(
      a.every((body) => Number.isFinite(body.x) && Math.abs(body.x) < 2000),
    ).toBe(true);
  });
});

describe("source control", () => {
  it("diffs paragraphs and marks words inside a changed paragraph", () => {
    const changes = diffParagraphs(
      ["one", "two words here", "three"],
      ["one", "two new words here", "four"],
    );
    expect(changes.map((c) => c.kind)).toEqual(["same", "change", "change"]);
    expect(
      changes[1].words
        ?.filter((w) => w.op === "add")
        .map((w) => w.value.trim())
        .filter(Boolean),
    ).toEqual(["new"]);
  });

  it("falls back to replace past the bound (D11)", () => {
    const big = Array.from({ length: 600 }, (_, i) => `p${i}`);
    const ops = diffSequences(big, [...big].reverse());
    expect(ops.filter((o) => o.op === "same")).toHaveLength(0);
  });

  it("knows modified, untracked and clean pages, and groups commits", () => {
    const summary = doc("a", null, 1);
    const snap = (id: string, seq: number, at: string, label = "m") => ({
      id,
      document_id: "a",
      revision: 1,
      label,
      content_json: null,
      seq,
      chain_hash: "",
      merkle_root: "",
      created_at: at,
    });
    expect(pageStatus(summary, []).state).toBe("untracked");
    expect(
      pageStatus(summary, [snap("1", 1, "2026-01-01T00:00:00Z")]).state,
    ).toBe("clean");
    expect(
      pageStatus({ ...summary, head_seq: 3 }, [
        snap("1", 1, "2026-01-01T00:00:00Z"),
      ]).state,
    ).toBe("modified");
    const commits = commitsOf([
      snap("1", 1, "2026-01-01T00:00:00Z"),
      snap("2", 1, "2026-01-01T00:01:00Z"),
      snap("3", 1, "2026-01-02T00:00:00Z", "other"),
    ]);
    expect(commits.map((c) => c.snapshots.length)).toEqual([1, 2]);
  });
});
