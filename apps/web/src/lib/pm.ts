import type { JSONContent } from "@tiptap/core";
import type { Node as PmNode } from "@tiptap/pm/model";

/**
 * A v4 UUID. `crypto.randomUUID` exists only in secure contexts; a desk
 * opened over plain HTTP from another machine still needs block ids.
 */
export function uuid(): string {
  const native = (crypto as { randomUUID?: () => string }).randomUUID;
  if (typeof native === "function") return native.call(crypto);
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = Array.from(bytes, (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

export function emptyDoc(): JSONContent {
  return {
    type: "doc",
    content: [
      {
        type: "paragraph",
        attrs: { id: uuid() },
        content: [],
      },
    ],
  };
}

export function toEditorDoc(content: unknown): JSONContent {
  if (
    content &&
    typeof content === "object" &&
    "type" in content &&
    (content as { type: string }).type === "doc"
  ) {
    return content as JSONContent;
  }
  if (
    content &&
    typeof content === "object" &&
    "text" in content &&
    typeof (content as { text: unknown }).text === "string"
  ) {
    const text = (content as { text: string }).text;
    return {
      type: "doc",
      content: [
        {
          type: "paragraph",
          attrs: { id: uuid() },
          content: text ? [{ type: "text", text }] : [],
        },
      ],
    };
  }
  return emptyDoc();
}

/**
 * Unicode `White_Space`, the set Rust's `str::split_whitespace` splits on.
 * JavaScript's `\s` differs at the edges (it adds U+FEFF and lacks U+0085),
 * so the class is written out to keep both counts identical.
 */
const WHITESPACE =
  /[\t\n\v\f\r \u0085\u00a0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000]+/;

/**
 * The desk's one word-count rule, shared with the server's `words` field:
 * the plain text of the document split on Unicode whitespace, empty runs
 * dropped. `asten_blocks::word_count` is the Rust twin.
 */
export function countWords(content: unknown): number {
  return plainText(content).split(WHITESPACE).filter(Boolean).length;
}

/**
 * Plain text the way the engine reads a document (`asten_blocks::plain_text`):
 * text nodes concatenated, a hard break is a newline, a formula is its source,
 * and the children of a `doc` are joined with newlines, everything else with
 * nothing.
 */
export function plainText(value: unknown): string {
  if (!value || typeof value !== "object") return "";
  const node = value as {
    type?: string;
    text?: string;
    attrs?: { latex?: unknown };
    content?: unknown;
  };
  if (typeof node.text === "string") return node.text;
  if (node.type === "hardBreak") return "\n";
  if (typeof node.attrs?.latex === "string") return node.attrs.latex;
  if (!Array.isArray(node.content)) return "";
  return node.content.map(plainText).join(node.type === "doc" ? "\n" : "");
}

/**
 * Plain text of a block node the way the engine counts it: text nodes joined,
 * a hard break is a newline. Offsets into this string are Unicode scalars.
 */
export function blockPlainText(node: PmNode): string {
  let out = "";
  node.descendants((child) => {
    if (child.isText) out += child.text ?? "";
    else if (child.type.name === "hardBreak") out += "\n";
    return true;
  });
  return out;
}

/**
 * Map a scalar offset inside a block to a ProseMirror document position.
 * `blockPos` is the position of the block node itself.
 */
export function scalarToPos(
  block: PmNode,
  blockPos: number,
  scalar: number,
): number {
  let remaining = scalar;
  let result = blockPos + 1; // inside the block
  let found = false;
  block.descendants((child, pos) => {
    if (found) return false;
    if (child.isText) {
      const text = child.text ?? "";
      const scalars = Array.from(text);
      if (remaining <= scalars.length) {
        // Convert scalar count to UTF-16 length for the PM position.
        const prefix = scalars.slice(0, remaining).join("");
        result = blockPos + 1 + pos + prefix.length;
        found = true;
        return false;
      }
      remaining -= scalars.length;
    } else if (child.type.name === "hardBreak") {
      if (remaining === 0) {
        result = blockPos + 1 + pos;
        found = true;
        return false;
      }
      remaining -= 1;
    }
    return true;
  });
  if (!found) result = blockPos + block.nodeSize - 1;
  return result;
}

/** Find the top-level block with a stable id. Returns its node and position. */
export function findBlock(
  doc: PmNode,
  id: string,
): { node: PmNode; pos: number } | undefined {
  let hit: { node: PmNode; pos: number } | undefined;
  doc.forEach((child, offset) => {
    if (!hit && child.attrs?.id === id) hit = { node: child, pos: offset };
  });
  return hit;
}

/** Scalar offset within a block for a ProseMirror position inside it. */
export function posToScalar(
  block: PmNode,
  blockPos: number,
  pos: number,
): number {
  const inner = pos - blockPos - 1;
  let scalars = 0;
  let done = false;
  block.descendants((child, childPos) => {
    if (done) return false;
    if (child.isText) {
      const text = child.text ?? "";
      if (inner <= childPos) {
        done = true;
        return false;
      }
      const take = Math.min(text.length, inner - childPos);
      scalars += Array.from(text.slice(0, take)).length;
      if (take < text.length) {
        done = true;
        return false;
      }
    } else if (child.type.name === "hardBreak") {
      if (inner <= childPos) {
        done = true;
        return false;
      }
      scalars += 1;
    }
    return true;
  });
  return scalars;
}
