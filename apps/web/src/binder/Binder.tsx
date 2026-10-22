import { type FormEvent, useState } from "react";
import type {
  AstenDiagnostic,
  DocumentKind,
  DocumentSummary,
} from "@asten/contracts";

interface Props {
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

/** The binder: manuscript, chapters, scenes, notes and sources in one tree. */
export function Binder({
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
  const roots = sorted(documents.filter((document) => !document.parent_id));

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

  return (
    <>
      <nav className="tree binder-tree" aria-label="Binder">
        {roots.length === 0 && (
          <p className="hint">Empty. Add a scene below.</p>
        )}
        {roots.map((document, index) => (
          <BinderNode
            key={document.id}
            document={document}
            siblings={roots}
            index={index}
            documents={documents}
            depth={0}
            selectedId={selectedId}
            marks={marks}
            onSelect={onSelect}
            onToggleCompile={onToggleCompile}
            onRename={onRename}
            onMove={onMove}
            onDelete={onDelete}
          />
        ))}
      </nav>
      {adding ? (
        <form className="new-project binder-new" onSubmit={submit}>
          <input name="title" placeholder="Scene title" required autoFocus />
          <select name="kind" defaultValue="scene">
            <option value="scene">Scene</option>
            <option value="chapter">Chapter</option>
            <option value="manuscript">Manuscript</option>
            <option value="note">Note</option>
            <option value="wiki">Wiki</option>
            <option value="source">Source</option>
          </select>
          <select name="parent_id" defaultValue="">
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

function sorted(items: DocumentSummary[]): DocumentSummary[] {
  return [...items].sort(
    (a, b) =>
      a.position - b.position || a.created_at.localeCompare(b.created_at),
  );
}

function BinderNode({
  document,
  siblings,
  index,
  documents,
  depth,
  selectedId,
  marks,
  onSelect,
  onToggleCompile,
  onRename,
  onMove,
  onDelete,
}: {
  document: DocumentSummary;
  siblings: DocumentSummary[];
  index: number;
  documents: DocumentSummary[];
  depth: number;
  selectedId?: string;
  marks: AstenDiagnostic[];
  onSelect: (document: DocumentSummary) => void;
  onToggleCompile: (document: DocumentSummary) => void;
  onRename: (document: DocumentSummary, title: string) => void;
  onMove: (
    document: DocumentSummary,
    position: number,
    parent_id: string | null,
  ) => void;
  onDelete: (document: DocumentSummary) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [title, setTitle] = useState(document.title);
  const children = sorted(
    documents.filter((child) => child.parent_id === document.id),
  );
  const count = marks.filter(
    (mark) => mark.primary?.document === document.id,
  ).length;
  const selected = document.id === selectedId;

  function swap(direction: -1 | 1) {
    const other = siblings[index + direction];
    if (!other) return;
    onMove(
      document,
      other.position === document.position
        ? other.position + direction
        : other.position,
      document.parent_id,
    );
    onMove(other, document.position, other.parent_id);
  }

  return (
    <div className="binder-node" style={{ paddingLeft: depth * 12 }}>
      <div
        className={`binder-row${selected ? " selected" : ""}${document.exclude_from_compile ? " excluded" : ""}`}
      >
        {editing ? (
          <form
            className="binder-rename"
            onSubmit={(event) => {
              event.preventDefault();
              if (title.trim() && title !== document.title)
                onRename(document, title.trim());
              setEditing(false);
            }}
          >
            <input
              value={title}
              autoFocus
              onChange={(event) => setTitle(event.target.value)}
              onBlur={() => setEditing(false)}
            />
          </form>
        ) : (
          <button
            type="button"
            className="binder-title"
            onClick={() => onSelect(document)}
            onDoubleClick={() => setEditing(true)}
            title="Double-click to rename"
          >
            <span>{document.title}</span>
            <small className="mono">
              {KIND_LABEL[document.kind]}
              {document.scene.story_time
                ? ` · ${document.scene.story_time}`
                : ""}
            </small>
          </button>
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
              onClick={() => swap(-1)}
              disabled={index === 0}
              title="Move up"
            >
              ↑
            </button>
            <button
              type="button"
              className="linkish"
              onClick={() => swap(1)}
              disabled={index === siblings.length - 1}
              title="Move down"
            >
              ↓
            </button>
            <button
              type="button"
              className="linkish"
              onClick={() => onToggleCompile(document)}
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
              onClick={() => onDelete(document)}
              title="Delete"
            >
              delete
            </button>
          </div>
        )}
      </div>
      {children.map((child, childIndex) => (
        <BinderNode
          key={child.id}
          document={child}
          siblings={children}
          index={childIndex}
          documents={documents}
          depth={depth + 1}
          selectedId={selectedId}
          marks={marks}
          onSelect={onSelect}
          onToggleCompile={onToggleCompile}
          onRename={onRename}
          onMove={onMove}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
