import type { FormEvent } from "react";
import type { DocumentKind, AstenDocument } from "@asten/contracts";

interface Props {
  documents: AstenDocument[];
  selectedId?: string;
  onSelect: (document: AstenDocument) => void;
  onCreate: (input: { title: string; kind: DocumentKind; parent_id?: string | null }) => void;
  onToggleCompile: (document: AstenDocument) => void;
  creating?: boolean;
}

export function Binder({
  documents,
  selectedId,
  onSelect,
  onCreate,
  onToggleCompile,
  creating,
}: Props) {
  const roots = documents.filter((document) => !document.parent_id);

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    onCreate({
      title: String(form.get("title") ?? ""),
      kind: String(form.get("kind") ?? "scene") as DocumentKind,
      parent_id: String(form.get("parent_id") || "") || null,
    });
    event.currentTarget.reset();
  }

  return (
    <>
      <nav className="tree binder-tree" aria-label="Binder">
        {roots.map((document) => (
          <BinderNode
            key={document.id}
            document={document}
            documents={documents}
            depth={0}
            selectedId={selectedId}
            onSelect={onSelect}
            onToggleCompile={onToggleCompile}
          />
        ))}
      </nav>
      <form className="new-project binder-new" onSubmit={submit}>
        <input name="title" placeholder="Scene title" required />
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
        <button type="submit" disabled={creating}>
          Add to binder
        </button>
      </form>
    </>
  );
}

function BinderNode({
  document,
  documents,
  depth,
  selectedId,
  onSelect,
  onToggleCompile,
}: {
  document: AstenDocument;
  documents: AstenDocument[];
  depth: number;
  selectedId?: string;
  onSelect: (document: AstenDocument) => void;
  onToggleCompile: (document: AstenDocument) => void;
}) {
  const children = documents.filter((child) => child.parent_id === document.id);
  return (
    <div style={{ paddingLeft: depth * 12 }}>
      <button
        className={document.id === selectedId ? "selected" : ""}
        type="button"
        onClick={() => onSelect(document)}
      >
        <span>
          {document.exclude_from_compile ? "· " : ""}
          {document.title}
        </span>
        <small>{document.kind}</small>
      </button>
      <button
        type="button"
        className="linkish compile-toggle"
        onClick={() => onToggleCompile(document)}
      >
        {document.exclude_from_compile ? "include" : "exclude"}
      </button>
      {children.map((child) => (
        <BinderNode
          key={child.id}
          document={child}
          documents={documents}
          depth={depth + 1}
          selectedId={selectedId}
          onSelect={onSelect}
          onToggleCompile={onToggleCompile}
        />
      ))}
    </div>
  );
}
