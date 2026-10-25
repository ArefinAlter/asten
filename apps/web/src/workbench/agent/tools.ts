/**
 * The assistant's closed tool table and the rules that decide every call.
 *
 * Asten's assistant is an editor, never a writer: it reads, checks, searches,
 * explains and proposes; a person presses every card that changes anything;
 * and no tool writes, rewrites or continues manuscript prose, at any access
 * level, with or without a model. These rules live here, in data, so they are
 * tested (`tools.test.ts`) and shown to the author as they are.
 */

export type Access = "read" | "propose";

export interface ToolSpec {
  name: string;
  title: string;
  description: string;
  /** The lowest thread access that may call it; `never` means no access does. */
  needs: Access | "never";
  /** Why a `never` tool is closed. */
  closedBecause?: string;
}

export const TOOLS: ToolSpec[] = [
  {
    name: "marks.list",
    title: "List continuity marks",
    description: "Reads the open marks with their codes and places.",
    needs: "read",
  },
  {
    name: "marks.explain",
    title: "Explain a mark",
    description: "Reads one mark's evidence chain and the repairs it offers.",
    needs: "read",
  },
  {
    name: "checks.run",
    title: "Run the checks",
    description:
      "Asks the engine to weigh the project again. Reads only; marks are computed, never written by the assistant.",
    needs: "read",
  },
  {
    name: "canon.search",
    title: "Search the canon",
    description: "Searches settled and proposed facts and the cast.",
    needs: "read",
  },
  {
    name: "graph.search",
    title: "Search the knowledge graph",
    description:
      "Follows wikilinks and backlinks between notes, characters and places.",
    needs: "read",
  },
  {
    name: "scene.summary",
    title: "Read a scene's record",
    description:
      "Reads a scene's time, context, names and marks. Not its prose.",
    needs: "read",
  },
  {
    name: "timeline.list",
    title: "Read the timeline",
    description: "Reads events in story time and reading order.",
    needs: "read",
  },
  {
    name: "sources.summarise",
    title: "Summarise a source note",
    description:
      "Summarises research and source notes. Never manuscript scenes.",
    needs: "read",
  },
  {
    name: "facts.propose",
    title: "Propose fact confirmations",
    description:
      "Prepares a card of proposed facts. You settle each one yourself.",
    needs: "propose",
  },
  {
    name: "repairs.propose",
    title: "Propose a repair",
    description:
      "Prepares a card with a mark's repairs. You press the one you want.",
    needs: "propose",
  },
  {
    name: "manuscript.write",
    title: "Write prose",
    description: "Write new manuscript text.",
    needs: "never",
    closedBecause:
      "Asten is your editor, not your writer. No tool writes your prose.",
  },
  {
    name: "manuscript.rewrite",
    title: "Rewrite prose",
    description: "Change the words of a scene.",
    needs: "never",
    closedBecause:
      "Asten never rewrites, rephrases or polishes your sentences.",
  },
  {
    name: "manuscript.continue",
    title: "Continue prose",
    description: "Continue or autocomplete a scene.",
    needs: "never",
    closedBecause: "No generate, continue or autocomplete. Ever.",
  },
  {
    name: "facts.settle",
    title: "Settle a fact",
    description: "Make a proposal canon.",
    needs: "never",
    closedBecause:
      "Only you settle canon; the assistant proposes and you press.",
  },
  {
    name: "settings.change",
    title: "Change settings",
    description: "Change providers, keys or rules.",
    needs: "never",
    closedBecause:
      "The assistant cannot change its own access, providers or keys.",
  },
];

export const TOOL_BY_NAME = new Map(TOOLS.map((tool) => [tool.name, tool]));

const ACCESS_RANK: Record<Access, number> = { read: 0, propose: 1 };

export interface CallDecision {
  allowed: boolean;
  reason?: string;
}

/**
 * Decide one tool call. Unknown tools are refused (the table is closed), a
 * `never` tool is refused at every access, a tool switched off in Settings is
 * refused, and a tool above the thread's access is refused.
 */
