import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import type {
  AstenDiagnostic,
  AstenDocument,
  DiagnosticsSnapshot,
  Location,
  Severity,
} from "@asten/contracts";
import { Scales } from "../brand/Scales";
import { createAstenClient } from "../lib/asten-client";
import { percent, severityWord } from "../lib/format";

type Filter =
  "all" | "error" | "warning" | "information" | "hint" | "new" | "suppressed";

interface Props {
  projectId: string;
  snapshot?: DiagnosticsSnapshot;
  loading: boolean;
  documents: AstenDocument[];
  selectedMarkId?: string;
  onSelectMark: (id: string | undefined) => void;
  onJump: (location: Location) => void;
  collapsed: boolean;
  onToggleCollapsed: () => void;
}

const ORDER: Severity[] = ["error", "warning", "information", "hint"];

/**
 * The Problems panel is "The Scales": every mark with its evidence, the most
 * likely slip, and the fixes the author owns. Its empty state is a balanced
 * scale: nothing disagrees with your canon.
 */
export function ScalesPanel({
  projectId,
  snapshot,
  loading,
  documents,
  selectedMarkId,
  onSelectMark,
  onJump,
  collapsed,
  onToggleCollapsed,
}: Props) {
  const client = createAstenClient();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<Filter>("all");
  const [reasonFor, setReasonFor] = useState<string>();
  const [reason, setReason] = useState("");
  const [bisecting, setBisecting] = useState<Record<string, string>>({});

  const marks = useMemo(() => snapshot?.diagnostics ?? [], [snapshot]);
  const visible = useMemo(() => {
    const unsuppressed = marks.filter((mark) => !mark.suppressed_by);
    switch (filter) {
      case "all":
        return unsuppressed;
      case "new":
        return unsuppressed.filter((mark) => mark.new_since_baseline);
      case "suppressed":
        return marks.filter((mark) => mark.suppressed_by);
      default:
        return unsuppressed.filter((mark) => mark.severity === filter);
    }
  }, [marks, filter]);

  const counts = useMemo(() => {
    const out: Record<string, number> = {
      all: 0,
      error: 0,
      warning: 0,
      information: 0,
      hint: 0,
      new: 0,
      suppressed: 0,
    };
    for (const mark of marks) {
      if (mark.suppressed_by) {
        out.suppressed += 1;
        continue;
      }
      out.all += 1;
      out[mark.severity] += 1;
      if (mark.new_since_baseline) out.new += 1;
    }
    return out;
  }, [marks]);

  const applyRepair = useMutation({
    mutationFn: (input: {
      diagnostic_id: string;
      repair_index: number;
      reason?: string;
    }) => client.applyRepair(projectId, input),
    onSuccess: async () => {
      setReasonFor(undefined);
      setReason("");
      await queryClient.invalidateQueries({
        queryKey: ["diagnostics", projectId],
      });
      await queryClient.invalidateQueries({
        queryKey: ["documents", projectId],
      });
      await queryClient.invalidateQueries({ queryKey: ["facts", projectId] });
      await queryClient.invalidateQueries({ queryKey: ["events", projectId] });
      await queryClient.invalidateQueries({
        queryKey: ["entities", projectId],
      });
    },
  });

  async function bisect(mark: AstenDiagnostic) {
    setBisecting((current) => ({
      ...current,
      [mark.id]: "Searching the history…",
    }));
    try {
      const result = await client.bisect(projectId, mark.id);
      const when = result.at ? new Date(result.at).toLocaleString() : "";
      setBisecting((current) => ({
        ...current,
        [mark.id]: result.found
          ? `Introduced by ${result.actor}'s edit (${result.event_type}) at event ${result.seq}${when ? `, ${when}` : ""} after ${result.probes} probes.`
          : "Present since the document was created.",
      }));
    } catch (error) {
      setBisecting((current) => ({
        ...current,
        [mark.id]: error instanceof Error ? error.message : "Could not search.",
      }));
    }
  }

  const docTitle = (id: string) =>
    documents.find((document) => document.id === id)?.title ?? "a page";
  const tilt = counts.error > 0 ? -7 : counts.warning > 0 ? -3 : 0;

  return (
    <section
      className={`scales-panel${collapsed ? " collapsed" : ""}`}
      aria-label="The Scales"
    >
      <header className="pane-heading">
        <button
          type="button"
          className="pane-title"
          onClick={onToggleCollapsed}
          aria-expanded={!collapsed}
        >
          <span className="mono">The Scales</span>
          <span className="pane-count">{counts.all}</span>
        </button>
        {!collapsed && (
          <div className="filters" role="tablist" aria-label="Filter marks">
            {(
              [
                "all",
                "error",
                "warning",
                "information",
                "hint",
                "new",
                "suppressed",
              ] as Filter[]
            ).map((item) => (
              <button
                key={item}
                type="button"
                role="tab"
                aria-selected={filter === item}
                className={`filter filter-${item}${filter === item ? " active" : ""}`}
                onClick={() => setFilter(item)}
                disabled={item !== "all" && counts[item] === 0}
              >
                {item === "all"
                  ? "All"
                  : item === "new"
                    ? "New since baseline"
                    : item === "suppressed"
                      ? "Silenced"
                      : `${severityWord(item)}s`}
                <small>{counts[item]}</small>
              </button>
            ))}
            {snapshot && (
              <span
                className="timing mono"
                title={`${snapshot.cached_blocks}/${snapshot.total_blocks} blocks from cache`}
              >
                {snapshot.timings_ms.total} ms
              </span>
            )}
          </div>
        )}
      </header>
      {!collapsed && (
        <div className="scales-body">
          {loading && marks.length === 0 ? (
            <p className="status">Weighing the pages…</p>
          ) : visible.length === 0 ? (
            <div className="scales-empty">
              <Scales tilt={filter === "all" ? 0 : tilt} />
              <p>
                {filter === "all"
                  ? "Nothing disagrees with your canon."
                  : "Nothing here."}
              </p>
              {snapshot?.ledger.length ? (
                <p className="hint">
                  {snapshot.ledger.length} setup
                  {snapshot.ledger.length === 1 ? "" : "s"} still waiting for a
                  payoff.
                </p>
              ) : null}
            </div>
          ) : (
            <ul className="mark-list">
              {ORDER.filter((severity) =>
                visible.some((mark) => mark.severity === severity),
              ).map((severity) => (
                <li key={severity} className="mark-group">
                  <h3 className="mono">{severityWord(severity)}s</h3>
                  <ul>
                    {visible
                      .filter((mark) => mark.severity === severity)
                      .map((mark) => {
                        const selected = selectedMarkId === mark.id;
                        return (
                          <li
                            key={mark.id}
                            className={`mark-item mark-${mark.severity}${selected ? " selected" : ""}${mark.suppressed_by ? " silenced" : ""}`}
                          >
                            <button
                              type="button"
                              className="mark-row"
                              onClick={() => {
                                onSelectMark(selected ? undefined : mark.id);
                                if (mark.primary) onJump(mark.primary);
                              }}
                              aria-expanded={selected}
                            >
                              <code className="chip chip--code">
                                {mark.code}
                              </code>
                              <span className="mark-title">{mark.title}</span>
                              <span className="mark-where mono">
                                {mark.primary
                                  ? docTitle(mark.primary.document)
                                  : ""}
                                {mark.tier === "may" ? " · may" : ""}
                                {mark.new_since_baseline &&
                                counts.new !== counts.all
                                  ? " · new"
                                  : ""}
                                {mark.probability < 1
                                  ? ` · p ${percent(mark.probability)}`
                                  : ""}
                              </span>
                            </button>
                            {selected && (
                              <div className="mark-detail">
                                <ol className="explanation">
                                  {mark.explanation.map((statement, index) => {
                                    const culprit = mark.culprit.find(
                                      (item) =>
                                        item.statement_id === statement.id,
                                    );
                                    const blamed =
                                      mark.culprit[0]?.statement_id ===
                                        statement.id &&
                                      (mark.culprit[0]?.probability ?? 0) >
                                        0.5 &&
                                      mark.explanation.length > 1;
                                    return (
                                      <li
                                        key={`${statement.id}-${index}`}
                                        className={blamed ? "blamed" : ""}
                                      >
                                        {statement.location ? (
                                          <button
                                            type="button"
                                            className="linkish"
                                            onClick={() =>
                                              statement.location &&
                                              onJump(statement.location)
                                            }
                                          >
                                            {statement.text}
                                          </button>
                                        ) : (
                                          <span>{statement.text}</span>
                                        )}
                                        {blamed && culprit ? (
                                          <small className="rubric">
                                            {" "}
                                            most likely slip (
                                            {percent(culprit.probability)})
                                          </small>
                                        ) : null}
                                      </li>
                                    );
                                  })}
                                </ol>
                                {mark.details && (
                                  <p className="hint">{mark.details}</p>
                                )}
                                <div className="repairs">
                                  {mark.repairs.map((repair, index) => (
                                    <button
                                      key={`${repair.label}-${index}`}
                                      type="button"
                                      className={`btn btn--small${repair.kind.kind === "keep" ? " btn--ghost" : ""}`}
                                      disabled={
                                        applyRepair.isPending ||
                                        Boolean(mark.suppressed_by)
                                      }
                                      onClick={() => {
                                        if (repair.kind.kind === "suppress") {
                                          setReasonFor(mark.id);
                                          return;
                                        }
                                        applyRepair.mutate({
                                          diagnostic_id: mark.id,
                                          repair_index: index,
                                        });
                                      }}
                                    >
                                      {repair.label}
                                    </button>
                                  ))}
                                  <button
                                    type="button"
                                    className="btn btn--small btn--ghost"
                                    onClick={() => void bisect(mark)}
                                  >
                                    Which edit introduced this?
                                  </button>
                                </div>
                                {reasonFor === mark.id && (
                                  <form
                                    className="reason-form"
                                    onSubmit={(event) => {
                                      event.preventDefault();
                                      const index = mark.repairs.findIndex(
                                        (repair) =>
                                          repair.kind.kind === "suppress",
                                      );
                                      applyRepair.mutate({
                                        diagnostic_id: mark.id,
                                        repair_index: index,
                                        reason,
                                      });
                                    }}
                                  >
                                    <input
                                      autoFocus
                                      placeholder="Why this is intentional"
                                      value={reason}
                                      onChange={(event) =>
                                        setReason(event.target.value)
                                      }
                                    />
                                    <button
                                      type="submit"
                                      className="btn btn--small"
                                    >
                                      Silence
                                    </button>
                                  </form>
                                )}
                                {bisecting[mark.id] && (
                                  <p className="hint">{bisecting[mark.id]}</p>
                                )}
                                {applyRepair.data &&
                                  applyRepair.variables?.diagnostic_id ===
                                    mark.id && (
                                    <p className="hint settled">
                                      {applyRepair.data.message}
                                    </p>
                                  )}
                                {applyRepair.error &&
                                  applyRepair.variables?.diagnostic_id ===
                                    mark.id && (
                                    <p className="error">
                                      {applyRepair.error.message}
                                    </p>
                                  )}
                                {mark.suppressed_by && (
                                  <p className="hint">
                                    Silenced. The mark returns if its evidence
                                    changes.
                                  </p>
                                )}
                              </div>
                            )}
                          </li>
                        );
                      })}
                  </ul>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </section>
  );
}
