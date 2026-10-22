import {
  type FocusEvent,
  type FormEvent,
  type KeyboardEvent,
  useMemo,
  useRef,
  useState,
} from "react";
import type {
  AstenDiagnostic,
  DocumentSummary,
  DocumentKind,
} from "@asten/contracts";

interface Props {
  /** Remembers which nodes are folded, per project. */
  projectId?: string;
  documents: DocumentSummary[];
  selectedId?: string;
  marks: AstenDiagnostic[];
  onSelect: (document: DocumentSummary) => void;
  onCreate: (input: {
    title: string;
    kind: DocumentKind;
    parent_id?: string | null;
  }) => void;
  onToggleCompile: (document: DocumentSummary) => void;
  onRename: (document: DocumentSummary, title: string) => void;
  onMove: (
    document: DocumentSummary,
    position: number,
    parent_id: string | null,
  ) => void;
  onDelete: (document: DocumentSummary) => void;
  creating?: boolean;
}

const KIND_LABEL: Record<DocumentKind, string> = {
  manuscript: "manuscript",
  chapter: "chapter",
  scene: "scene",
  wiki: "wiki",
  note: "note",
  source: "source",
};

/**
 * The binder: manuscript, chapters, scenes, notes and sources in one tree.
 *
 * Keyboard: Up and Down move between rows, Right opens a folder or steps
 * into it, Left closes a folder or steps out to its parent, Home and End
 * jump, Enter opens the page, F2 renames, Delete asks before deleting, and
 * Ctrl (or Cmd) with Up or Down reorders among siblings. One row carries the
 * tab stop; the arrows move it.
 */
export function Binder({
  projectId,
  documents,
  selectedId,
  marks,
  onSelect,
  onCreate,
  onToggleCompile,
  onRename,
  onMove,
  onDelete,
  creating,
}: Props) {
  const [adding, setAdding] = useState(false);
  const [collapsed, setCollapsed] = useCollapsed(projectId);
  const [activeId, setActiveId] = useState<string>();
  const [renamingId, setRenamingId] = useState<string>();
  const treeRef = useRef<HTMLDivElement>(null);

  const childrenOf = useMemo(() => {
    const map = new Map<string | null, DocumentSummary[]>();
    for (const document of documents) {
      const key = document.parent_id ?? null;
      const list = map.get(key) ?? [];
      list.push(document);
      map.set(key, list);
    }
    for (const list of map.values()) list.sort(byPosition);
    return map;
  }, [documents]);
  const roots = useMemo(() => childrenOf.get(null) ?? [], [childrenOf]);
  const visible = useMemo(() => {
    const out: DocumentSummary[] = [];
    const walk = (nodes: DocumentSummary[]) => {
      for (const node of nodes) {
        out.push(node);
        if (!collapsed.has(node.id)) walk(childrenOf.get(node.id) ?? []);
      }
    };
    walk(roots);
    return out;
  }, [roots, childrenOf, collapsed]);

  // Roving tabindex: the row last focused, else the open page, else the first.
  const currentId = visible.some((node) => node.id === activeId)
    ? activeId
    : visible.some((node) => node.id === selectedId)
      ? selectedId
      : visible[0]?.id;

  function focusRow(id: string | undefined) {
    if (!id) return;
    setActiveId(id);
    treeRef.current
      ?.querySelector<HTMLElement>(
        `[role="treeitem"][data-id="${CSS.escape(id)}"]`,
      )
      ?.focus();
  }

  function swap(node: DocumentSummary, direction: -1 | 1) {
    const siblings = childrenOf.get(node.parent_id ?? null) ?? [];
    const other = siblings[siblings.indexOf(node) + direction];
    if (!other) return;
    onMove(
      node,
      other.position === node.position
        ? other.position + direction
        : other.position,
      node.parent_id,
    );
    onMove(other, node.position, other.parent_id);
  }

  function onTreeKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    const target = event.target as HTMLElement;
    // Keys inside the rename field or the row's tool buttons are theirs.
    if (target.getAttribute("role") !== "treeitem") return;
    const node = documents.find((item) => item.id === target.dataset.id);
    if (!node) return;
    const index = visible.indexOf(node);
    const kids = childrenOf.get(node.id) ?? [];
    const open = kids.length > 0 && !collapsed.has(node.id);
    const reorder = event.ctrlKey || event.metaKey;
    switch (event.key) {
      case "ArrowDown":
        if (reorder) swap(node, 1);
        else focusRow(visible[index + 1]?.id);
        break;
      case "ArrowUp":
        if (reorder) swap(node, -1);
        else focusRow(visible[index - 1]?.id);
        break;
      case "ArrowRight":
        if (kids.length === 0) return;
        if (open) focusRow(kids[0].id);
        else setCollapsed(node.id, false);
        break;
      case "ArrowLeft":
        if (open) setCollapsed(node.id, true);
        else if (node.parent_id) focusRow(node.parent_id);
        else return;
        break;
      case "Home":
        focusRow(visible[0]?.id);
        break;
      case "End":
        focusRow(visible[visible.length - 1]?.id);
        break;
      case "Enter":
        onSelect(node);
        break;
      case "F2":
        setRenamingId(node.id);
        break;
      case "Delete":
        onDelete(node);
        break;
      default:
        return;
    }
    event.preventDefault();
  }

  function onTreeFocus(event: FocusEvent<HTMLDivElement>) {
    const target = event.target as HTMLElement;
    if (target.getAttribute("role") === "treeitem" && target.dataset.id)
      setActiveId(target.dataset.id);
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    onCreate({
      title: String(form.get("title") ?? ""),
      kind: String(form.get("kind") ?? "scene") as DocumentKind,
      parent_id: String(form.get("parent_id") || "") || null,
    });
    event.currentTarget.reset();
    setAdding(false);
  }

  const tree: TreeContext = {
    childrenOf,
    collapsed,
    currentId,
    selectedId,
    renamingId,
    marks,
    onSelect,
    onToggleCompile,
    onRename,
    onDelete,
    onSwap: swap,
    onToggleCollapsed: setCollapsed,
    onRenameStart: setRenamingId,
    onRenameEnd: (id, refocus) => {
      setRenamingId(undefined);
      if (refocus) focusRow(id);
    },
  };

  return (
    <>
      <nav className="tree binder-tree" aria-label="Binder">
        {roots.length === 0 ? (
          <p className="hint">Nothing in the binder yet. Add a scene below.</p>
        ) : (
          <div
            role="tree"
            aria-label="Binder"
            ref={treeRef}
            onKeyDown={onTreeKeyDown}
            onFocus={onTreeFocus}
          >
            {roots.map((document) => (
              <BinderNode
                key={document.id}
                document={document}
                depth={0}
                tree={tree}
              />
            ))}
          </div>
        )}
      </nav>
      {adding ? (
        <form className="new-project binder-new" onSubmit={submit}>
          <input
            name="title"
            placeholder="Scene title"
            aria-label="Title"
            required
            autoFocus
          />
          <select name="kind" defaultValue="scene" aria-label="Kind">
            <option value="scene">Scene</option>
            <option value="chapter">Chapter</option>
            <option value="manuscript">Manuscript</option>
            <option value="note">Note</option>
            <option value="wiki">Wiki</option>
            <option value="source">Source</option>
          </select>
          <select name="parent_id" defaultValue="" aria-label="Place">
            <option value="">Top of binder</option>
            {documents.map((document) => (
              <option key={document.id} value={document.id}>
                Under {document.title}
              </option>
            ))}
          </select>
          <div className="row">
            <button
              type="submit"
              className="btn btn--small btn--primary"
              disabled={creating}
            >
              Add
            </button>
            <button
              type="button"
              className="btn btn--small btn--ghost"
              onClick={() => setAdding(false)}
            >
              Cancel
            </button>
          </div>
        </form>
      ) : (
        <button
          type="button"
          className="binder-add linkish"
          onClick={() => setAdding(true)}
        >
          + Add to the binder
        </button>
      )}
    </>
  );
}

