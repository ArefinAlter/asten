/**
 * Source Control over Asten's own history: pages changed since their last
 * snapshot, a paragraph diff with word-level changes inside a paragraph, and
 * commits grouped from snapshots that share a message. Bounded (D11): the
 * diff gives up on a minimal edit script past a size limit and reports a
 * whole replacement instead.
 */
import type { DocumentSnapshot, DocumentSummary } from "@asten/contracts";
import { documentText } from "./knowledge/vault";

export type DiffOp<T> = { op: "same" | "add" | "del"; value: T };

const MAX_CELLS = 250_000;

/** LCS diff of two sequences; falls back to delete-all/add-all past the bound. */
export function diffSequences<T>(
  a: T[],
  b: T[],
  equal: (x: T, y: T) => boolean = Object.is,
): DiffOp<T>[] {
  if (a.length * b.length > MAX_CELLS)
    return [
      ...a.map((value) => ({ op: "del" as const, value })),
      ...b.map((value) => ({ op: "add" as const, value })),
    ];
  const rows = a.length + 1;
  const cols = b.length + 1;
  const table = new Uint32Array(rows * cols);
  for (let i = a.length - 1; i >= 0; i -= 1)
    for (let j = b.length - 1; j >= 0; j -= 1)
      table[i * cols + j] = equal(a[i], b[j])
        ? table[(i + 1) * cols + j + 1] + 1
        : Math.max(table[(i + 1) * cols + j], table[i * cols + j + 1]);
  const out: DiffOp<T>[] = [];
  let i = 0;
  let j = 0;
  while (i < a.length && j < b.length) {
    if (equal(a[i], b[j])) {
      out.push({ op: "same", value: a[i] });
      i += 1;
      j += 1;
    } else if (table[(i + 1) * cols + j] >= table[i * cols + j + 1]) {
      out.push({ op: "del", value: a[i] });
      i += 1;
    } else {
      out.push({ op: "add", value: b[j] });
      j += 1;
    }
  }
  while (i < a.length) out.push({ op: "del", value: a[i++] });
  while (j < b.length) out.push({ op: "add", value: b[j++] });
  return out;
}

export function paragraphsOf(content: unknown): string[] {
  return documentText(content)
    .split("\n")
    .map((p) => p.trim())
    .filter(Boolean);
}

export interface ParagraphChange {
  kind: "same" | "add" | "del" | "change";
  before?: string;
  after?: string;
  words?: DiffOp<string>[];
}

/** Paragraph diff; a deletion followed by an addition is shown as one changed paragraph with word marks. */
export function diffParagraphs(
  before: string[],
  after: string[],
): ParagraphChange[] {
  const ops = diffSequences(before, after);
  const out: ParagraphChange[] = [];
  let index = 0;
  while (index < ops.length) {
    const op = ops[index];
    if (op.op === "same") {
      out.push({ kind: "same", before: op.value, after: op.value });
      index += 1;
      continue;
    }
    // A run of deletions and additions: pair them up as changed paragraphs.
    const dels: string[] = [];
    const adds: string[] = [];
    while (index < ops.length && ops[index].op !== "same") {
      if (ops[index].op === "del") dels.push(ops[index].value);
      else adds.push(ops[index].value);
      index += 1;
    }
    const pairs = Math.min(dels.length, adds.length);
    for (let k = 0; k < pairs; k += 1)
      out.push({
        kind: "change",
        before: dels[k],
        after: adds[k],
        words: diffSequences(dels[k].split(/(\s+)/), adds[k].split(/(\s+)/)),
      });
    for (const value of dels.slice(pairs))
      out.push({ kind: "del", before: value });
    for (const value of adds.slice(pairs))
      out.push({ kind: "add", after: value });
  }
  return out;
}

export function diffStats(changes: ParagraphChange[]) {
  let added = 0;
  let removed = 0;
  for (const change of changes) {
    if (change.kind === "add") added += 1;
    else if (change.kind === "del") removed += 1;
    else if (change.kind === "change") {
      added += 1;
      removed += 1;
    }
  }
  return { added, removed };
}

export interface PageStatus {
  document: DocumentSummary;
  latest?: DocumentSnapshot;
  state: "modified" | "untracked" | "clean";
}

export function latestSnapshot(
  snapshots: DocumentSnapshot[],
): DocumentSnapshot | undefined {
  return [...snapshots].sort(
    (a, b) => b.created_at.localeCompare(a.created_at) || b.seq - a.seq,
  )[0];
}

/** Whether each page moved since its last snapshot. */
export function pageStatus(
  document: DocumentSummary,
  snapshots: DocumentSnapshot[],
): PageStatus {
  const latest = latestSnapshot(snapshots);
  if (!latest) return { document, state: "untracked" };
  return {
    document,
    latest,
    state:
      latest.seq === document.head_seq &&
      latest.chain_hash === document.chain_hash
        ? "clean"
        : "modified",
  };
}

export interface Commit {
  id: string;
  message: string;
  at: string;
  snapshots: DocumentSnapshot[];
}

/** Group snapshots into commits: same message within two minutes is one commit. */
export function commitsOf(snapshots: DocumentSnapshot[]): Commit[] {
  const sorted = [...snapshots].sort((a, b) =>
    b.created_at.localeCompare(a.created_at),
  );
  const commits: Commit[] = [];
  for (const snapshot of sorted) {
    const message = snapshot.label ?? "(no message)";
    const last = commits.at(-1);
    if (
      last &&
      last.message === message &&
      Math.abs(Date.parse(last.at) - Date.parse(snapshot.created_at)) < 120_000
    )
      last.snapshots.push(snapshot);
    else
      commits.push({
        id: snapshot.id,
        message,
        at: snapshot.created_at,
        snapshots: [snapshot],
      });
  }
  return commits;
}
