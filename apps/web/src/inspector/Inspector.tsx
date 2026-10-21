import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useState } from "react";
import type {
  AstenDiagnostic,
  DocumentSummary,
  Location,
} from "@asten/contracts";
import { createAstenClient } from "../lib/asten-client";
import { percent, severityWord, timeAgo } from "../lib/format";

interface Props {
  projectId: string;
  document?: DocumentSummary;
  documents: DocumentSummary[];
  entityKey?: string;
  marks: AstenDiagnostic[];
  selectedMarkId?: string;
  onJump: (location: Location) => void;
  onOpenEntity: (key: string) => void;
  onSelectMark: (id: string | undefined) => void;
}

type Tab = "mark" | "entity" | "page";

/** The right-hand pane: the selected mark, the hovered entity, and the page's notes. */
export function Inspector({
  projectId,
  document,
  documents,
  entityKey,
  marks,
  selectedMarkId,
  onJump,
  onOpenEntity,
  onSelectMark,
}: Props) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<Tab | undefined>();
  const mark = marks.find((item) => item.id === selectedMarkId);
  const active: Tab = tab ?? (mark ? "mark" : entityKey ? "entity" : "page");

  const card = useQuery({
    queryKey: ["fact-card", projectId, entityKey, document?.id],
    queryFn: () => client.factCard(projectId, entityKey!, document?.id),
    enabled: Boolean(entityKey) && active === "entity",
  });
  const comments = useQuery({
    queryKey: ["comments", projectId, document?.id],
    queryFn: () => client.listComments(projectId, document!.id),
    enabled: Boolean(document) && active === "page",
  });
  const snapshots = useQuery({
    queryKey: ["snapshots", projectId, document?.id],
    queryFn: () => client.listSnapshots(projectId, document!.id),
    enabled: Boolean(document) && active === "page",
  });
  const createComment = useMutation({
    mutationFn: (body: string) =>
      client.createComment(projectId, document!.id, { body }),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: ["comments", projectId, document?.id],
      }),
  });
  const resolveComment = useMutation({
    mutationFn: (input: { id: string; resolved: boolean }) =>
      client.updateComment(projectId, input.id, { resolved: input.resolved }),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: ["comments", projectId, document?.id],
      }),
  });
  const createSnapshot = useMutation({
    mutationFn: (label: string) =>
      client.createSnapshot(projectId, document!.id, label || undefined),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: ["snapshots", projectId, document?.id],
      }),
  });
  const restoreSnapshot = useMutation({
    mutationFn: (id: string) => client.restoreSnapshot(projectId, id),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["documents", projectId] }),
  });

  function submitComment(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    createComment.mutate(String(form.get("body") ?? ""));
    event.currentTarget.reset();
  }

  const docTitle = (id: string) =>
    documents.find((item) => item.id === id)?.title ?? "a page";

  return (
    <div className="inspector-body">
      <div className="segmented small" role="tablist">
        <button
          type="button"
          role="tab"
          aria-selected={active === "mark"}
          className={active === "mark" ? "active" : ""}
          onClick={() => setTab("mark")}
          disabled={!mark}
        >
          Mark
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={active === "entity"}
          className={active === "entity" ? "active" : ""}
          onClick={() => setTab("entity")}
          disabled={!entityKey}
        >
          Entity
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={active === "page"}
          className={active === "page" ? "active" : ""}
          onClick={() => setTab("page")}
          disabled={!document}
        >
          Page
        </button>
      </div>

      {active === "mark" && mark && (
        <section className="inspector-section">
          <header className="mark-head">
            <code className="chip chip--code">{mark.code}</code>
            <span className="mono">
              {severityWord(mark.severity)} · {mark.tier}
              {mark.probability < 1 ? ` · p ${percent(mark.probability)}` : ""}
            </span>
          </header>
          <h3>{mark.title}</h3>
          <ol className="explanation">
            {mark.explanation.map((statement, index) => (
              <li key={`${statement.id}-${index}`}>
                {statement.location ? (
                  <button
                    type="button"
                    className="linkish"
                    onClick={() =>
                      statement.location && onJump(statement.location)
                    }
                  >
                    {statement.text}
                  </button>
                ) : (
                  statement.text
                )}
              </li>
            ))}
          </ol>
          {mark.related.length > 0 && (
            <p className="hint">
              Also at{" "}
              {mark.related.map(([location, role], index) => (
                <button
                  key={`${location.document}-${index}`}
                  type="button"
                  className="linkish"
                  onClick={() => onJump(location)}
                >
                  {docTitle(location.document)} ({role})
                </button>
              ))}
            </p>
          )}
          {mark.entity && (
            <button
              type="button"
              className="linkish"
              onClick={() => onOpenEntity(mark.entity!)}
            >
              Open {mark.entity.replace(/_/g, " ")} in the cast
            </button>
          )}
          <p className="hint">
            Fixes are in The Scales below. The engine never edits your prose.
          </p>
          <button
            type="button"
            className="linkish"
            onClick={() => onSelectMark(undefined)}
          >
            Clear selection
          </button>
        </section>
      )}

      {active === "entity" && entityKey && (
        <section className="inspector-section">
          {card.data ? (
            <>
              <header className="mark-head">
                <span className="cartouche-chip">{card.data.name}</span>
                <span className="mono">{card.data.sort}</span>
              </header>
              {card.data.at_label && (
                <p className="hint">At {card.data.at_label}</p>
              )}
              <ul className="fact-list">
                {card.data.facts.map((fact) => (
                  <li key={fact.id} className={fact.status}>
                    <span className="mono">{fact.predicate}</span>
                    <strong>{fact.value}</strong>
                    <small>
                      {fact.status}
                      {fact.context !== "world" ? ` · ${fact.context}` : ""}
                      {fact.valid_label ? ` · ${fact.valid_label}` : ""}
                    </small>
                    {fact.settled_in && (
                      <button
                        type="button"
                        className="linkish"
                        onClick={() =>
                          client
                            .whereSettled(projectId, fact.id)
                            .then(
                              (where) =>
                                where.document &&
                                onJump({
                                  document: where.document,
                                  block: where.block,
                                  quote: "",
                                  label: "",
                                  start: where.start,
                                  end: where.end,
                                }),
                            )
                            .catch(() => undefined)
                        }
                      >
                        where settled
                      </button>
                    )}
                  </li>
                ))}
                {card.data.facts.length === 0 && (
                  <li className="hint">Nothing settled yet.</li>
                )}
              </ul>
              <p className="hint">
                {card.data.mention_count} mention
                {card.data.mention_count === 1 ? "" : "s"} ·{" "}
                {card.data.open_marks} open mark
                {card.data.open_marks === 1 ? "" : "s"}
              </p>
              <button
                type="button"
                className="btn btn--small"
                onClick={() => onOpenEntity(entityKey)}
              >
                Open in cast
              </button>
            </>
          ) : (
            <p className="status">{card.isError ? "Not in the cast." : "…"}</p>
          )}
        </section>
      )}

      {active === "page" && document && (
        <section className="inspector-section">
          <h3 className="mono">Comments</h3>
          <ul className="fact-list">
            {comments.data?.map((comment) => (
              <li
                key={comment.id}
                className={comment.resolved ? "resolved" : ""}
              >
                <span>{comment.body}</span>
                <small>{timeAgo(comment.created_at)}</small>
                <button
                  type="button"
                  className="linkish"
                  onClick={() =>
                    resolveComment.mutate({
                      id: comment.id,
                      resolved: !comment.resolved,
                    })
                  }
                >
                  {comment.resolved ? "reopen" : "resolve"}
                </button>
              </li>
            ))}
          </ul>
          <form className="stacked-form" onSubmit={submitComment}>
            <input name="body" placeholder="A note for later" required />
            <button
              type="submit"
              className="btn btn--small"
              disabled={createComment.isPending}
            >
              Leave a comment
            </button>
          </form>
          <h3 className="mono">Snapshots</h3>
          <form
            className="row"
            onSubmit={(event) => {
              event.preventDefault();
              const form = new FormData(event.currentTarget);
              createSnapshot.mutate(String(form.get("label") ?? ""));
              event.currentTarget.reset();
            }}
          >
            <input name="label" placeholder="Name this version" />
            <button
              type="submit"
              className="btn btn--small"
              disabled={createSnapshot.isPending}
            >
              Snapshot
            </button>
          </form>
          <ul className="fact-list">
            {snapshots.data?.map((snapshot) => (
              <li key={snapshot.id}>
                <span>
                  {snapshot.label ?? "Snapshot"}{" "}
                  <small className="mono">
                    r{snapshot.revision} · seq {snapshot.seq}
                  </small>
                </span>
                <small>{timeAgo(snapshot.created_at)}</small>
                <button
                  type="button"
                  className="linkish"
                  onClick={() => restoreSnapshot.mutate(snapshot.id)}
                >
                  Restore
                </button>
              </li>
            ))}
          </ul>
          <p className="hint mono">
            head {document.head_seq} · {document.chain_hash.slice(0, 10)}
          </p>
        </section>
      )}
    </div>
  );
}
