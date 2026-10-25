/**
 * The console in the bottom panel: Asten's own commands, run against the
 * engine the desk is connected to (the local API, or the in-page sample
 * backend). It is not a system shell and has no path to one; anything that
 * is not an `asten` command is refused with that sentence.
 */
import type {
  AstenDiagnostic,
  CanonEntity,
  CompileArtifact,
  CompileFormat,
  DocumentSummary,
  Fact,
  MethodPack,
} from "@asten/contracts";

export interface ConsoleEngine {
  projectName(): string | undefined;
  diagnostics(): Promise<{ revision: string; diagnostics: AstenDiagnostic[] }>;
  documents(): Promise<DocumentSummary[]>;
  compile(format: CompileFormat): Promise<CompileArtifact>;
  packs(): Promise<MethodPack[]>;
  setPack(id: string, enabled: boolean): Promise<MethodPack>;
  facts(): Promise<Fact[]>;
  entities(): Promise<CanonEntity[]>;
  snapshot(documentId: string, label: string): Promise<{ id: string }>;
  openDocumentId(): string | undefined;
}

export interface ConsoleLine {
  kind: "input" | "out" | "err" | "ok" | "dim";
  text: string;
}

export interface ConsoleResult {
  lines: ConsoleLine[];
  clear?: boolean;
  artifact?: CompileArtifact;
}

export const FORMATS: CompileFormat[] = [
  "markdown",
  "plain",
  "fountain",
  "html",
  "docx",
  "epub",
  "fdx",
];

/** Split a command line into words, honouring single and double quotes. */
export function tokenize(line: string): string[] {
  const out: string[] = [];
  let current = "";
  let quote: string | null = null;
  let had = false;
  for (const char of line.trim()) {
    if (quote) {
      if (char === quote) quote = null;
      else current += char;
    } else if (char === '"' || char === "'") {
      quote = char;
      had = true;
    } else if (/\s/.test(char)) {
      if (current || had) out.push(current);
      current = "";
      had = false;
    } else current += char;
  }
  if (current || had) out.push(current);
  return out;
}

export interface Parsed {
  command: string;
  args: string[];
  flags: Record<string, string | true>;
}

export function parseCommand(line: string): Parsed | { error: string } | null {
  const words = tokenize(line);
  if (!words.length) return null;
  if (words[0] === "clear" || words[0] === "cls")
    return { command: "clear", args: [], flags: {} };
  if (words[0] !== "asten")
    return {
      error: `${words[0]}: not an Asten command. This console runs Asten's own commands against the in-app engine; it is not a system shell. Try \`asten help\`.`,
    };
  const [, command = "help", ...rest] = words;
  const args: string[] = [];
  const flags: Record<string, string | true> = {};
  for (let index = 0; index < rest.length; index += 1) {
    const word = rest[index];
    if (word.startsWith("--")) {
      const [key, inline] = word.slice(2).split("=", 2);
      if (inline !== undefined) flags[key] = inline;
      else if (rest[index + 1] && !rest[index + 1].startsWith("--")) {
        flags[key] = rest[index + 1];
        index += 1;
      } else flags[key] = true;
    } else args.push(word);
  }
  return { command, args, flags };
}

export const HELP: [string, string][] = [
  ["asten check", "weigh the project and list its open marks"],
  [
    "asten marks [--severity error|warning|information]",
    "list marks without re-running",
  ],
  [
    "asten compile --format <format>",
    `compile in binder order (${FORMATS.join(", ")})`,
  ],
  ["asten pack list", "list method packs"],
  ["asten pack enable|disable <key>", "switch a pack (Scribe stays on)"],
  ["asten facts [name]", "settled and proposed facts"],
  ["asten cast", "the names Asten knows"],
  ["asten status", "project, pages, words, marks, revision"],
  ['asten snapshot "message"', "commit the open page as a version"],
  ["clear", "clear the console"],
];

