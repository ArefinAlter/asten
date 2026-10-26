/**
 * The assistant's turn: read the request, plan tool calls from the closed
 * table, run the allowed ones against the desk's own data, and answer with
 * plain words and cards a person presses. No model is called in this
 * release: the planning is these rules and the words are written from what
 * the tools returned. It never produces manuscript prose.
 */
import type {
  AstenDiagnostic,
  CanonEntity,
  DocumentSummary,
  Fact,
  TimelineEvent,
} from "@asten/contracts";
import {
  backlinks,
  outgoing,
  type Vault,
  type VaultNote,
} from "../knowledge/vault";
import {
  classifyRequest,
  decideToolCall,
  type Access,
  type Intent,
} from "./tools";

export interface AgentContext {
  projectName?: string;
  access: Access;
  disabledTools: ReadonlySet<string>;
  /** Context chips the person attached: a mark, a scene, a note. */
  chips: Chip[];
  diagnostics(): Promise<AstenDiagnostic[]>;
  rerunChecks(): Promise<AstenDiagnostic[]>;
  documents(): Promise<DocumentSummary[]>;
  facts(): Promise<Fact[]>;
  entities(): Promise<CanonEntity[]>;
  events(): Promise<TimelineEvent[]>;
  vault(): Promise<Vault>;
}

export interface Chip {
  kind: "mark" | "scene" | "note" | "selection";
  id: string;
  label: string;
  /** For a picked passage: the words themselves and where they came from. */
  text?: string;
  documentId?: string;
}

/** A fact the model proposed, as the card shows it. */
export interface ProposedFact {
  entity: string;
  entity_key: string;
  entity_id: string | null;
  predicate: string;
  value: string;
  reason: string | null;
  exists: boolean;
}

export interface ToolCall {
  tool: string;
  input: string;
  allowed: boolean;
  reason?: string;
  output?: string;
  ms: number;
}

export type Card =
  | { type: "marks"; marks: AstenDiagnostic[]; titles: Record<string, string> }
  | {
      type: "explain";
      mark: AstenDiagnostic;
      title: string;
      canRepair: boolean;
    }
  | {
      type: "facts";
      facts: Fact[];
      names: Record<string, string>;
      canPropose: boolean;
    }
  | { type: "canon"; entity: CanonEntity; facts: Fact[] }
  | { type: "graph"; note: VaultNote; out: VaultNote[]; back: VaultNote[] }
  | { type: "source"; note: VaultNote; points: string[]; links: string[] }
  | {
      type: "timeline";
      events: TimelineEvent[];
      titles: Record<string, string>;
    }
  | { type: "proposed"; items: ProposedFact[] }
  | { type: "rules" };

export interface Turn {
  intent: Intent;
  calls: ToolCall[];
  reply: string;
  cards: Card[];
}

async function call<T>(
  calls: ToolCall[],
  ctx: AgentContext,
  tool: string,
  input: string,
  run: () => Promise<T>,
  summarise: (value: T) => string,
): Promise<T | undefined> {
  const decision = decideToolCall(tool, ctx.access, ctx.disabledTools);
  const started = performance.now();
  if (!decision.allowed) {
    calls.push({ tool, input, allowed: false, reason: decision.reason, ms: 0 });
    return undefined;
  }
  try {
    const value = await run();
    calls.push({
      tool,
      input,
      allowed: true,
      output: summarise(value),
      ms: Math.max(1, Math.round(performance.now() - started)),
    });
    return value;
  } catch (error) {
    calls.push({
      tool,
      input,
      allowed: true,
      output: `failed: ${error instanceof Error ? error.message : String(error)}`,
      ms: Math.round(performance.now() - started),
    });
    return undefined;
  }
}

const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;

/** First sentence of each paragraph, links kept: an extractive summary, not a generated one. */
export function extractivePoints(text: string, max = 5): string[] {
  return text
    .split("\n")
    .map((p) => p.trim())
    .filter(Boolean)
    .map((p) =>
      (/^(.+?[.!?])(\s|$)/.exec(p)?.[1] ?? p).replace(
        /\[\[([^\]|]+)(\|[^\]]+)?\]\]/g,
        "$1",
      ),
    )
    .slice(0, max);
}

