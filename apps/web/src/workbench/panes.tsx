import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { useWorkbench } from "./context";
import { Icon } from "./icons";
import { log } from "./output";
import { countBySeverity, problemRows } from "./problems";

interface Block {
  type?: string;
  text?: string;
  attrs?: { id?: string };
  content?: Block[];
}

const textOf = (node: Block): string =>
  node.type === "text"
    ? (node.text ?? "")
    : (node.content ?? []).map(textOf).join("");

/** Search across every page, Word's Navigation pane results and VS Code's Search in one. */
export function SearchPane() {
  const api = useWorkbench();
  const [query, setQuery] = useState("");
  const [matchCase, setMatchCase] = useState(false);
  const documents = useQuery({
    queryKey: ["documents-full", api.projectId],
    queryFn: () => api.client.listDocuments(api.projectId!),
    enabled: Boolean(api.projectId),
    staleTime: 5000,
  });
  const results = useMemo(() => {
    const q = matchCase ? query.trim() : query.trim().toLowerCase();
    if (q.length < 2 || !documents.data) return [];
    const out: {
      documentId: string;
      title: string;
      hits: {
        block: string;
        start: number;
        before: string;
        match: string;
        after: string;
      }[];
    }[] = [];
    for (const document of documents.data) {
      const hits = [];
      for (const block of (document.content_json as Block)?.content ?? []) {
        const text = textOf(block);
        const hay = matchCase ? text : text.toLowerCase();
        let at = hay.indexOf(q);
        while (at >= 0 && hits.length < 20) {
          hits.push({
            block: block.attrs?.id ?? "",
            start: at,
            before: text.slice(Math.max(0, at - 30), at),
            match: text.slice(at, at + q.length),
            after: text.slice(at + q.length, at + q.length + 40),
          });
          at = hay.indexOf(q, at + q.length);
        }
      }
      if (
        hits.length ||
        (matchCase ? document.title : document.title.toLowerCase()).includes(q)
      )
        out.push({ documentId: document.id, title: document.title, hits });
    }
    return out;
  }, [query, matchCase, documents.data]);
  const total = results.reduce((n, r) => n + r.hits.length, 0);
  if (!api.projectId)
    return <p className="pane-empty">Open a project to search it.</p>;
  return (
    <div className="search-pane">
      <div className="search-box">
        <input
          placeholder="Search the project"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          aria-label="Search the project"
          autoFocus
        />
        <button
          type="button"
          className={`wb-icon-btn${matchCase ? " active" : ""}`}
          title="Match case"
          aria-pressed={matchCase}
          onClick={() => setMatchCase((v) => !v)}
        >
          Aa
        </button>
      </div>
      {query.trim().length >= 2 && (
        <p className="muted small search-count">
          {total} result{total === 1 ? "" : "s"} in {results.length} page
          {results.length === 1 ? "" : "s"}
        </p>
      )}
      {results.map((result) => (
        <div key={result.documentId} className="search-group">
          <button
            type="button"
            className="search-file"
            onClick={() => api.openDocument(result.documentId)}
          >
            <Icon name="file" size={14} /> {result.title}{" "}
            <span className="wb-count">{result.hits.length}</span>
          </button>
          {result.hits.map((hit, i) => (
            <button
              key={i}
              type="button"
              className="search-hit"
              onClick={() =>
                api.jumpTo({
                  document: result.documentId,
                  block: hit.block,
                  start: hit.start,
                  end: hit.start + hit.match.length,
                  quote: hit.match,
                  label: result.title,
                })
              }
            >
              {hit.before}
              <mark>{hit.match}</mark>
              {hit.after}
            </button>
          ))}
        </div>
      ))}
    </div>
  );
}

