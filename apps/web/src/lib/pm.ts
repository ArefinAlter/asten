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

export function countWords(content: unknown): number {
  const text = collectText(content).trim();
  if (!text) return 0;
  return text.split(/\s+/).length;
}

function collectText(value: unknown): string {
  if (!value) return "";
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.map(collectText).join(" ");
  if (typeof value === "object") {
    const node = value as { text?: string; content?: unknown };
    return `${node.text ?? ""} ${collectText(node.content)}`;
  }
  return "";
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
