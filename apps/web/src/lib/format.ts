import type { Context, EventKind, Severity, Sort } from "@asten/contracts";

export function severityWord(severity: Severity): string {
  switch (severity) {
    case "error":
      return "Error";
    case "warning":
      return "Warning";
    case "information":
      return "Note";
    case "hint":
      return "Hint";
  }
}

export function contextLabel(context: Context | undefined): string {
  if (!context) return "narration";
  switch (context.kind) {
    case "world":
      return "world";
    case "narration":
      return "narration";
    case "says":
      return context.speaker ? `said by ${context.speaker}` : "dialogue";
    case "believes":
      return `believed by ${context.who}`;
    case "memory":
      return "memory";
    case "dream":
      return "dream";
    case "vision":
      return "vision";
    case "frame":
      return `frame story ${context.key}`;
    case "hypothetical":
      return "hypothetical";
  }
}

export const CONTEXT_CHOICES: {
  value: string;
  label: string;
  context: Context;
}[] = [
  { value: "narration", label: "Narration", context: { kind: "narration" } },
  { value: "memory", label: "Memory / flashback", context: { kind: "memory" } },
  { value: "dream", label: "Dream", context: { kind: "dream", key: "dream" } },
  {
    value: "vision",
    label: "Vision",
    context: { kind: "vision", key: "vision" },
  },
  {
    value: "frame",
    label: "Story within the story",
    context: { kind: "frame", key: "frame" },
  },
  {
    value: "hypothetical",
    label: "Hypothetical",
    context: { kind: "hypothetical" },
  },
];

export function contextValue(context: Context | undefined): string {
  if (!context) return "narration";
  return context.kind === "says" ||
    context.kind === "believes" ||
    context.kind === "world"
    ? "narration"
    : context.kind;
}

export function eventKindLabel(kind: EventKind): string {
  switch (kind.kind) {
    case "born":
      return "is born";
    case "dies":
      return "dies";
    case "resurrected":
      return "returns from death";
    case "destroyed":
      return "is destroyed";
    case "repaired":
      return "is repaired";
    case "travels":
      return kind.to ? `travels to ${humanKey(kind.to)}` : "travels";
    case "arrives":
      return `arrives at ${humanKey(kind.place)}`;
    case "acquires":
      return `acquires ${humanKey(kind.item)}`;
    case "loses":
      return `loses ${humanKey(kind.item)}`;
    case "learns":
      return `learns ${kind.secret}`;
    case "tells":
      return `tells ${humanKey(kind.to)} about ${kind.secret}`;
    case "reveals":
      return `reveals ${kind.secret}`;
    case "uses_knowledge":
      return `uses knowledge of ${kind.secret}`;
    case "marries":
      return `marries ${humanKey(kind.to)}`;
    case "divorces":
      return `divorces ${humanKey(kind.to)}`;
    case "changes":
      return `changes ${kind.fluent.predicate.replace(/_/g, " ")}`;
    case "acts":
      return "acts";
    case "other":
      return kind.label;
  }
}

export const EVENT_KIND_CHOICES: {
  value: string;
  label: string;
  needs?: "place" | "item" | "entity" | "secret" | "label";
}[] = [
  { value: "acts", label: "Acts (appears, speaks, moves)" },
  { value: "dies", label: "Dies" },
  { value: "born", label: "Is born" },
  { value: "resurrected", label: "Returns from death" },
  { value: "destroyed", label: "Is destroyed" },
  { value: "repaired", label: "Is repaired" },
  { value: "arrives", label: "Arrives at a place", needs: "place" },
  { value: "acquires", label: "Acquires an item", needs: "item" },
  { value: "loses", label: "Loses an item", needs: "item" },
  { value: "learns", label: "Learns a secret", needs: "secret" },
  { value: "reveals", label: "Reveals a secret to all", needs: "secret" },
  {
    value: "uses_knowledge",
    label: "Uses knowledge of a secret",
    needs: "secret",
  },
  { value: "marries", label: "Marries", needs: "entity" },
  { value: "divorces", label: "Divorces", needs: "entity" },
  { value: "other", label: "Something else", needs: "label" },
];

export function buildEventKind(value: string, argument: string): EventKind {
  const key = entityKey(argument);
  switch (value) {
    case "dies":
    case "born":
    case "resurrected":
    case "destroyed":
    case "repaired":
    case "acts":
      return { kind: value } as EventKind;
    case "arrives":
      return { kind: "arrives", place: key };
    case "acquires":
      return { kind: "acquires", item: key };
    case "loses":
      return { kind: "loses", item: key };
    case "learns":
      return { kind: "learns", secret: argument };
    case "reveals":
      return { kind: "reveals", secret: argument };
    case "uses_knowledge":
      return { kind: "uses_knowledge", secret: argument };
    case "marries":
      return { kind: "marries", to: key };
    case "divorces":
      return { kind: "divorces", to: key };
    default:
      return { kind: "other", label: argument || "something" };
  }
}

/** Mirror of the Rust entity key: folded, spaces to underscores. */
export function entityKey(name: string): string {
  return name.trim().toLowerCase().split(/\s+/).join("_");
}

export function humanKey(key: string): string {
  return key.replace(/_/g, " ");
}

export function sortLabel(sort: Sort | string): string {
  return sort.replace(/_/g, " ");
}

export function shortHash(hash: string): string {
  return hash.slice(0, 10);
}

export function timeAgo(iso: string): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return iso;
  const seconds = Math.round((Date.now() - then) / 1000);
  if (seconds < 60) return "just now";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} min ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 48) return `${hours} h ago`;
  return new Date(iso).toLocaleDateString();
}

export function percent(value: number): string {
  return `${Math.round(value * 100)}%`;
}
