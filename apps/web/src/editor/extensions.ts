import { Extension, Mark, Node } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";
import type { Node as PmNode } from "@tiptap/pm/model";
import type { AstenDiagnostic, CanonEntity } from "@asten/contracts";
import { findBlock, scalarToPos, uuid } from "../lib/pm";

/**
 * Stable block identity, the Dynodoc idea in the editor. Every top-level block
 * carries `attrs.id`; new blocks get one, and when a paragraph is split the
 * second half gets a fresh id so the first keeps its history and anchors.
 */
const BLOCK_TYPES = [
  "paragraph",
  "heading",
  "blockquote",
  "bulletList",
  "orderedList",
  "listItem",
  "codeBlock",
  "horizontalRule",
  "sceneBreak",
];

export const BlockId = Extension.create({
  name: "blockId",
  addGlobalAttributes() {
    return [
      {
        types: BLOCK_TYPES,
        attributes: {
          id: {
            default: null,
            parseHTML: (element) => element.getAttribute("data-block-id"),
            renderHTML: (attributes) =>
              attributes.id ? { "data-block-id": attributes.id } : {},
            keepOnSplit: true,
          },
        },
      },
    ];
  },
  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey("blockId"),
        appendTransaction(transactions, old, state) {
          if (!transactions.some((tr) => tr.docChanged)) return null;
          // What each block said before this change, so that when a split
          // leaves two blocks with one id, the half that kept the text keeps
          // the id (and its anchors, marks and history).
          const before = new Map<string, string>();
          old.doc.forEach((node) => {
            const id = node.attrs.id as string | null;
            if (id && !before.has(id)) before.set(id, node.textContent);
          });
          const groups = new Map<string, { offset: number; text: string }[]>();
          const missing: number[] = [];
          state.doc.forEach((node, offset) => {
            if (!BLOCK_TYPES.includes(node.type.name)) return;
            const id = node.attrs.id as string | null;
            if (!id) {
              missing.push(offset);
              return;
            }
            const group = groups.get(id) ?? [];
            group.push({ offset, text: node.textContent });
            groups.set(id, group);
          });
          const remint: number[] = [...missing];
          for (const [id, group] of groups) {
            if (group.length < 2) continue;
            const previous = before.get(id);
            const keeper =
              group.find((item) => item.text === previous) ??
              group.find(
                (item) =>
                  previous !== undefined &&
                  item.text.length > 0 &&
                  (previous.startsWith(item.text) ||
                    item.text.startsWith(previous)),
              ) ??
              group.reduce((best, item) =>
                item.text.length > best.text.length ? item : best,
              );
            for (const item of group) {
              if (item !== keeper) remint.push(item.offset);
            }
          }
          if (remint.length === 0) return null;
          let tr = state.tr;
          for (const offset of remint) {
            const node = state.doc.nodeAt(offset);
            if (!node) continue;
            tr = tr.setNodeMarkup(offset, undefined, {
              ...node.attrs,
              id: uuid(),
            });
          }
          return tr;
        },
      }),
    ];
  },
});

export const Insertion = Mark.create({
  name: "insertion",
  parseHTML() {
    return [{ tag: "ins" }];
  },
  renderHTML() {
    return ["ins", { class: "tracked-ins" }, 0];
  },
});

export const Deletion = Mark.create({
  name: "deletion",
  parseHTML() {
    return [{ tag: "del" }];
  },
  renderHTML() {
    return ["del", { class: "tracked-del" }, 0];
  },
});

export const SceneBreak = Node.create({
  name: "sceneBreak",
  group: "block",
  parseHTML() {
    return [{ tag: "hr[data-scene]" }];
  },
  renderHTML() {
    return ["hr", { "data-scene": "true", class: "scene-break" }];
  },
});

// ───────────────────────── red marks in the margin ─────────────────────────

export interface MarkSpan {
  id: string;
  severity: string;
  block: string;
  start: number;
  end: number;
  title: string;
  role: "primary" | "related";
}

export const marksKey = new PluginKey<DecorationSet>("astenMarks");

function buildMarkDecorations(doc: PmNode, spans: MarkSpan[]): DecorationSet {
  const decorations: Decoration[] = [];
  for (const span of spans) {
    const block = findBlock(doc, span.block);
    if (!block) continue;
    const textLength = Array.from(block.node.textContent).length;
    if (span.end <= span.start || span.start > textLength) {
      // Whole-block mark: a thin red line down the block's left edge.
      decorations.push(
        Decoration.node(block.pos, block.pos + block.node.nodeSize, {
          class: `mark-block mark-${span.severity}`,
          "data-mark": span.id,
        }),
      );
      continue;
    }
    const from = scalarToPos(block.node, block.pos, span.start);
    const to = scalarToPos(
      block.node,
      block.pos,
      Math.min(span.end, textLength),
    );
    if (to > from) {
      decorations.push(
        Decoration.inline(from, to, {
          class: `mark mark-${span.severity} mark-${span.role}`,
          "data-mark": span.id,
          title: span.title,
        }),
      );
    }
  }
  return DecorationSet.create(doc, decorations);
}

/** Decorations for diagnostics. Update with `view.dispatch(tr.setMeta(marksKey, spans))`. */
export const AstenMarks = Extension.create({
  name: "astenMarks",
  addProseMirrorPlugins() {
    return [
      new Plugin<DecorationSet>({
        key: marksKey,
        state: {
          init: () => DecorationSet.empty,
          apply(tr, set, _old, state) {
            const spans = tr.getMeta(marksKey) as MarkSpan[] | undefined;
            if (spans) return buildMarkDecorations(state.doc, spans);
            return tr.docChanged ? set.map(tr.mapping, tr.doc) : set;
          },
        },
        props: {
          decorations(state) {
            return marksKey.getState(state);
          },
        },
      }),
    ];
  },
});