export function decideToolCall(
  name: string,
  access: Access,
  disabled: ReadonlySet<string> = new Set(),
): CallDecision {
  const tool = TOOL_BY_NAME.get(name);
  if (!tool)
    return {
      allowed: false,
      reason: `No tool called ${name}. The table is closed.`,
    };
  if (tool.needs === "never")
    return {
      allowed: false,
      reason: tool.closedBecause ?? "Closed by a fixed rule.",
    };
  if (disabled.has(name))
    return {
      allowed: false,
      reason: `${tool.title} is switched off in Settings.`,
    };
  if (ACCESS_RANK[access] < ACCESS_RANK[tool.needs])
    return {
      allowed: false,
      reason: `${tool.title} needs Propose access; this thread is Read only.`,
    };
  return { allowed: true };
}

/** Which documents a tool may read the text of. Manuscript scenes are never summarised. */
export function mayReadTextOf(tool: string, kind: string): boolean {
  if (tool === "sources.summarise")
    return kind === "source" || kind === "note" || kind === "wiki";
  return false;
}

export type Intent =
  | { kind: "prose"; verb: string }
  | { kind: "explain"; code?: string }
  | { kind: "marks" }
  | { kind: "checks" }
  | { kind: "canon"; query: string }
  | { kind: "graph"; query: string }
  | { kind: "facts" }
  | { kind: "source"; query: string }
  | { kind: "timeline" }
  | { kind: "help" };

const PROSE_VERBS =
  /\b(write|rewrite|re-write|continue|finish|draft|generate|compose|expand|rephrase|paraphrase|reword|polish|improve|fix the (?:prose|wording|sentence)|autocomplete|complete|edit|make (?:it|this) (?:better|flow)|punch up|tighten)\b/i;
const PROSE_OBJECTS =
  /\b(scene|chapter|paragraph|sentence|sentences|prose|dialogue|line|lines|opening|ending|page|story|text|words|it|this|that|next part|the rest)\b/i;

/**
 * Read what the person asked. A request to produce or change manuscript
 * prose is recognised first and refused by rule, whatever else it says.
 */
export function classifyRequest(text: string): Intent {
  const t = text.trim();
  const verb = PROSE_VERBS.exec(t);
  if (
    verb &&
    PROSE_OBJECTS.test(t) &&
    !/\b(explain|why|what is|summar)/i.test(t)
  )
    return { kind: "prose", verb: verb[1].toLowerCase() };
  const code = /\b((?:CANON|NAME|PLOT|CRAFT)-[A-Z]\d{3})\b/i
    .exec(t)?.[1]
    ?.toUpperCase();
  if (
    /\b(explain|why|what does|what is)\b/i.test(t) &&
    (code || /\bmark|flag|error|warning\b/i.test(t))
  )
    return { kind: "explain", code };
  if (
    /\b(run|re-?run)\b.*\bchecks?\b|\bcheck (the )?(project|book|manuscript)\b/i.test(
      t,
    )
  )
    return { kind: "checks" };
  if (
    /\b(propose|confirm|settle|pending)\b.*\bfacts?\b|\bproposals?\b/i.test(t)
  )
    return { kind: "facts" };
  if (/\bsummar(y|ise|ize)\b|\bsources?\b|\bresearch\b/i.test(t))
    return {
      kind: "source",
      query: t.replace(/.*\b(of|on|about)\b/i, "").trim(),
    };
  if (/\b(graph|links?|backlinks?|connected|related)\b/i.test(t))
    return {
      kind: "graph",
      query: t.replace(/.*\b(to|for|of|about)\b/i, "").trim(),
    };
  if (/\b(timeline|when|story time|order)\b/i.test(t))
    return { kind: "timeline" };
  if (/\b(marks?|problems?|contradictions?|continuity)\b/i.test(t))
    return { kind: "marks" };
  if (/\b(who|what|canon|fact|eye|age|where|search|find)\b/i.test(t))
    return {
      kind: "canon",
      query: t
        .replace(/^(who|what|where) (is|are|was)\s*/i, "")
        .replace(/[?.!]+$/, "")
        .trim(),
    };
  return { kind: "help" };
}