function findNote(
  vault: Vault,
  query: string,
  prefer?: (n: VaultNote) => boolean,
): VaultNote | undefined {
  const q = query
    .toLowerCase()
    .replace(/[?.!'’s]+$/g, "")
    .trim();
  const candidates = vault.notes.filter((n) => n.kind !== "unresolved");
  const pool = prefer ? candidates.filter(prefer) : candidates;
  return (
    pool.find((n) => n.title.toLowerCase() === q) ??
    pool.find((n) => q && q.includes(n.title.toLowerCase())) ??
    pool.find((n) => n.aliases.some((a) => q.includes(a.toLowerCase()))) ??
    pool.find((n) => q && n.title.toLowerCase().includes(q))
  );
}

export async function respond(text: string, ctx: AgentContext): Promise<Turn> {
  const intent = classifyRequest(text);
  const calls: ToolCall[] = [];
  const cards: Card[] = [];
  const markChip = ctx.chips.find((c) => c.kind === "mark");

  if (intent.kind === "prose") {
    const tool =
      intent.verb === "continue" ||
      intent.verb === "finish" ||
      intent.verb === "autocomplete" ||
      intent.verb === "complete"
        ? "manuscript.continue"
        : ["write", "draft", "generate", "compose", "expand"].includes(
              intent.verb,
            )
          ? "manuscript.write"
          : "manuscript.rewrite";
    await call(
      calls,
      ctx,
      tool,
      text,
      async () => null,
      () => "",
    );
    cards.push({ type: "rules" });
    return {
      intent,
      calls,
      cards,
      reply:
        "I can't do that one, and no setting changes it: I never write, rewrite or continue your prose. The sentences stay yours.\n\nWhat I can do for this page: run the checks on it, explain any mark on it, show what is settled about the names in it, or find where a fact was first settled.",
    };
  }

  if (intent.kind === "explain" || (intent.kind === "help" && markChip)) {
    const marks = await call(
      calls,
      ctx,
      "marks.list",
      "open marks",
      () => ctx.diagnostics(),
      (m) => plural(m.length, "mark"),
    );
    const code = intent.kind === "explain" ? intent.code : undefined;
    const mark =
      marks?.find((m) => m.id === markChip?.id) ??
      marks?.find((m) => code && m.code === code) ??
      marks?.find((m) => !m.suppressed_by);
    if (!mark)
      return {
        intent,
        calls,
        cards,
        reply: marks
          ? "There is no open mark to explain. The scales balance."
          : "I could not read the marks.",
      };
    await call(
      calls,
      ctx,
      "marks.explain",
      mark.id,
      async () => mark,
      (m) =>
        `${m.code}, ${plural(m.explanation.length, "statement")}, ${plural(m.repairs.length, "repair")}`,
    );
    const documents = (await ctx.documents()) ?? [];
    const title =
      documents.find((d) => d.id === mark.primary?.document)?.title ??
      "the project";
    const canRepair = decideToolCall(
      "repairs.propose",
      ctx.access,
      ctx.disabledTools,
    ).allowed;
    if (canRepair)
      await call(
        calls,
        ctx,
        "repairs.propose",
        mark.id,
        async () => mark.repairs,
        (r) => plural(r.length, "option"),
      );
    cards.push({ type: "explain", mark, title, canRepair });
    return {
      intent,
      calls,
      cards,
      reply: `**${mark.code} · ${mark.title}** in “${title}”.\n\n${mark.message}\n\n${canRepair ? "The card lists the repairs the rule offers. Press one if you want it; the choice is yours, and keeping your sentence is always an option." : "This thread is Read only, so I show the repairs without a button. Open the mark in Problems to act on it, or switch the thread to Propose."}`,
    };
  }

  if (intent.kind === "checks" || intent.kind === "marks") {
    const marks =
      intent.kind === "checks"
        ? await call(
            calls,
            ctx,
            "checks.run",
            ctx.projectName ?? "project",
            () => ctx.rerunChecks(),
            (m) => `${plural(m.length, "mark")} after weighing`,
          )
        : await call(
            calls,
            ctx,
            "marks.list",
            "open marks",
            () => ctx.diagnostics(),
            (m) => plural(m.length, "mark"),
          );
    if (!marks)
      return {
        intent,
        calls,
        cards,
        reply: "That call was refused, so I have nothing to report.",
      };
    const open = marks.filter((m) => !m.suppressed_by);
    const documents = await ctx.documents();
    const titles = Object.fromEntries(documents.map((d) => [d.id, d.title]));
    if (open.length) cards.push({ type: "marks", marks: open, titles });
    const errors = open.filter((m) => m.severity === "error").length;
    return {
      intent,
      calls,
      cards,
      reply: open.length
        ? `${intent.kind === "checks" ? "Weighed again. " : ""}${plural(open.length, "open mark")}, ${plural(errors, "error")}. The worst first: **${open[0].code}** in “${titles[open[0].primary?.document ?? ""] ?? "the project"}”. Press a row to jump to it, or ask me to explain one.`
        : "No open marks. The scales balance.",
    };
  }

  if (intent.kind === "facts") {
    const facts = await call(
      calls,
      ctx,
      "canon.search",
      "status:proposed",
      () => ctx.facts(),
      (f) =>
        plural(f.filter((x) => x.status === "proposed").length, "proposal"),
    );
    const entities = await ctx.entities();
    const names = Object.fromEntries(entities.map((e) => [e.id, e.name]));
    const proposed = (facts ?? []).filter((f) => f.status === "proposed");
    const canPropose = Boolean(
      await call(
        calls,
        ctx,
        "facts.propose",
        plural(proposed.length, "fact"),
        async () => true,
        () => "card prepared",
      ),
    );
    if (proposed.length)
      cards.push({ type: "facts", facts: proposed, names, canPropose });
    return {
      intent,
      calls,
      cards,
      reply: proposed.length
        ? `${plural(proposed.length, "fact")} read from the page and waiting for you. ${canPropose ? "Settle the ones you mean; reject the rest. Nothing becomes canon until you press." : "This thread is Read only, so the card has no buttons. Settle them in Canon, or switch the thread to Propose."}`
        : "Nothing is waiting to be settled.",
    };
  }

  if (intent.kind === "source") {
    const vault = await call(
      calls,
      ctx,
      "graph.search",
      "kind:source",
      () => ctx.vault(),
      (v) =>
        plural(
          v.notes.filter((n) => n.kind === "source" || n.kind === "note")
            .length,
          "note",
        ),
    );
    if (!vault)
      return { intent, calls, cards, reply: "That call was refused." };
    const note =
      findNote(
        vault,
        intent.query,
        (n) => n.kind === "source" || n.kind === "note",
      ) ?? vault.notes.find((n) => n.kind === "source");
    if (!note)
      return {
        intent,
        calls,
        cards,
        reply: "There is no source note to summarise yet.",
      };
    const points = await call(
      calls,
      ctx,
      "sources.summarise",
      note.title,
      async () => extractivePoints(note.text),
      (p) => plural(p.length, "point"),
    );
    if (!points)
      return { intent, calls, cards, reply: "That call was refused." };
    const links = outgoing(vault, note.id).map((n) => n.title);
    cards.push({ type: "source", note, points, links });
    return {
      intent,
      calls,
      cards,
      reply: `“${note.title}”, in ${plural(points.length, "point")}. The summary takes the opening sentence of each paragraph; it is extractive, and no model was called.`,
    };
  }

  if (intent.kind === "graph") {
    const vault = await call(
      calls,
      ctx,
      "graph.search",
      intent.query || "links",
      () => ctx.vault(),
      (v) =>
        `${plural(v.notes.length, "note")}, ${plural(v.links.length, "link")}`,
    );
    if (!vault)
      return { intent, calls, cards, reply: "That call was refused." };
    const chipNote = ctx.chips.find((c) => c.kind === "note");
    const note =
      (chipNote && vault.byId.get(chipNote.id)) ||
      findNote(vault, intent.query);
    if (!note)
      return {
        intent,
        calls,
        cards,
        reply: "I could not find that note. Try its exact title.",
      };
    const out = outgoing(vault, note.id);
    const back = backlinks(vault, note.id).map((b) => b.note);
    cards.push({ type: "graph", note, out, back });
    return {
      intent,
      calls,
      cards,
      reply: `“${note.title}” links to ${plural(out.length, "note")} and is linked from ${plural(back.length, "note")}.`,
    };
  }

  if (intent.kind === "timeline") {
    const events = await call(
      calls,
      ctx,
      "timeline.list",
      "story time",
      () => ctx.events(),
      (e) => plural(e.length, "event"),
    );
    if (!events)
      return { intent, calls, cards, reply: "That call was refused." };
    const documents = await ctx.documents();
    const titles = Object.fromEntries(documents.map((d) => [d.id, d.title]));
    const sorted = [...events].sort((a, b) => (a.tick ?? 0) - (b.tick ?? 0));
    cards.push({ type: "timeline", events: sorted, titles });
    return {
      intent,
      calls,
      cards,
      reply: `${plural(sorted.length, "event")} in story time. Reading order differs where a scene steps back; the Problems panel says where that matters.`,
    };
  }

  if (intent.kind === "canon") {
    const entities = await call(
      calls,
      ctx,
      "canon.search",
      intent.query,
      () => ctx.entities(),
      (e) => plural(e.length, "name"),
    );
    if (!entities)
      return { intent, calls, cards, reply: "That call was refused." };
    const q = intent.query.toLowerCase();
    const entity =
      entities.find((e) => q.includes(e.name.toLowerCase())) ??
      entities.find((e) =>
        e.aliases.some((a) => q.includes(a.toLowerCase())),
      ) ??
      entities.find((e) =>
        e.name
          .toLowerCase()
          .split(" ")
          .some((part) => part.length > 2 && q.includes(part)),
      );
    if (!entity)
      return {
        intent,
        calls,
        cards,
        reply: `No name in the canon matches “${intent.query}”. The cast is in the Knowledge view.`,
      };
    const facts = (await ctx.facts()).filter(
      (f) =>
        (f.entity_key === entity.id ||
          f.entity_key ===
            entity.name.trim().toLowerCase().split(/\s+/).join("_")) &&
        f.status !== "rejected",
    );
    cards.push({ type: "canon", entity, facts });
    const settled = facts.filter((f) => f.status === "confirmed");
    return {
      intent,
      calls,
      cards,
      reply: `${entity.name}: ${plural(settled.length, "settled fact")}${facts.length > settled.length ? `, ${facts.length - settled.length} proposed` : ""}.`,
    };
  }

  cards.push({ type: "rules" });
  return {
    intent,
    calls,
    cards,
    reply:
      "I'm the editor's assistant. Ask me to run the checks, explain a mark (try “explain CANON-E013”), search the canon (“what is settled about Mara?”), follow links in the knowledge graph, list proposals waiting to be settled, or summarise a source note. I don't write or rewrite your prose.",
  };
}