/** Checks at a glance: counts by severity and family, packs, and the shortcuts to act. */
export function ChecksPane() {
  const api = useWorkbench();
  const rows = problemRows(api.diagnostics?.diagnostics ?? [], api.documents);
  const counts = countBySeverity(rows);
  const families = new Map<string, number>();
  for (const row of rows)
    families.set(
      row.code.split("-")[0],
      (families.get(row.code.split("-")[0]) ?? 0) + 1,
    );
  if (!api.projectId)
    return <p className="pane-empty">Open a project to weigh it.</p>;
  return (
    <div className="checks-pane">
      <div className="checks-summary">
        <div>
          <strong className="sev-error-fg">{counts.error}</strong>
          <small>errors</small>
        </div>
        <div>
          <strong className="sev-warning-fg">{counts.warning}</strong>
          <small>warnings</small>
        </div>
        <div>
          <strong>{counts.information}</strong>
          <small>information</small>
        </div>
      </div>
      <button
        type="button"
        className="wb-btn wb-btn--primary wb-btn--block"
        onClick={() => api.run("app.runChecks")}
      >
        <Icon name="check" /> Run checks <kbd>F7</kbd>
      </button>
      <div className="pane-section">Families</div>
      {[...families].map(([family, n]) => (
        <button
          key={family}
          type="button"
          className="pane-row"
          onClick={() => api.run("panel.problems")}
        >
          <code>{family}</code>
          <span className="spacer" />
          <span className="wb-count">{n}</span>
        </button>
      ))}
      <div className="pane-section">Marks</div>
      {rows.slice(0, 12).map((row) => (
        <button
          key={row.id}
          type="button"
          className={`pane-row${api.selectedMarkId === row.id ? " selected" : ""}`}
          onClick={() => {
            api.selectMark(row.id);
            if (row.diagnostic.primary) api.jumpTo(row.diagnostic.primary);
          }}
        >
          <span className={`sev-dot sev-${row.severity}`} />
          <span className="pane-row-main">
            {row.title}
            <small>{row.scene}</small>
          </span>
        </button>
      ))}
      <div className="pane-links">
        <button
          type="button"
          className="linkish"
          onClick={() => api.setView("canon")}
        >
          Canon
        </button>
        <button
          type="button"
          className="linkish"
          onClick={() => api.openSettings("checks")}
        >
          Strictness and clock
        </button>
      </div>
    </div>
  );
}

export function PacksPane() {
  const api = useWorkbench();
  const queryClient = useQueryClient();
  const packs = useQuery({
    queryKey: ["packs"],
    queryFn: () => api.client.listPacks(),
  });
  const toggle = useMutation({
    mutationFn: (input: { id: string; enabled: boolean }) =>
      api.client.updatePack(input.id, input.enabled),
    onSuccess: async (pack) => {
      log("Packs", `${pack.name} ${pack.enabled ? "enabled" : "disabled"}.`);
      await queryClient.invalidateQueries({ queryKey: ["packs"] });
      await queryClient.invalidateQueries({ queryKey: ["diagnostics"] });
    },
    onError: (error) => api.toast(error.message, "error"),
  });
  return (
    <div className="packs-pane">
      {(packs.data ?? []).map((pack) => (
        <div key={pack.id} className="pane-row static">
          <span className="pane-row-main">
            {pack.name}
            <small>{pack.summary}</small>
          </span>
          <button
            type="button"
            role="switch"
            aria-checked={pack.enabled}
            aria-label={pack.name}
            className={`wb-switch${pack.enabled ? " on" : ""}`}
            disabled={pack.pack_key === "scribe" || toggle.isPending}
            title={pack.pack_key === "scribe" ? "Scribe stays on" : undefined}
            onClick={() =>
              toggle.mutate({ id: pack.id, enabled: !pack.enabled })
            }
          >
            <span />
          </button>
        </div>
      ))}
      <div className="pane-links">
        <button
          type="button"
          className="linkish"
          onClick={() => api.setView("packs")}
        >
          Open packs
        </button>
      </div>
    </div>
  );
}