function byPosition(a: DocumentSummary, b: DocumentSummary): number {
  return a.position - b.position || a.created_at.localeCompare(b.created_at);
}

/* ───────── Folded nodes, remembered per project ───────── */

function readCollapsed(key: string): Set<string> {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(key) ?? "[]");
    return new Set(
      Array.isArray(parsed)
        ? parsed.filter((id): id is string => typeof id === "string")
        : [],
    );
  } catch {
    return new Set();
  }
}

function useCollapsed(
  projectId: string | undefined,
): [Set<string>, (id: string, collapsed: boolean) => void] {
  const key = `asten-binder-collapsed:${projectId ?? "none"}`;
  const [stored, setStored] = useState(() => ({
    key,
    ids: readCollapsed(key),
  }));
  // A different project: start from its own remembered set.
  if (stored.key !== key) setStored({ key, ids: readCollapsed(key) });
  const ids = stored.key === key ? stored.ids : readCollapsed(key);
  const update = (id: string, collapsed: boolean) => {
    const next = new Set(ids);
    if (collapsed) next.add(id);
    else next.delete(id);
    try {
      localStorage.setItem(key, JSON.stringify([...next]));
    } catch {
      // Fine without storage; the fold still applies to this session.
    }
    setStored({ key, ids: next });
  };
  return [ids, update];
}

/* ───────── Rows ───────── */

