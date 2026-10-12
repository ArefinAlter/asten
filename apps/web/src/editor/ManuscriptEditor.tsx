import { EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import CharacterCount from "@tiptap/extension-character-count";
import Underline from "@tiptap/extension-underline";
import { Selection } from "@tiptap/pm/state";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import type {
  AstenDiagnostic,
  AstenDocument,
  CanonEntity,
  Location,
} from "@asten/contracts";
import {
  AstenMarks,
  BlockId,
  Deletion,
  EntityTokens,
  Insertion,
  SceneBreak,
  entityKey,
  marksKey,
  marksToSpans,
} from "./extensions";
import { FactCardPopover } from "./FactCardPopover";
import { SceneHeader } from "./SceneHeader";
import { ApiError } from "../lib/asten-client";
import { countWords, findBlock, scalarToPos, toEditorDoc } from "../lib/pm";
import { severityWord } from "../lib/format";

export interface Jump {
  block: string | null | undefined;
  start: number;
  end: number;
  nonce: number;
}

type SaveState = "saved" | "unsaved" | "saving" | "failed" | "conflict";

interface Props {
  projectId: string;
  document: AstenDocument;
  diagnostics: AstenDiagnostic[];
  entities: CanonEntity[];
  pageView: boolean;
  trackChanges: boolean;
  findQuery: string;
  showBlame: boolean;
  blame?: Record<
    string,
    { start: number; end: number; actor: string; seq: number; at: string }[]
  >;
  selectedMarkId?: string;
  jump?: Jump;
  /**
   * Persist the content against the head sequence the editor last saw. Must
   * resolve with the saved document so the editor learns the new head.
   */
  onSave: (content: unknown, headSeq: number) => Promise<AstenDocument>;
  onSelectMark: (id: string) => void;
  onSelectEntity: (key: string) => void;
  saving?: boolean;
  readOnlyLabel?: string;
}

interface MarginCard {
  id: string;
  top: number;
  severity: string;
  code: string;
  title: string;
}

const SAVE_DEBOUNCE_MS = 900;
const RETRY_MS = 5000;

export function ManuscriptEditor({
  projectId,
  document,
  diagnostics,
  entities,
  pageView,
  trackChanges,
  findQuery,
  showBlame,
  blame,
  selectedMarkId,
  jump,
  onSave,
  onSelectMark,
  onSelectEntity,
  saving,
  readOnlyLabel,
}: Props) {
  const [matches, setMatches] = useState(0);
  const [saveState, setSaveState] = useState<SaveState>("saved");
  const [margin, setMargin] = useState<MarginCard[]>([]);
  const [hover, setHover] = useState<{
    entity: string;
    x: number;
    y: number;
  } | null>(null);
  const findCursor = useRef(0);
  const pageRef = useRef<HTMLDivElement>(null);
  const saveTimer = useRef<number | undefined>(undefined);
  const retryTimer = useRef<number | undefined>(undefined);

  // The save pipeline. `syncedSeq` is the head the editor's content is based
  // on; it advances only from our own successful save or from an external
  // version we chose to adopt. `editVersion` counts local edits and
  // `sentVersion` the last one persisted, so "dirty" is a fact about the
  // server, not about whether a request was sent.
  const syncedSeq = useRef(document.head_seq);
  const editVersion = useRef(0);
  const sentVersion = useRef(0);
  const inFlight = useRef(false);
  const latestDocument = useRef(document);
  const onSaveRef = useRef(onSave);

  const editor = useEditor({
    immediatelyRender: false,
    extensions: [
      StarterKit.configure({ heading: { levels: [1, 2, 3] } }),
      Placeholder.configure({ placeholder: "The sentence stays yours." }),
      CharacterCount,
      Underline,
      BlockId,
      Insertion,
      Deletion,
      SceneBreak,
      AstenMarks,
      EntityTokens,
    ],
    content: toEditorDoc(document.content_json),
    editorProps: {
      attributes: { class: "page-prose", spellcheck: "true" },
    },
    onUpdate: () => {
      editVersion.current += 1;
      setSaveState("unsaved");
      window.clearTimeout(saveTimer.current);
      saveTimer.current = window.setTimeout(() => {
        void flushRef.current();
      }, SAVE_DEBOUNCE_MS);
    },
    onBlur: () => {
      if (editVersion.current === sentVersion.current) return;
      window.clearTimeout(saveTimer.current);
      void flushRef.current();
    },
  });

  /** Replace the editor's content with the server's, keeping the caret. */
  const adoptExternal = useCallback(() => {
    if (!editor) return;
    const current = latestDocument.current;
    const next = toEditorDoc(current.content_json);
    try {
      const node = editor.schema.nodeFromJSON(next);
      if (!editor.state.doc.eq(node)) {
        const { from, to } = editor.state.selection;
        const tr = editor.state.tr
          .replaceWith(0, editor.state.doc.content.size, node.content)
          .setMeta("addToHistory", false)
          .setMeta("preventUpdate", true);
        const size = tr.doc.content.size;
        const anchor = Selection.near(tr.doc.resolve(Math.min(from, size)));
        const head = Selection.near(tr.doc.resolve(Math.min(to, size)));
        tr.setSelection(
          anchor.from === head.from
            ? anchor
            : Selection.near(tr.doc.resolve(anchor.from)),
        );
        editor.view.dispatch(tr);
      }
    } catch {
      // Malformed content from the server: leave the page as it is.
      return;
    }
    syncedSeq.current = current.head_seq;
    sentVersion.current = editVersion.current;
    setSaveState("saved");
  }, [editor]);

  /** Look at the server's head and decide: nothing, adopt, or conflict. */
  const reconcile = useCallback(() => {
    if (!editor || inFlight.current) return;
    const current = latestDocument.current;
    if (current.head_seq === syncedSeq.current) return;
    const dirty = editVersion.current !== sentVersion.current;
    if (dirty) setSaveState("conflict");
    else adoptExternal();
  }, [editor, adoptExternal]);

  const flush = useCallback(
    async (force = false) => {
      if (!editor || inFlight.current) return;
      if (!force && editVersion.current === sentVersion.current) return;
      window.clearTimeout(retryTimer.current);
      inFlight.current = true;
      const version = editVersion.current;
      setSaveState("saving");
      try {
        const saved = await onSaveRef.current(
          editor.getJSON(),
          syncedSeq.current,
        );
        syncedSeq.current = saved.head_seq;
        sentVersion.current = version;
        inFlight.current = false;
        if (editVersion.current !== version) {
          void flushRef.current();
        } else {
          setSaveState("saved");
          reconcile();
        }
      } catch (error) {
        inFlight.current = false;
        if (error instanceof ApiError && error.status === 409) {
          setSaveState("conflict");
        } else {
          setSaveState("failed");
          retryTimer.current = window.setTimeout(() => {
            void flushRef.current();
          }, RETRY_MS);
        }
      }
    },
    [editor, reconcile],
  );
  const flushRef = useRef(flush);
  // Latest-value refs for timers and editor callbacks, written after render.
  useLayoutEffect(() => {
    latestDocument.current = document;
    onSaveRef.current = onSave;
    flushRef.current = flush;
  });

  // Flush on unmount (switching documents) rather than lose the last edits.
  useEffect(() => {
    return () => {
      window.clearTimeout(saveTimer.current);
      window.clearTimeout(retryTimer.current);
      if (editor && editVersion.current !== sentVersion.current) {
        void onSaveRef
          .current(editor.getJSON(), syncedSeq.current)
          .catch(() => undefined);
      }
    };
  }, [editor]);

  // External changes to the document (another client, a repair, a merge).
  useEffect(() => {
    reconcile();
  }, [document.id, document.head_seq, reconcile]);

  function keepMine() {
    syncedSeq.current = latestDocument.current.head_seq;
    void flush(true);
  }

  // Marks and entity tokens as decorations.
  useEffect(() => {
    if (!editor) return;
    const spans = marksToSpans(diagnostics, document.id);
    editor.view.dispatch(
      editor.state.tr.setMeta(marksKey, spans).setMeta("addToHistory", false),
    );
  }, [editor, diagnostics, document.id, document.head_seq]);

  useEffect(() => {
    if (!editor) return;
    editor.view.dispatch(
      editor.state.tr
        .setMeta(entityKey, entities)
        .setMeta("addToHistory", false),
    );
  }, [editor, entities, document.head_seq]);

  // Position the margin cards beside their marks.
  const layoutMargin = useCallback(() => {
    const page = pageRef.current;
    if (!page || !editor) return;
    const pageTop = page.getBoundingClientRect().top;
    const cards: MarginCard[] = [];
    const seen = new Set<string>();
    page.querySelectorAll<HTMLElement>("[data-mark]").forEach((element) => {
      const id = element.dataset.mark;
      if (!id || seen.has(id)) return;
      if (
        !element.classList.contains("mark-primary") &&
        !element.classList.contains("mark-block")
      )
        return;
      const diagnostic = diagnostics.find((d) => d.id === id);
      if (!diagnostic) return;
      seen.add(id);
      cards.push({
        id,
        top: element.getBoundingClientRect().top - pageTop,
        severity: diagnostic.severity,
        code: diagnostic.code,
        title: diagnostic.title,
      });
    });
    cards.sort((a, b) => a.top - b.top);
    // Keep cards from overlapping: push down by card height.
    let last = -Infinity;
    for (const card of cards) {
      if (card.top < last + 56) card.top = last + 56;
      last = card.top;
    }
    setMargin(cards);
  }, [diagnostics, editor]);

  useLayoutEffect(() => {
    layoutMargin();
    const page = pageRef.current;
    if (!page) return;
    const observer = new ResizeObserver(() => layoutMargin());
    observer.observe(page);
    return () => observer.disconnect();
  }, [layoutMargin, document.head_seq, pageView]);

  // Find.
  useEffect(() => {
    if (!editor || !findQuery.trim()) {
      setMatches(0);
      return;
    }
    const text = editor.getText().toLowerCase();
    const needle = findQuery.toLowerCase();
    let count = 0;
    let index = 0;
    while (index < text.length) {
      const found = text.indexOf(needle, index);
      if (found < 0) break;
      count += 1;
      index = found + needle.length;
    }
    setMatches(count);
    findCursor.current = -1;
  }, [editor, findQuery, document.head_seq]);

  // Jump to a location (from the Scales, a fact card, a mention list).
  useEffect(() => {
    if (!editor || !jump) return;
    if (jump.block) {
      const block = findBlock(editor.state.doc, jump.block);
      if (block) {
        const from = scalarToPos(block.node, block.pos, jump.start);
        const to =
          jump.end > jump.start
            ? scalarToPos(block.node, block.pos, jump.end)
            : from;
        editor
          .chain()
          .focus()
          .setTextSelection({ from, to })
          .scrollIntoView()
          .run();
        return;
      }
    }
    editor.chain().focus("start").scrollIntoView().run();
  }, [editor, jump?.nonce]); // eslint-disable-line react-hooks/exhaustive-deps

  function stepFind(direction: 1 | -1) {
    if (!editor) return;
    const needle = findQuery.trim().toLowerCase();
    if (!needle) return;
    const found: { from: number; to: number }[] = [];
    editor.state.doc.descendants((node, pos) => {
      if (!node.isText || !node.text) return;
      const text = node.text.toLowerCase();
      let start = 0;
      while (start < text.length) {
        const at = text.indexOf(needle, start);
        if (at < 0) break;
        found.push({ from: pos + at, to: pos + at + needle.length });
        start = at + needle.length;
      }
    });
    if (!found.length) return;
    const next =
      findCursor.current < 0
        ? direction === 1
          ? 0
          : found.length - 1
        : (findCursor.current + direction + found.length) % found.length;
    findCursor.current = next;
    const range = found[next];
    if (range)
      editor.chain().focus().setTextSelection(range).scrollIntoView().run();
  }

  function onPageClick(event: React.MouseEvent<HTMLDivElement>) {
    const target = (event.target as HTMLElement).closest<HTMLElement>(
      "[data-mark]",
    );
    if (target?.dataset.mark) onSelectMark(target.dataset.mark);
  }

  function onPageMove(event: React.MouseEvent<HTMLDivElement>) {
    const target = (event.target as HTMLElement).closest<HTMLElement>(
      ".entity-token",
    );
    if (target?.dataset.entity) {
      const rect = target.getBoundingClientRect();
      const page = pageRef.current?.getBoundingClientRect();
      if (page)
        setHover({
          entity: target.dataset.entity,
          x: rect.left - page.left,
          y: rect.bottom - page.top,
        });
    } else if (hover && !(event.target as HTMLElement).closest(".fact-card")) {
      setHover(null);
    }
  }

  if (!editor) return <p className="status">Opening the page…</p>;

  const words = countWords(editor.getJSON());
  const chars = editor.storage.characterCount?.characters?.() ?? 0;
  const blameSpans = showBlame && blame ? Object.entries(blame) : [];
  const status =
    saving || saveState === "saving"
      ? " · saving…"
      : saveState === "unsaved"
        ? " · unsaved"
        : saveState === "failed"
          ? " · not saved, retrying"
          : saveState === "conflict"
            ? " · conflict"
            : " · saved";

  return (
    <div className={`manuscript${pageView ? " page-view" : ""}`}>
      <SceneHeader projectId={projectId} document={document} />
      <div className="editor-toolbar" role="toolbar" aria-label="Manuscript">
        <button
          type="button"
          title="Bold"
          onClick={() => editor.chain().focus().toggleBold().run()}
        >
          <b>B</b>
        </button>
        <button
          type="button"
          title="Italic"
          onClick={() => editor.chain().focus().toggleItalic().run()}
        >
          <i>I</i>
        </button>
        <button
          type="button"
          title="Underline"
          onClick={() => editor.chain().focus().toggleUnderline().run()}
        >
          <u>U</u>
        </button>
        <button
          type="button"
          title="Heading"
          onClick={() =>
            editor.chain().focus().toggleHeading({ level: 2 }).run()
          }
        >
          H
        </button>
        <button
          type="button"
          title="Scene break"
          onClick={() => editor.chain().focus().setHorizontalRule().run()}
        >
          # # #
        </button>
        {trackChanges && (
          <>
            <button
              type="button"
              onClick={() => editor.chain().focus().setMark("insertion").run()}
            >
              Mark insert
            </button>
            <button
              type="button"
              onClick={() => editor.chain().focus().setMark("deletion").run()}
            >
              Mark delete
            </button>
            <button
              type="button"
              onClick={() =>
                editor
                  .chain()
                  .focus()
                  .unsetMark("insertion")
                  .unsetMark("deletion")
                  .run()
              }
            >
              Accept
            </button>
          </>
        )}
        {findQuery.trim() && (
          <>
            <button type="button" onClick={() => stepFind(-1)}>
              Previous
            </button>
            <button type="button" onClick={() => stepFind(1)}>
              Next
            </button>
          </>
        )}
        <span className="toolbar-stats">
          {readOnlyLabel ? <span className="chip">{readOnlyLabel}</span> : null}
          {words} words · {chars} characters
          {findQuery.trim() ? ` · ${matches} found` : ""}
          {status}
        </span>
      </div>
      {saveState === "conflict" && (
        <div className="save-conflict" role="alert">
          <span>
            This page changed elsewhere while you were writing. Keep your
            version, or reload theirs and lose your unsaved edits.
          </span>
          <button type="button" className="btn btn--small" onClick={keepMine}>
            Keep mine
          </button>
          <button
            type="button"
            className="btn btn--small btn--ghost"
            onClick={adoptExternal}
          >
            Reload theirs
          </button>
        </div>
      )}
      <div className="page-with-margin">
        <div
          className="page-sheet"
          ref={pageRef}
          onClick={onPageClick}
          onMouseMove={onPageMove}
          onMouseLeave={() => setHover(null)}
        >
          <p className="eyebrow mono">{document.kind}</p>
          <h1 className="page-title">{document.title}</h1>
          <EditorContent editor={editor} />
          {blameSpans.length > 0 && (
            <aside className="blame-legend">
              {Array.from(
                new Set(
                  blameSpans.flatMap(([, spans]) =>
                    spans.map((span) => span.actor),
                  ),
                ),
              ).map((actor) => (
                <span key={actor} className="chip">
                  {actor}
                </span>
              ))}
            </aside>
          )}
          {hover && (
            <FactCardPopover
              projectId={projectId}
              entity={hover.entity}
              documentId={document.id}
              x={hover.x}
              y={hover.y}
              onOpen={() => onSelectEntity(hover.entity)}
              onClose={() => setHover(null)}
            />
          )}
        </div>
        <aside className="page-margin" aria-label="Marks in the margin">
          {margin.map((card) => (
            <button
              key={card.id}
              type="button"
              className={`margin-mark mark-${card.severity}${selectedMarkId === card.id ? " selected" : ""}`}
              style={{ top: card.top }}
              onClick={() => onSelectMark(card.id)}
              title={`${severityWord(card.severity as never)}: ${card.title}`}
            >
              <code className="chip chip--code">{card.code}</code>
              <span>{card.title}</span>
            </button>
          ))}
        </aside>
      </div>
    </div>
  );
}

export type { Location };