export function marksToSpans(
  diagnostics: AstenDiagnostic[],
  documentId: string,
): MarkSpan[] {
  const spans: MarkSpan[] = [];
  for (const diagnostic of diagnostics) {
    if (diagnostic.suppressed_by) continue;
    const primary = diagnostic.primary;
    if (primary && primary.document === documentId && primary.block) {
      spans.push({
        id: diagnostic.id,
        severity: diagnostic.severity,
        block: primary.block,
        start: primary.start,
        end: primary.end,
        title: diagnostic.title,
        role: "primary",
      });
    }
    for (const [location] of diagnostic.related) {
      if (location.document === documentId && location.block) {
        spans.push({
          id: diagnostic.id,
          severity: diagnostic.severity,
          block: location.block,
          start: location.start,
          end: location.end,
          title: diagnostic.title,
          role: "related",
        });
      }
    }
  }
  return spans;
}

// ───────────────────────── entity tokens ─────────────────────────

export const entityKey = new PluginKey<EntityTokenState>("astenEntities");

interface EntityForm {
  key: string;
  form: string;
}

function entityForms(entities: CanonEntity[]): EntityForm[] {
  const forms: EntityForm[] = [];
  for (const entity of entities) {
    const key = entity.name.trim().toLowerCase().split(/\s+/).join("_");
    for (const form of [entity.name, ...entity.aliases]) {
      const trimmed = form.trim();
      if (trimmed.length >= 2) forms.push({ key, form: trimmed });
    }
  }
  // Longest first, so "Captain Oren" wins over "Oren".
  forms.sort((a, b) => b.form.length - a.form.length);
  return forms;
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function buildEntityDecorations(
  doc: PmNode,
  entities: CanonEntity[],
): DecorationSet {
  const forms = entityForms(entities);
  if (forms.length === 0) return DecorationSet.empty;
  const pattern = new RegExp(
    `(^|[^\\p{L}\\p{N}])(${forms.map((f) => escapeRegExp(f.form)).join("|")})(?![\\p{L}\\p{N}]{5,})`,
    "giu",
  );
  const byForm = new Map(forms.map((f) => [f.form.toLowerCase(), f.key]));
  const decorations: Decoration[] = [];
  doc.descendants((node, pos) => {
    if (!node.isText || !node.text) return true;
    const text = node.text;
    pattern.lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = pattern.exec(text)) !== null) {
      const lead = match[1].length;
      const start = pos + match.index + lead;
      const end = start + match[2].length;
      const key = byForm.get(match[2].toLowerCase());
      if (key) {
        decorations.push(
          Decoration.inline(start, end, {
            class: "entity-token",
            "data-entity": key,
          }),
        );
      }
      if (match[0].length === 0) pattern.lastIndex++;
    }
    return true;
  });
  return DecorationSet.create(doc, decorations);
}

/** Subtle colouring of cast names so hovering can show the fact card. */
interface EntityTokenState {
  entities: CanonEntity[];
  set: DecorationSet;
}

export const EntityTokens = Extension.create({
  name: "astenEntities",
  addProseMirrorPlugins() {
    return [
      new Plugin<EntityTokenState>({
        key: entityKey,
        state: {
          init: () => ({ entities: [], set: DecorationSet.empty }),
          apply(tr, value, _old, state) {
            const entities = tr.getMeta(entityKey) as CanonEntity[] | undefined;
            if (entities) {
              return {
                entities,
                set: buildEntityDecorations(state.doc, entities),
              };
            }
            if (tr.docChanged) {
              // The cast is kept in plugin state, so a name typed just now is
              // recognised without waiting for a save.
              return {
                entities: value.entities,
                set: value.entities.length
                  ? buildEntityDecorations(state.doc, value.entities)
                  : value.set.map(tr.mapping, tr.doc),
              };
            }
            return value;
          },
        },
        props: {
          decorations(state) {
            return entityKey.getState(state)?.set;
          },
        },
      }),
    ];
  },
});

export { buildEntityDecorations };

// ───────────────────────── paragraph alignment ─────────────────────────

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    textAlign: {
      /** Align the paragraphs and headings in the selection. */
      setTextAlign: (
        alignment: "left" | "center" | "right" | "justify",
      ) => ReturnType;
      unsetTextAlign: () => ReturnType;
    };
  }
}

const ALIGNMENTS = ["left", "center", "right", "justify"];

/**
 * Alignment as a block attribute (`textAlign`), so it rides in the block's
 * fields and the history like any other attribute. Left is the default and
 * is not stored.
 */
export const TextAlign = Extension.create({
  name: "textAlign",
  addGlobalAttributes() {
    return [
      {
        types: ["paragraph", "heading"],
        attributes: {
          textAlign: {
            default: null,
            parseHTML: (element: HTMLElement) => {
              const value = element.style.textAlign;
              return ALIGNMENTS.includes(value) && value !== "left"
                ? value
                : null;
            },
            renderHTML: (attributes: Record<string, unknown>) =>
              attributes.textAlign
                ? { style: `text-align: ${String(attributes.textAlign)}` }
                : {},
          },
        },
      },
    ];
  },
  addCommands() {
    return {
      setTextAlign:
        (alignment) =>
        ({ commands }) => {
          if (!ALIGNMENTS.includes(alignment)) return false;
          const value = alignment === "left" ? null : alignment;
          return ["paragraph", "heading"]
            .map((type) =>
              commands.updateAttributes(type, { textAlign: value }),
            )
            .some(Boolean);
        },
      unsetTextAlign:
        () =>
        ({ commands }) =>
          ["paragraph", "heading"]
            .map((type) => commands.resetAttributes(type, "textAlign"))
            .some(Boolean),
    };
  },
});
