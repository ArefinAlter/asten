import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import type { AstenDocument, MergePreview } from "@asten/contracts";
import { createAstenClient } from "../lib/asten-client";
import { shortHash, timeAgo } from "../lib/format";
import { ManuscriptEditor } from "../editor/ManuscriptEditor";

interface Props {
  projectId: string;
  documents: AstenDocument[];
  documentId?: string;
  onSelectDocument: (id: string) => void;
}

/** The git-like side of the desk: the hash-chained log, verification, snapshots, drafts and merges. */
export function History({
  projectId,
  documents,
  documentId,
  onSelectDocument,
}: Props) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const document =
    documents.find((item) => item.id === documentId) ?? documents[0];
  const history = useQuery({
    queryKey: ["history", projectId, document?.id, document?.head_seq],
    queryFn: () => client.history(projectId, document!.id),
    enabled: Boolean(document),
  });
  const drafts = useQuery({
    queryKey: ["drafts", projectId, document?.id],
    queryFn: () => client.listDrafts(projectId, document!.id),
    enabled: Boolean(document),
  });
  const [verifying, setVerifying] = useState<string>();
  const [preview, setPreview] = useState<MergePreview>();
  const [resolutions, setResolutions] = useState<Record<string, string>>({});
  const [editingDraft, setEditingDraft] = useState<string>();
  const [peek, setPeek] = useState<{ seq: number; text: string }>();

  const createDraft = useMutation({
    mutationFn: (name: string) =>
      client.createDraft(projectId, document!.id, name),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: ["drafts", projectId, document?.id],
      }),
  });
  const updateDraft = useMutation({
    mutationFn: (input: { id: string; content_json: unknown }) =>
      client.updateDraft(projectId, input.id, {
        content_json: input.content_json,
      }),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: ["drafts", projectId, document?.id],
      }),
  });
  const deleteDraft = useMutation({
    mutationFn: (id: string) => client.deleteDraft(projectId, id),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: ["drafts", projectId, document?.id],
      }),
  });
  const mergePreview = useMutation({
    mutationFn: (id: string) => client.mergePreview(projectId, id, resolutions),
    onSuccess: setPreview,
  });
  const merge = useMutation({
    mutationFn: (id: string) => client.mergeDraft(projectId, id, resolutions),
    onSuccess: async () => {
      setPreview(undefined);
      setResolutions({});
      await queryClient.invalidateQueries({
        queryKey: ["documents", projectId],
      });
      await queryClient.invalidateQueries({
        queryKey: ["drafts", projectId, document?.id],
      });
      await queryClient.invalidateQueries({
        queryKey: ["diagnostics", projectId],
      });
    },
  });

  async function verify() {
    if (!document) return;
    setVerifying("Checking every link in the chain…");
    try {
      const report = await client.verify(projectId, document.id);
      setVerifying(
        report.ok
          ? `Verified: ${report.events} events, head ${report.head_seq}, chain ${shortHash(report.chain_hash)}, Merkle ${shortHash(report.merkle_root)}.`
          : `Broken: ${report.error}`,
      );
    } catch (error) {
      setVerifying(
        error instanceof Error ? error.message : "The check did not finish.",
      );
    }
  }

  async function peekAt(seq: number) {
    if (!document) return;
    try {
      const content = await client.documentAt(projectId, document.id, seq);
      setPeek({ seq, text: plain(content) });
    } catch (error) {
      setPeek({
        seq,
        text:
          error instanceof Error
            ? error.message
            : "Could not read that revision.",
      });
    }
  }

  if (!document) {
    return (
      <section className="editor-placeholder register">
        <p className="eyebrow mono">History</p>
        <h1>Nothing to remember yet</h1>
        <p>
          Add a scene and every change becomes a link in a hash chain you can
          verify.
        </p>
      </section>
    );
  }

  const draftBeingEdited = drafts.data?.find(
    (draft) => draft.id === editingDraft,
  );
  const records = [...(history.data ?? [])].reverse();

  return (
    <section className="history register">
      <header>
        <p className="eyebrow mono">History</p>
        <h1>Every change, chained</h1>
        <div className="row wrap">
          <select
            value={document.id}
            onChange={(event) => onSelectDocument(event.target.value)}
            aria-label="Document"
          >
            {documents.map((item) => (
              <option key={item.id} value={item.id}>
                {item.title}
              </option>
            ))}
          </select>
          <button
            type="button"
            className="btn btn--small"
            onClick={() => void verify()}
          >
            Verify the chain
          </button>
          <span className="mono hint">
            head {document.head_seq} · {shortHash(document.chain_hash)}
          </span>
        </div>
        {verifying && (
          <p
            className={
              verifying.startsWith("Broken") ? "error" : "hint settled"
            }
          >
            {verifying}
          </p>
        )}
      </header>

      <div className="history-columns">
        <div>
          <h2>Events</h2>
          <ol className="event-log">
            {records.map((record) => (
              <li key={record.seq} className="event-row">
                <span className="mono seq">{record.seq}</span>
                <span className="type">{record.payload.type}</span>
                <span className="actor">{record.actor}</span>
                <span className="when hint">{timeAgo(record.at)}</span>
                <code className="hash" title={record.chain_hash}>
                  {shortHash(record.chain_hash)}
                </code>
                <button
                  type="button"
                  className="linkish"
                  onClick={() => void peekAt(record.seq)}
                >
                  view
                </button>
              </li>
            ))}
            {records.length === 0 && <li className="hint">No events yet.</li>}
          </ol>
          {peek && (
            <div className="peek">
              <h3 className="mono">As of event {peek.seq}</h3>
              <pre>{peek.text}</pre>
              <button
                type="button"
                className="linkish"
                onClick={() => setPeek(undefined)}
              >
                close
              </button>
            </div>
          )}
        </div>
        <div>
          <h2>Drafts</h2>
          <p className="lede">
            A draft is a personal branch of this page. Work in it, then merge
            three-way. Disjoint edits combine; overlapping wording asks you to
            choose. The preview also shows continuity marks the merge would
            introduce.
          </p>
          <ul className="plain-list">
            {(drafts.data ?? []).map((draft) => (
              <li key={draft.id} className="draft-row">
                <div className="row">
                  <strong>{draft.name}</strong>
                  <small className="mono">
                    from seq {draft.base_seq} · {timeAgo(draft.updated_at)}
                    {draft.merged_at ? " · merged" : ""}
                  </small>
                </div>
                {!draft.merged_at && (
                  <div className="row">
                    <button
                      type="button"
                      className="btn btn--small"
                      onClick={() =>
                        setEditingDraft(
                          editingDraft === draft.id ? undefined : draft.id,
                        )
                      }
                    >
                      {editingDraft === draft.id
                        ? "Close draft"
                        : "Write in draft"}
                    </button>
                    <button
                      type="button"
                      className="btn btn--small"
                      onClick={() => mergePreview.mutate(draft.id)}
                      disabled={mergePreview.isPending}
                    >
                      Preview merge
                    </button>
                    <button
                      type="button"
                      className="linkish rubric"
                      onClick={() => deleteDraft.mutate(draft.id)}
                    >
                      discard
                    </button>
                  </div>
                )}
                {preview?.draft_id === draft.id && (
                  <div className="merge-preview">
                    <p className="hint">
                      {preview.ops} change{preview.ops === 1 ? "" : "s"} ·{" "}
                      {preview.conflicts.length} conflict
                      {preview.conflicts.length === 1 ? "" : "s"} ·{" "}
                      {preview.new_marks.length} new mark
                      {preview.new_marks.length === 1 ? "" : "s"} after merging
                      · {preview.resolved_marks.length} resolved
                    </p>
                    {preview.conflicts.map((conflict) => (
                      <fieldset key={conflict.key} className="conflict">
                        <legend className="mono">
                          {conflict.field === "content"
                            ? "overlapping wording"
                            : conflict.field}
                        </legend>
                        <label>
                          <input
                            type="radio"
                            name={conflict.key}
                            checked={
                              (resolutions[conflict.key] ?? "ours") === "ours"
                            }
                            onChange={() =>
                              setResolutions({
                                ...resolutions,
                                [conflict.key]: "ours",
                              })
                            }
                          />
                          <span>
                            Keep the page: {plain(conflict.ours).slice(0, 140)}
                          </span>
                        </label>
                        <label>
                          <input
                            type="radio"
                            name={conflict.key}
                            checked={resolutions[conflict.key] === "theirs"}
                            onChange={() =>
                              setResolutions({
                                ...resolutions,
                                [conflict.key]: "theirs",
                              })
                            }
                          />
                          <span>
                            Take the draft:{" "}
                            {plain(conflict.theirs).slice(0, 140)}
                          </span>
                        </label>
                      </fieldset>
                    ))}
                    {preview.new_marks.length > 0 && (
                      <ul className="plain-list">
                        {preview.new_marks.map((mark) => (
                          <li key={mark.id}>
                            <code className="chip chip--code">{mark.code}</code>{" "}
                            {mark.title}
                          </li>
                        ))}
                      </ul>
                    )}
                    <div className="row">
                      {preview.conflicts.length > 0 && (
                        <button
                          type="button"
                          className="btn btn--small"
                          onClick={() => mergePreview.mutate(draft.id)}
                        >
                          Re-check with my choices
                        </button>
                      )}
                      <button
                        type="button"
                        className="btn btn--small btn--primary"
                        onClick={() => merge.mutate(draft.id)}
                        disabled={
                          merge.isPending ||
                          preview.conflicts.some(
                            (conflict) => !resolutions[conflict.key] && false,
                          )
                        }
                      >
                        Merge into the page
                      </button>
                    </div>
                    {merge.error && (
                      <p className="error">{merge.error.message}</p>
                    )}
                  </div>
                )}
              </li>
            ))}
          </ul>
          <form
            className="row"
            onSubmit={(event) => {
              event.preventDefault();
              const form = new FormData(event.currentTarget);
              createDraft.mutate(String(form.get("name") ?? "A draft"));
              event.currentTarget.reset();
            }}
          >
            <input name="name" placeholder="Try a stronger ending" required />
            <button
              type="submit"
              className="btn btn--small"
              disabled={createDraft.isPending}
            >
              New draft
            </button>
          </form>
        </div>
      </div>

      {draftBeingEdited && (
        <div className="draft-editor">
          <ManuscriptEditor
            key={draftBeingEdited.id}
            projectId={projectId}
            document={{
              ...document,
              content_json: draftBeingEdited.content_json,
              title: `${document.title} — ${draftBeingEdited.name}`,
              head_seq: 0,
            }}
            diagnostics={[]}
            entities={[]}
            pageView={false}
            trackChanges={false}
            findQuery=""
            showBlame={false}
            selectedMarkId={undefined}
            saving={updateDraft.isPending}
            onSave={async (content_json) => {
              await updateDraft.mutateAsync({
                id: draftBeingEdited.id,
                content_json,
              });
              return { ...document, content_json, head_seq: 0 };
            }}
            onSelectMark={() => undefined}
            onSelectEntity={() => undefined}
            readOnlyLabel="draft"
          />
        </div>
      )}
    </section>
  );
}

function plain(value: unknown): string {
  if (!value) return "";
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.map(plain).join("\n");
  if (typeof value === "object") {
    const node = value as {
      text?: string;
      content?: unknown;
      current_fields?: { content?: unknown };
    };
    if (node.current_fields?.content) return plain(node.current_fields.content);
    return `${node.text ?? ""}${plain(node.content)}`;
  }
  return "";
}
