import type { JSONContent } from "@tiptap/core";

export function emptyDoc(): JSONContent {
  return {
    type: "doc",
    content: [
      {
        type: "paragraph",
        attrs: { id: crypto.randomUUID() },
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
          attrs: { id: crypto.randomUUID() },
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
