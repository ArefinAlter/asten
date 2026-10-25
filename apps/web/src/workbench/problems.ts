/**
 * The Problems list: continuity marks as rows (code, message, chapter and
 * scene, place in the text), sorted the way VS Code sorts problems: by
 * severity, then by where the page sits in the binder, then by offset.
 * Pure, so the order is tested (`problems.test.ts`).
 */
import type {
  AstenDiagnostic,
  DocumentSummary,
  Severity,
} from "@asten/contracts";

export interface ProblemRow {
  id: string;
  code: string;
  severity: Severity;
  title: string;
  message: string;
  documentId: string | null;
  scene: string;
  chapter: string;
  readingIndex: number;
  start: number;
  line: number;
  suppressed: boolean;
  diagnostic: AstenDiagnostic;
}

export const SEVERITY_RANK: Record<Severity, number> = {
  error: 0,
  warning: 1,
  information: 2,
  hint: 3,
};

/** Depth-first binder order: the reading order of every document. */
export function readingOrder(
  documents: Pick<DocumentSummary, "id" | "parent_id" | "position">[],
): string[] {
  const children = new Map<string | null, typeof documents>();
  for (const document of documents) {
    const key = document.parent_id ?? null;
    const list = children.get(key) ?? [];
    list.push(document);
    children.set(key, list);
  }
  const known = new Set(documents.map((d) => d.id));
  const out: string[] = [];
  const walk = (parent: string | null) => {
    const list = [...(children.get(parent) ?? [])].sort(
      (a, b) => a.position - b.position,
    );
    for (const document of list) {
      out.push(document.id);
      walk(document.id);
    }
  };
  walk(null);
  // Orphans (a parent that no longer exists) go last, in position order.
  for (const document of [...documents].sort(
    (a, b) => a.position - b.position,
  )) {
    if (!out.includes(document.id) && !known.has(document.parent_id ?? ""))
      out.push(document.id);
  }
  return out;
}

export interface ProblemOptions {
  showSuppressed?: boolean;
  filter?: string;
  severities?: Severity[];
}

export function problemRows(
  diagnostics: AstenDiagnostic[],
  documents: DocumentSummary[],
  options: ProblemOptions = {},
): ProblemRow[] {
  const order = readingOrder(documents);
  const index = new Map(order.map((id, position) => [id, position]));
  const byId = new Map(documents.map((d) => [d.id, d]));
  const filter = options.filter?.trim().toLowerCase() ?? "";
  const rows: ProblemRow[] = [];
  for (const diagnostic of diagnostics) {
    const suppressed = Boolean(diagnostic.suppressed_by);
    if (suppressed && !options.showSuppressed) continue;
    if (options.severities && !options.severities.includes(diagnostic.severity))
      continue;
    const documentId = diagnostic.primary?.document ?? null;
    const document = documentId ? byId.get(documentId) : undefined;
    const parent = document?.parent_id
      ? byId.get(document.parent_id)
      : undefined;
    const row: ProblemRow = {
      id: diagnostic.id,
      code: diagnostic.code,
      severity: diagnostic.severity,
      title: diagnostic.title,
      message: diagnostic.message,
      documentId,
      scene: document?.title ?? diagnostic.primary?.label ?? "Project",
      chapter: parent?.title ?? "",
      readingIndex: documentId
        ? (index.get(documentId) ?? Number.MAX_SAFE_INTEGER)
        : -1,
      start: diagnostic.primary?.start ?? 0,
      line: blockLine(diagnostic.primary?.block),
      suppressed,
      diagnostic,
    };
    if (
      filter &&
      ![row.code, row.title, row.message, row.scene, row.chapter].some((text) =>
        text.toLowerCase().includes(filter),
      )
    )
      continue;
    rows.push(row);
  }
  return rows.sort(compareProblems);
}

export function compareProblems(a: ProblemRow, b: ProblemRow): number {
  return (
    SEVERITY_RANK[a.severity] - SEVERITY_RANK[b.severity] ||
    a.readingIndex - b.readingIndex ||
    a.line - b.line ||
    a.start - b.start ||
    a.code.localeCompare(b.code) ||
    a.id.localeCompare(b.id)
  );
}

/** A paragraph number for display, when the block id carries one (`…-p3`). */
function blockLine(block: string | null | undefined): number {
  const match = block ? /-p(\d+)$/.exec(block) : null;
  return match ? Number(match[1]) : 0;
}

export function countBySeverity(rows: Pick<ProblemRow, "severity">[]) {
  const counts = { error: 0, warning: 0, information: 0, hint: 0 };
  for (const row of rows) counts[row.severity] += 1;
  return counts;
}

/** Next or previous mark from the current one, wrapping; for F8 and Shift+F8. */
export function stepMark(
  rows: ProblemRow[],
  currentId: string | undefined,
  direction: 1 | -1,
): ProblemRow | undefined {
  if (!rows.length) return undefined;
  const at = rows.findIndex((row) => row.id === currentId);
  if (at < 0) return direction === 1 ? rows[0] : rows[rows.length - 1];
  return rows[(at + direction + rows.length) % rows.length];
}
