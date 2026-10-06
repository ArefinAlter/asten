import { EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import CharacterCount from "@tiptap/extension-character-count";
import Underline from "@tiptap/extension-underline";
import { useEffect, useState } from "react";
import type { AstenDocument } from "@asten/contracts";
import { Deletion, Insertion, SceneBreak } from "./marks";
import { countWords, toEditorDoc } from "../lib/pm";

interface Props {
  document: AstenDocument;
  pageView: boolean;
  trackChanges: boolean;
  findQuery: string;
  onSave: (content: unknown) => void;
  saving: boolean;
}

export function ManuscriptEditor({
  document,
  pageView,
  trackChanges,
  findQuery,
  onSave,
  saving,
}: Props) {
  const [matches, setMatches] = useState(0);
  const editor = useEditor({
    immediatelyRender: false,
    extensions: [
      StarterKit.configure({ heading: { levels: [1, 2, 3] } }),
      Placeholder.configure({
        placeholder: "The sentence stays yours.",
      }),
      CharacterCount,
      Underline,
      Insertion,
      Deletion,
      SceneBreak,
    ],
    content: toEditorDoc(document.content_json),
    onBlur: ({ editor: instance }) => {
      onSave(instance.getJSON());
    },
  });

  useEffect(() => {
    if (!editor) return;
    const next = toEditorDoc(document.content_json);
    const current = JSON.stringify(editor.getJSON());
    if (current !== JSON.stringify(next)) {
      editor.commands.setContent(next, { emitUpdate: false });
    }
  }, [document.id, document.revision, editor]);

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
  }, [editor, findQuery, document.revision]);

  if (!editor) return <p className="status">Opening the page…</p>;

  const words = countWords(editor.getJSON());
  const chars = editor.storage.characterCount?.characters?.() ?? 0;

  return (
    <div className={`manuscript${pageView ? " page-view" : ""}`}>
      <div className="editor-toolbar" role="toolbar" aria-label="Manuscript">
        <button
          type="button"
          onClick={() => editor.chain().focus().toggleBold().run()}
        >
          B
        </button>
        <button
          type="button"
          onClick={() => editor.chain().focus().toggleItalic().run()}
        >
          I
        </button>
        <button
          type="button"
          onClick={() => editor.chain().focus().toggleUnderline().run()}
        >
          U
        </button>
        <button
          type="button"
          onClick={() =>
            editor.chain().focus().toggleHeading({ level: 2 }).run()
          }
        >
          H
        </button>
        <button
          type="button"
          onClick={() => editor.chain().focus().setHorizontalRule().run()}
        >
          Break
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
        <button
          type="button"
          disabled={saving}
          onClick={() => onSave(editor.getJSON())}
        >
          {saving ? "Saving…" : "Save"}
        </button>
        <span className="toolbar-stats">
          {words} words · {chars} characters
          {findQuery.trim() ? ` · ${matches} found` : ""}
        </span>
      </div>
      <div className="page-sheet">
        <p className="eyebrow">{document.kind}</p>
        <h1>{document.title}</h1>
        <EditorContent editor={editor} />
      </div>
    </div>
  );
}
