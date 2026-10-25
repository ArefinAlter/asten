import {
  useMutation,
  useQueries,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { useMemo, useState } from "react";
import type { DocumentSnapshot } from "@asten/contracts";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import { log } from "../output";
import {
  commitsOf,
  diffParagraphs,
  diffStats,
  pageStatus,
  paragraphsOf,
  type PageStatus,
} from "../scm";
import { relativeTime } from "../util";

const TRACKED = new Set(["scene", "chapter", "wiki", "note", "source"]);

function useSnapshots() {
  const api = useWorkbench();
  const pages = api.documents.filter((d) => TRACKED.has(d.kind));
  const results = useQueries({
    queries: pages.map((d) => ({
      queryKey: ["snapshots", api.projectId, d.id],
      queryFn: () => api.client.listSnapshots(api.projectId!, d.id),
      enabled: Boolean(api.projectId),
    })),
  });
  const byDoc = new Map<string, DocumentSnapshot[]>();
  pages.forEach((d, i) => byDoc.set(d.id, results[i]?.data ?? []));
  const loading = results.some((r) => r.isPending);
  return { pages, byDoc, loading };
}

/** Source Control, VS Code style, over Asten's snapshots and drafts. */
export function SourceControlPane() {
  const api = useWorkbench();
  const queryClient = useQueryClient();
  const { pages, byDoc, loading } = useSnapshots();
  const [message, setMessage] = useState("");
  const [staged, setStaged] = useState<Set<string> | null>(null);
  const statuses = useMemo(
    () => pages.map((d) => pageStatus(d, byDoc.get(d.id) ?? [])),
    [pages, byDoc],
  );
  const changes = statuses.filter((s) => s.state !== "clean");
  const chosen = staged ?? new Set(changes.map((c) => c.document.id));
  const commits = useMemo(() => commitsOf([...byDoc.values()].flat()), [byDoc]);
  const drafts = useQuery({
    queryKey: ["drafts", api.projectId, api.openDocumentId],
    queryFn: () => api.client.listDrafts(api.projectId!, api.openDocumentId!),
    enabled: Boolean(api.projectId && api.openDocumentId),
  });
  const commit = useMutation({
    mutationFn: async () => {
      const ids = changes
        .filter((c) => chosen.has(c.document.id))
        .map((c) => c.document.id);
      for (const id of ids)
        await api.client.createSnapshot(api.projectId!, id, message.trim());
      return ids.length;
    },
    onSuccess: async (count) => {
      log(
        "Versions",
        `Committed ${count} page${count === 1 ? "" : "s"}: ${message.trim()}`,
      );
      setMessage("");
      setStaged(null);
      await queryClient.invalidateQueries({ queryKey: ["snapshots"] });
    },
    onError: (error) => api.toast(error.message, "error"),
  });
  const newDraft = useMutation({
    mutationFn: (name: string) =>
      api.client.createDraft(api.projectId!, api.openDocumentId!, name),
    onSuccess: async (draft) => {
      log(
        "Versions",
        `Draft branch “${draft.name}” created from the open page.`,
      );
      await queryClient.invalidateQueries({ queryKey: ["drafts"] });
    },
  });

  if (!api.projectId)
    return <p className="pane-empty">Open a project to see its versions.</p>;
  const openTitle = api.documents.find(
    (d) => d.id === api.openDocumentId,
  )?.title;

  return (
    <div className="scm-pane">
      <div className="scm-branch">
        <Icon name="branch" />
        <span>
          {openTitle ? (
            <>
              main · <small>{openTitle}</small>
            </>
          ) : (
            "main"
          )}
        </span>
      </div>
      <textarea
        className="scm-message"
        placeholder={`Message (Ctrl+Enter to commit ${chosen.size} page${chosen.size === 1 ? "" : "s"})`}
        value={message}
        rows={2}
        onChange={(e) => setMessage(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && e.ctrlKey && message.trim() && chosen.size)
            commit.mutate();
        }}
        aria-label="Commit message"
      />
      <button
        type="button"
        className="wb-btn wb-btn--primary wb-btn--block"
        disabled={!message.trim() || !chosen.size || commit.isPending}
        onClick={() => commit.mutate()}
      >
        <Icon name="check" /> Commit{" "}
        {chosen.size
          ? `${chosen.size} page${chosen.size === 1 ? "" : "s"}`
          : ""}
      </button>
      <div className="scm-section">
        <div className="scm-section-head">
          Changes <span className="wb-count">{changes.length}</span>
        </div>
        {loading && <p className="pane-empty">Reading versions…</p>}
        {!loading && !changes.length && (
          <p className="pane-empty">
            No page has changed since its last version.
          </p>
        )}
        {changes.map((status) => (
          <ChangeRow
            key={status.document.id}
            status={status}
            checked={chosen.has(status.document.id)}
            onCheck={(on) => {
              const next = new Set(chosen);
              if (on) next.add(status.document.id);
              else next.delete(status.document.id);
              setStaged(next);
            }}
          />
        ))}
      </div>
      <div className="scm-section">
        <div className="scm-section-head">
          Drafts of this page{" "}
          <span className="wb-count">{drafts.data?.length ?? 0}</span>
          <button
            type="button"
            className="wb-icon-btn"
            title="New draft branch"
            aria-label="New draft branch"
            disabled={!api.openDocumentId}
            onClick={async () => {
              const name = await api.ask({
                title: "New draft branch",
                label: "Name",
                placeholder: "alternate-ending",
              });
              if (name) newDraft.mutate(name);
            }}
          >
            <Icon name="plus" />
          </button>
        </div>
        {!api.openDocumentId && (
          <p className="pane-empty">Open a page to see its drafts.</p>
        )}
        {drafts.data?.map((draft) => (
          <button
            key={draft.id}
            type="button"
            className="scm-row"
            onClick={() => api.setView("history")}
            title="Open in History to compare and merge"
          >
            <Icon name="branch" />
            <span className="scm-row-main">{draft.name}</span>
            <small className="muted">
              {draft.merged_at ? "merged" : relativeTime(draft.updated_at)}
            </small>
          </button>
        ))}
      </div>
      <div className="scm-section">
        <div className="scm-section-head">
          History <span className="wb-count">{commits.length}</span>
        </div>
        <ol className="scm-graph">
          {commits.map((c, index) => (
            <li key={c.id}>
              <span className={`graph-dot${index === 0 ? " head" : ""}`} />
              <button
                type="button"
                className="scm-commit"
                onClick={() =>
                  api.openDiff(c.snapshots[0].document_id, c.snapshots[0].id)
                }
              >
                <span>{c.message}</span>
                <small className="muted">
                  {c.snapshots.length} page{c.snapshots.length === 1 ? "" : "s"}{" "}
                  · {relativeTime(c.at)}
                </small>
              </button>
            </li>
          ))}
        </ol>
      </div>
    </div>
  );
}

