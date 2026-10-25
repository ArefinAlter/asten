/**
 * Ribbon command mapping. Every ribbon control names a command id; this file
 * says what each id does, as data: a list of Tiptap chain steps for editor
 * commands, an app action for the rest, or a stated reason when the command
 * is not in this release. Pure, so the mapping is tested without an editor.
 */

/** One chained editor call: the method name and its arguments. */
export type EditorStep = readonly [method: string, ...args: unknown[]];

export const EDITOR_COMMANDS: Record<string, readonly EditorStep[]> = {
  "editor.bold": [["toggleBold"]],
  "editor.italic": [["toggleItalic"]],
  "editor.underline": [["toggleUnderline"]],
  "editor.strike": [["toggleStrike"]],
  "editor.code": [["toggleCode"]],
  "editor.clearFormatting": [["unsetAllMarks"], ["clearNodes"]],
  "editor.bulletList": [["toggleBulletList"]],
  "editor.orderedList": [["toggleOrderedList"]],
  "editor.indent": [["sinkListItem", "listItem"]],
  "editor.outdent": [["liftListItem", "listItem"]],
  "editor.blockquote": [["toggleBlockquote"]],
  "editor.alignLeft": [["setTextAlign", "left"]],
  "editor.alignCenter": [["setTextAlign", "center"]],
  "editor.alignRight": [["setTextAlign", "right"]],
  "editor.alignJustify": [["setTextAlign", "justify"]],
  "editor.styleNormal": [["setParagraph"]],
  "editor.styleHeading1": [["toggleHeading", { level: 1 }]],
  "editor.styleHeading2": [["toggleHeading", { level: 2 }]],
  "editor.styleHeading3": [["toggleHeading", { level: 3 }]],
  "editor.styleQuote": [["setParagraph"], ["toggleBlockquote"]],
  "editor.sceneBreak": [["insertContent", { type: "sceneBreak" }]],
  "editor.horizontalRule": [["setHorizontalRule"]],
  "editor.hardBreak": [["setHardBreak"]],
  "editor.undo": [["undo"]],
  "editor.redo": [["redo"]],
  "editor.selectAll": [["selectAll"]],
  "editor.trackInsert": [["setMark", "insertion"]],
  "editor.trackDelete": [["setMark", "deletion"]],
  "editor.acceptMarked": [
    ["unsetMark", "insertion"],
    ["unsetMark", "deletion"],
  ],
  "editor.unlink": [["unsetLink"]],
};

/** Which `isActive` check lights a toggle in the ribbon: a node or mark name, attributes, or both. */
export interface ActiveCheck {
  name?: string;
  attrs?: Record<string, unknown>;
}

export const ACTIVE_STATE: Record<string, ActiveCheck> = {
  "editor.bold": { name: "bold" },
  "editor.italic": { name: "italic" },
  "editor.underline": { name: "underline" },
  "editor.strike": { name: "strike" },
  "editor.code": { name: "code" },
  "editor.bulletList": { name: "bulletList" },
  "editor.orderedList": { name: "orderedList" },
  "editor.blockquote": { name: "blockquote" },
  "editor.styleNormal": { name: "paragraph" },
  "editor.styleHeading1": { name: "heading", attrs: { level: 1 } },
  "editor.styleHeading2": { name: "heading", attrs: { level: 2 } },
  "editor.styleHeading3": { name: "heading", attrs: { level: 3 } },
  "editor.alignLeft": { attrs: { textAlign: "left" } },
  "editor.alignCenter": { attrs: { textAlign: "center" } },
  "editor.alignRight": { attrs: { textAlign: "right" } },
  "editor.alignJustify": { attrs: { textAlign: "justify" } },
};

/**
 * Commands that exist on the ribbon but not in this release, with the reason
 * shown on the disabled control. Honest beats hidden.
 */
export const UNAVAILABLE: Record<string, string> = {
  "editor.image":
    "Images are not part of the block model yet; the compile writers would drop them.",
  "editor.table":
    "Tables are not part of the block model yet; the compile writers would drop them.",
  "editor.footnote": "Footnotes are planned. Use a comment or a note for now.",
  "editor.formatPainter": "Format painter is planned.",
  "editor.subscript": "Subscript is not part of the manuscript schema.",
  "editor.superscript": "Superscript is not part of the manuscript schema.",
  "app.share":
    "Asten has no cloud and no accounts (D8). Share by compiling a file or by pairing a guest.",
};

export type ResolvedCommand =
  | { kind: "editor"; steps: readonly EditorStep[] }
  | { kind: "app"; action: string }
  | { kind: "unavailable"; reason: string };

export function resolveCommand(id: string): ResolvedCommand {
  if (UNAVAILABLE[id]) return { kind: "unavailable", reason: UNAVAILABLE[id] };
  const steps = EDITOR_COMMANDS[id];
  if (steps) return { kind: "editor", steps };
  if (
    id.startsWith("app.") ||
    id.startsWith("view.") ||
    id.startsWith("panel.")
  )
    return { kind: "app", action: id };
  return { kind: "unavailable", reason: "Not in this release." };
}

/** The slice of a Tiptap chain the mapping needs. */
export interface ChainLike {
  run(): boolean;
  [method: string]: unknown;
}

export interface EditorLike {
  chain(): ChainLike;
}

/**
 * Run an editor command on an editor (or anything chain-shaped). Returns
 * false when the editor lacks a step (an extension not loaded) or the chain
 * refused, so the ribbon can say so instead of failing silently.
 */
export function runEditorCommand(
  editor: EditorLike,
  id: string,
  extra?: EditorStep[],
): boolean {
  const steps = [...(EDITOR_COMMANDS[id] ?? []), ...(extra ?? [])];
  if (!steps.length) return false;
  let chain = editor.chain();
  const focus = chain.focus as ((...args: unknown[]) => ChainLike) | undefined;
  if (typeof focus === "function") chain = focus.call(chain);
  for (const [method, ...args] of steps) {
    const fn = chain[method] as ((...args: unknown[]) => ChainLike) | undefined;
    if (typeof fn !== "function") return false;
    chain = fn.apply(chain, args);
  }
  return chain.run();
}