interface TreeContext {
  childrenOf: Map<string | null, DocumentSummary[]>;
  collapsed: Set<string>;
  currentId?: string;
  selectedId?: string;
  renamingId?: string;
  marks: AstenDiagnostic[];
  onSelect: (document: DocumentSummary) => void;
  onToggleCompile: (document: DocumentSummary) => void;
  onRename: (document: DocumentSummary, title: string) => void;
  onDelete: (document: DocumentSummary) => void;
  onSwap: (document: DocumentSummary, direction: -1 | 1) => void;
  onToggleCollapsed: (id: string, collapsed: boolean) => void;
  onRenameStart: (id: string) => void;
  onRenameEnd: (id: string, refocus: boolean) => void;
}

function BinderNode({
  document,
  depth,
  tree,
}: {
  document: DocumentSummary;
  depth: number;
  tree: TreeContext;
}) {
  const kids = tree.childrenOf.get(document.id) ?? [];
  const siblings = tree.childrenOf.get(document.parent_id ?? null) ?? [];
  const index = siblings.indexOf(document);
  const hasKids = kids.length > 0;
  const folded = hasKids && tree.collapsed.has(document.id);
  const count = tree.marks.filter(
    (mark) => mark.primary?.document === document.id,
  ).length;
  const selected = document.id === tree.selectedId;
  const titleId = `binder-title-${document.id}`;

  return (
    <div
      role="treeitem"
      className="binder-node"
      data-id={document.id}
      tabIndex={document.id === tree.currentId ? 0 : -1}
      aria-level={depth + 1}
      aria-selected={selected}
      aria-expanded={hasKids ? !folded : undefined}
      aria-labelledby={titleId}
      style={{ paddingLeft: depth * 12 }}
    >
      <div
        className={`binder-row${selected ? " selected" : ""}${document.exclude_from_compile ? " excluded" : ""}`}
      >
        {hasKids && (
          <button
            type="button"
            className="binder-twist"
            tabIndex={-1}
            aria-hidden="true"
            title={folded ? "Expand" : "Collapse"}
            onClick={() => tree.onToggleCollapsed(document.id, !folded)}
          >
            {folded ? "▸" : "▾"}
          </button>
        )}
        {tree.renamingId === document.id ? (
          <RenameField
            initial={document.title}
            label={`Rename ${document.title}`}
            onDone={(value, refocus) => {
              const next = value?.trim();
              if (next && next !== document.title)
                tree.onRename(document, next);
              tree.onRenameEnd(document.id, refocus);
            }}
          />
        ) : (
          <div
            className="binder-title"
            id={titleId}
            onClick={() => tree.onSelect(document)}
            onDoubleClick={() => tree.onRenameStart(document.id)}
            title="Double-click to rename"
          >
            <span>{document.title}</span>
            <small className="mono">
              {KIND_LABEL[document.kind]}
              {document.scene.story_time
                ? ` · ${document.scene.story_time}`
                : ""}
            </small>
          </div>
        )}
        {count > 0 && (
          <span
            className="binder-marks mono"
            title={`${count} mark${count === 1 ? "" : "s"}`}
          >
            {count}
          </span>
        )}
        {selected && (
          <div className="binder-tools">
            <button
              type="button"
              className="linkish"
              onClick={() => tree.onSwap(document, -1)}
              disabled={index === 0}
              title="Move up"
              aria-label="Move up"
            >
              ↑
            </button>
            <button
              type="button"
              className="linkish"
              onClick={() => tree.onSwap(document, 1)}
              disabled={index === siblings.length - 1}
              title="Move down"
              aria-label="Move down"
            >
              ↓
            </button>
            <button
              type="button"
              className="linkish"
              onClick={() => tree.onToggleCompile(document)}
              title={
                document.exclude_from_compile
                  ? "Include in compile"
                  : "Exclude from compile"
              }
            >
              {document.exclude_from_compile ? "include" : "exclude"}
            </button>
            <button
              type="button"
              className="linkish rubric"
              onClick={() => tree.onDelete(document)}
              title="Delete"
            >
              delete
            </button>
          </div>
        )}
      </div>
      {hasKids && !folded && (
        <div role="group">
          {kids.map((child) => (
            <BinderNode
              key={child.id}
              document={child}
              depth={depth + 1}
              tree={tree}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/** One rename session: Enter commits, Escape cancels, leaving commits. */
function RenameField({
  initial,
  label,
  onDone,
}: {
  initial: string;
  label: string;
  onDone: (value: string | null, refocus: boolean) => void;
}) {
  const [value, setValue] = useState(initial);
  const done = useRef(false);
  function finish(commit: boolean, refocus: boolean) {
    if (done.current) return;
    done.current = true;
    onDone(commit ? value : null, refocus);
  }
  return (
    <form
      className="binder-rename"
      onSubmit={(event) => {
        event.preventDefault();
        finish(true, true);
      }}
    >
      <input
        value={value}
        autoFocus
        aria-label={label}
        onChange={(event) => setValue(event.target.value)}
        onBlur={() => finish(true, false)}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            event.stopPropagation();
            finish(false, true);
          }
        }}
      />
    </form>
  );
}