function ChangeRow({
  status,
  checked,
  onCheck,
}: {
  status: PageStatus;
  checked: boolean;
  onCheck: (on: boolean) => void;
}) {
  const api = useWorkbench();
  return (
    <div
      className={`scm-row${api.diffTarget?.documentId === status.document.id ? " selected" : ""}`}
    >
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onCheck(e.target.checked)}
        aria-label={`Include ${status.document.title}`}
      />
      <button
        type="button"
        className="scm-row-main"
        onClick={() => api.openDiff(status.document.id)}
      >
        <Icon name="file" size={14} /> {status.document.title}
      </button>
      <span className={`scm-state scm-${status.state}`} title={status.state}>
        {status.state === "modified" ? "M" : "U"}
      </span>
    </div>
  );
}

/** The diff of a page against a version: paragraphs added, removed and changed, words marked. */
export function DiffView() {
  const api = useWorkbench();
  const target = api.diffTarget;
  const documentId = target?.documentId;
  const doc = useQuery({
    queryKey: ["document", api.projectId, documentId],
    queryFn: () => api.client.getDocument(api.projectId!, documentId!),
    enabled: Boolean(api.projectId && documentId),
  });
  const snapshots = useQuery({
    queryKey: ["snapshots", api.projectId, documentId],
    queryFn: () => api.client.listSnapshots(api.projectId!, documentId!),
    enabled: Boolean(api.projectId && documentId),
  });
  const sorted = [...(snapshots.data ?? [])].sort((a, b) =>
    b.created_at.localeCompare(a.created_at),
  );
  const [chosen, setChosen] = useState<string | undefined>(target?.snapshotId);
  const base =
    sorted.find((s) => s.id === (chosen ?? target?.snapshotId)) ?? sorted[0];
  const restore = useMutation({
    mutationFn: (id: string) => api.client.restoreSnapshot(api.projectId!, id),
    onSuccess: () => {
      log(
        "Versions",
        `Restored “${doc.data?.title}” to “${base?.label ?? "a version"}”.`,
      );
      api.toast(
        "Restored. The restore is itself a new entry in the page's history.",
      );
    },
  });
  if (!documentId)
    return <p className="status">Choose a change in Source Control.</p>;
  if (!doc.data) return <p className="status">Opening…</p>;
  const changes = diffParagraphs(
    base ? paragraphsOf(base.content_json) : [],
    paragraphsOf(doc.data.content_json),
  );
  const stats = diffStats(changes);
  return (
    <section className="diff-view">
      <header>
        <h1>{doc.data.title}</h1>
        <div className="diff-meta">
          <label>
            Compare with{" "}
            <select
              value={base?.id ?? ""}
              onChange={(e) => setChosen(e.target.value)}
            >
              {sorted.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.label ?? "Untitled"} · {relativeTime(s.created_at)}
                </option>
              ))}
              {!sorted.length && <option value="">No earlier version</option>}
            </select>
          </label>
          <span className="diff-stat add">+{stats.added}</span>
          <span className="diff-stat del">−{stats.removed}</span>
          <button
            type="button"
            className="wb-btn wb-btn--small"
            onClick={() => api.openDocument(documentId)}
          >
            Open the page
          </button>
          {base && (
            <button
              type="button"
              className="wb-btn wb-btn--small"
              disabled={restore.isPending}
              onClick={() => restore.mutate(base.id)}
            >
              Restore this version
            </button>
          )}
        </div>
      </header>
      <div className="diff-body">
        {changes.map((change, index) => (
          <div key={index} className={`diff-para diff-${change.kind}`}>
            <span className="diff-gutter">
              {change.kind === "add"
                ? "+"
                : change.kind === "del"
                  ? "−"
                  : change.kind === "change"
                    ? "±"
                    : ""}
            </span>
            <p>
              {change.kind === "change"
                ? change.words!.map((w, i) => (
                    <span
                      key={i}
                      className={
                        w.op === "add"
                          ? "w-add"
                          : w.op === "del"
                            ? "w-del"
                            : undefined
                      }
                    >
                      {w.value}
                    </span>
                  ))
                : (change.after ?? change.before)}
            </p>
          </div>
        ))}
      </div>
    </section>
  );
}