const sev = (d: AstenDiagnostic) =>
  d.severity === "error"
    ? "error  "
    : d.severity === "warning"
      ? "warning"
      : "info   ";

function listMarks(
  marks: AstenDiagnostic[],
  titles: Map<string, string>,
): ConsoleLine[] {
  if (!marks.length)
    return [{ kind: "ok", text: "No open marks. The scales balance." }];
  return marks.map((mark) => ({
    kind: mark.severity === "error" ? "err" : "out",
    text: `${sev(mark)} ${mark.code}  ${mark.title}  — ${titles.get(mark.primary?.document ?? "") ?? "project"}`,
  }));
}

export async function runCommand(
  line: string,
  engine: ConsoleEngine,
): Promise<ConsoleResult> {
  const input: ConsoleLine = { kind: "input", text: line };
  const parsed = parseCommand(line);
  if (!parsed) return { lines: [] };
  if ("error" in parsed)
    return { lines: [input, { kind: "err", text: parsed.error }] };
  if (parsed.command === "clear") return { lines: [], clear: true };
  const out = (
    lines: ConsoleLine[],
    extra: Partial<ConsoleResult> = {},
  ): ConsoleResult => ({
    lines: [input, ...lines],
    ...extra,
  });
  const needsProject = () =>
    engine.projectName()
      ? null
      : out([{ kind: "err", text: "Open a project first." }]);
  try {
    switch (parsed.command) {
      case "help":
        return out(
          HELP.map(([command, text]) => ({
            kind: "out",
            text: `${command.padEnd(52)}${text}`,
          })),
        );
      case "check":
      case "marks": {
        const blocked = needsProject();
        if (blocked) return blocked;
        const started = performance.now();
        const [snapshot, documents] = await Promise.all([
          engine.diagnostics(),
          engine.documents(),
        ]);
        const titles = new Map(documents.map((d) => [d.id, d.title]));
        const severity = parsed.flags.severity;
        const open = snapshot.diagnostics.filter(
          (d) =>
            !d.suppressed_by &&
            (typeof severity !== "string" || d.severity === severity),
        );
        const errors = open.filter((d) => d.severity === "error").length;
        const warnings = open.filter((d) => d.severity === "warning").length;
        return out([
          ...(parsed.command === "check"
            ? [
                {
                  kind: "dim" as const,
                  text: `Weighing ${engine.projectName()} at revision ${snapshot.revision.slice(0, 8)} …`,
                },
              ]
            : []),
          ...listMarks(open, titles),
          {
            kind: errors ? "err" : "ok",
            text: `${errors} error${errors === 1 ? "" : "s"}, ${warnings} warning${warnings === 1 ? "" : "s"}, ${open.length - errors - warnings} information · ${Math.max(1, Math.round(performance.now() - started))} ms`,
          },
        ]);
      }
      case "compile": {
        const blocked = needsProject();
        if (blocked) return blocked;
        const format = (parsed.flags.format ??
          parsed.args[0] ??
          "markdown") as string;
        if (!FORMATS.includes(format as CompileFormat))
          return out([
            {
              kind: "err",
              text: `Unknown format "${format}". One of: ${FORMATS.join(", ")}.`,
            },
          ]);
        const artifact = await engine.compile(format as CompileFormat);
        const bytes =
          artifact.encoding === "base64"
            ? Math.floor((artifact.body.length * 3) / 4)
            : new TextEncoder().encode(artifact.body).length;
        return out(
          [
            { kind: "dim", text: `Compiling in binder order to ${format} …` },
            {
              kind: "ok",
              text: `Wrote ${artifact.filename} (${bytes.toLocaleString("en")} bytes, ${artifact.mime}).`,
            },
          ],
          { artifact },
        );
      }
      case "pack": {
        const [sub = "list", key] = parsed.args;
        const packs = await engine.packs();
        if (sub === "list")
          return out(
            packs.map((pack) => ({
              kind: "out" as const,
              text: `${pack.enabled ? "on " : "off"}  ${pack.pack_key.padEnd(12)} ${pack.version.padEnd(7)} ${pack.summary}`,
            })),
          );
        if (sub !== "enable" && sub !== "disable")
          return out([
            {
              kind: "err",
              text: `asten pack ${sub}: use list, enable or disable.`,
            },
          ]);
        const pack = packs.find((p) => p.pack_key === key || p.id === key);
        if (!pack)
          return out([{ kind: "err", text: `No pack "${key ?? ""}".` }]);
        if (pack.pack_key === "scribe" && sub === "disable")
          return out([
            {
              kind: "err",
              text: "Scribe stays on: attribute contradiction is the core check.",
            },
          ]);
        await engine.setPack(pack.id, sub === "enable");
        return out([
          {
            kind: "ok",
            text: `${pack.name} ${sub === "enable" ? "enabled" : "disabled"}.`,
          },
        ]);
      }
      case "facts": {
        const blocked = needsProject();
        if (blocked) return blocked;
        const [facts, entities] = await Promise.all([
          engine.facts(),
          engine.entities(),
        ]);
        const names = new Map(entities.map((e) => [e.id, e.name]));
        const query = parsed.args.join(" ").toLowerCase();
        const rows = facts.filter(
          (f) =>
            !query ||
            (names.get(f.entity_key) ?? f.entity_key)
              .toLowerCase()
              .includes(query),
        );
        if (!rows.length)
          return out([{ kind: "dim", text: "No facts match." }]);
        return out(
          rows.map((f) => ({
            kind: "out" as const,
            text: `${f.status.padEnd(10)} ${(names.get(f.entity_key) ?? f.entity_key).padEnd(18)} ${f.predicate} = ${f.value}`,
          })),
        );
      }
      case "cast": {
        const blocked = needsProject();
        if (blocked) return blocked;
        const entities = await engine.entities();
        return out(
          entities.map((e) => ({
            kind: "out" as const,
            text: `${e.kind.padEnd(10)} ${e.name}${e.aliases.length ? `  (${e.aliases.join(", ")})` : ""}`,
          })),
        );
      }
      case "status": {
        const blocked = needsProject();
        if (blocked) return blocked;
        const [documents, snapshot] = await Promise.all([
          engine.documents(),
          engine.diagnostics(),
        ]);
        const words = documents.reduce((sum, d) => sum + d.words, 0);
        const scenes = documents.filter((d) => d.kind === "scene").length;
        const open = snapshot.diagnostics.filter(
          (d) => !d.suppressed_by,
        ).length;
        return out([
          { kind: "out", text: `project   ${engine.projectName()}` },
          {
            kind: "out",
            text: `pages     ${documents.length} (${scenes} scenes)`,
          },
          { kind: "out", text: `words     ${words.toLocaleString("en")}` },
          { kind: "out", text: `marks     ${open} open` },
          { kind: "out", text: `revision  ${snapshot.revision.slice(0, 12)}` },
        ]);
      }
      case "snapshot": {
        const id = engine.openDocumentId();
        if (!id)
          return out([{ kind: "err", text: "Open a page to snapshot it." }]);
        const label =
          parsed.args.join(" ") ||
          (typeof parsed.flags.m === "string" ? parsed.flags.m : "");
        if (!label)
          return out([
            {
              kind: "err",
              text: 'Give the version a message: asten snapshot "message"',
            },
          ]);
        const snapshot = await engine.snapshot(id, label);
        return out([
          { kind: "ok", text: `Committed ${snapshot.id}: ${label}` },
        ]);
      }
      default:
        return out([
          {
            kind: "err",
            text: `asten ${parsed.command}: unknown command. Try \`asten help\`.`,
          },
        ]);
    }
  } catch (error) {
    return out([
      {
        kind: "err",
        text: error instanceof Error ? error.message : String(error),
      },
    ]);
  }
}
