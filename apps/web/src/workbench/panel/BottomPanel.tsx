import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import type { Severity } from "@asten/contracts";
import { ScalesPanel } from "../../scales/ScalesPanel";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import type { PanelTab } from "../layout";
import { clearOutput, log, useOutput, type Channel } from "../output";
import { countBySeverity, problemRows } from "../problems";
import { runCommand, type ConsoleLine } from "../terminal";
import { downloadArtifact, relativeTime } from "../util";

const TABS: { id: PanelTab; label: string }[] = [
  { id: "problems", label: "Problems" },
  { id: "scales", label: "Scales" },
  { id: "output", label: "Output" },
  { id: "terminal", label: "Console" },
  { id: "timeline", label: "Timeline" },
];

export function BottomPanel({
  onResizeStart,
}: {
  onResizeStart: (event: React.PointerEvent) => void;
}) {
  const api = useWorkbench();
  const { layout, dispatch } = api;
  const [scalesCollapsed, setScalesCollapsed] = useState(false);
  const open = (api.diagnostics?.diagnostics ?? []).filter(
    (d) => !d.suppressed_by,
  );
  return (
    <section className="bottom-panel" aria-label="Panel">
      <div
        className="panel-resizer"
        role="separator"
        aria-orientation="horizontal"
        aria-label="Resize the panel"
        onPointerDown={onResizeStart}
      />
      <div className="panel-tabs" role="tablist">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            type="button"
            role="tab"
            aria-selected={layout.panelTab === tab.id}
            className={layout.panelTab === tab.id ? "active" : ""}
            onClick={() => dispatch({ type: "showPanel", tab: tab.id })}
          >
            {tab.label}
            {tab.id === "problems" && open.length > 0 && (
              <span className="wb-count">{open.length}</span>
            )}
          </button>
        ))}
        <span className="spacer" />
        <button
          type="button"
          className="wb-icon-btn"
          title={
            layout.panelMaximized ? "Restore panel size" : "Maximise panel"
          }
          aria-label={
            layout.panelMaximized ? "Restore panel size" : "Maximise panel"
          }
          onClick={() => dispatch({ type: "toggleMaximizePanel" })}
        >
          <Icon name={layout.panelMaximized ? "restore" : "maximize"} />
        </button>
        <button
          type="button"
          className="wb-icon-btn"
          title="Close panel (Ctrl+J)"
          aria-label="Close panel"
          onClick={() => dispatch({ type: "togglePanel" })}
        >
          <Icon name="x" />
        </button>
      </div>
      <div className="panel-body">
        {layout.panelTab === "problems" && <Problems />}
        {layout.panelTab === "scales" &&
          (api.projectId ? (
            <ScalesPanel
              projectId={api.projectId}
              snapshot={api.diagnostics}
              loading={!api.diagnostics}
              documents={api.documents}
              selectedMarkId={api.selectedMarkId}
              onSelectMark={api.selectMark}
              onJump={api.jumpTo}
              collapsed={scalesCollapsed}
              onToggleCollapsed={() => setScalesCollapsed((value) => !value)}
            />
          ) : (
            <p className="pane-empty">Open a project.</p>
          ))}
        {layout.panelTab === "output" && <Output />}
        {layout.panelTab === "terminal" && <Console />}
        {layout.panelTab === "timeline" && <PageTimeline />}
      </div>
    </section>
  );
}

function Problems() {
  const api = useWorkbench();
  const [filter, setFilter] = useState("");
  const [showSuppressed, setShowSuppressed] = useState(false);
  const [severities, setSeverities] = useState<Severity[]>([
    "error",
    "warning",
    "information",
    "hint",
  ]);
  const rows = useMemo(
    () =>
      problemRows(api.diagnostics?.diagnostics ?? [], api.documents, {
        filter,
        showSuppressed,
        severities,
      }),
    [api.diagnostics, api.documents, filter, showSuppressed, severities],
  );
  const counts = countBySeverity(
    problemRows(api.diagnostics?.diagnostics ?? [], api.documents),
  );
  if (!api.projectId)
    return <p className="pane-empty">Open a project to see its marks.</p>;
  const toggle = (s: Severity) =>
    setSeverities((all) =>
      all.includes(s) ? all.filter((x) => x !== s) : [...all, s],
    );
  return (
    <div className="problems">
      <div className="problems-bar">
        <input
          placeholder="Filter (e.g. CANON-E201, Mara, Night crossing)"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          aria-label="Filter problems"
        />
        {(["error", "warning", "information"] as Severity[]).map((s) => (
          <button
            key={s}
            type="button"
            className={`sev-filter${severities.includes(s) ? " on" : ""}`}
            aria-pressed={severities.includes(s)}
            onClick={() => toggle(s)}
          >
            <span className={`sev-dot sev-${s}`} /> {counts[s]}{" "}
            {s === "information" ? "info" : `${s}${counts[s] === 1 ? "" : "s"}`}
          </button>
        ))}
        <label className="ctl-inline">
          <input
            type="checkbox"
            checked={showSuppressed}
            onChange={(e) => setShowSuppressed(e.target.checked)}
          />{" "}
          suppressed
        </label>
      </div>
      {rows.length === 0 ? (
        <p className="pane-empty">
          {api.diagnostics
            ? "No problems have been detected in the project."
            : "Weighing…"}
        </p>
      ) : (
        <ul className="problem-list" role="listbox" aria-label="Problems">
          {rows.map((row) => (
            <li key={row.id}>
              <button
                type="button"
                role="option"
                aria-selected={api.selectedMarkId === row.id}
                className={`problem-row${api.selectedMarkId === row.id ? " selected" : ""}${row.suppressed ? " suppressed" : ""}`}
                onClick={() => {
                  api.selectMark(row.id);
                  if (row.diagnostic.primary)
                    api.jumpTo(row.diagnostic.primary);
                }}
              >
                <span
                  className={`sev-dot sev-${row.severity}`}
                  aria-label={row.severity}
                />
                <span className="problem-msg">
                  {row.title}
                  <span className="muted">
                    {" "}
                    — {row.message.replace(/[①②③④⑤]\s*/g, "")}
                  </span>
                </span>
                <code className="problem-code">{row.code}</code>
                <span className="problem-where">
                  {row.chapter ? `${row.chapter} › ` : ""}
                  {row.scene}
                  {row.line ? ` [¶${row.line}, ${row.start + 1}]` : ""}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function Output() {
  const lines = useOutput();
  const [channel, setChannel] = useState<Channel | "All">("All");
  const shown =
    channel === "All" ? lines : lines.filter((l) => l.channel === channel);
  return (
    <div className="output">
      <div className="problems-bar">
        <select
          value={channel}
          onChange={(e) => setChannel(e.target.value as Channel | "All")}
          aria-label="Output channel"
        >
          {[
            "All",
            "Compile",
            "Packs",
            "Checks",
            "Versions",
            "Assistant",
            "Desk",
          ].map((c) => (
            <option key={c}>{c}</option>
          ))}
        </select>
        <span className="spacer" />
        <button
          type="button"
          className="wb-icon-btn"
          title="Clear output"
          aria-label="Clear output"
          onClick={clearOutput}
        >
          <Icon name="trash" />
        </button>
      </div>
      <pre className="output-log" aria-live="polite">
        {shown.length ? (
          shown.map((l, i) => (
            <div key={i} className={`out-${l.level}`}>
              [{l.at.slice(11, 19)}] [{l.channel}] {l.text}
            </div>
          ))
        ) : (
          <span className="muted">
            Nothing yet. Compile, switch a pack or run the checks.
          </span>
        )}
      </pre>
    </div>
  );
}

let history: string[] = [];
let lastLines: ConsoleLine[] = [
  {
    kind: "dim",
    text: "Asten console. Commands run against the in-app engine; this is not a system shell. Type `asten help`.",
  },
];

function Console() {
  const api = useWorkbench();
  const queryClient = useQueryClient();
  const [lines, setLines] = useState<ConsoleLine[]>(lastLines);
  const [input, setInput] = useState("");
  const [cursor, setCursor] = useState(-1);
  const [busy, setBusy] = useState(false);
  const end = useRef<HTMLDivElement>(null);
  useEffect(() => {
    lastLines = lines;
    end.current?.scrollIntoView({ block: "end" });
  }, [lines]);

  async function submit() {
    const line = input;
    setInput("");
    setCursor(-1);
    if (!line.trim()) return;
    history = [line, ...history.filter((h) => h !== line)].slice(0, 50);
    setBusy(true);
    const projectId = api.projectId;
    const result = await runCommand(line, {
      projectName: () => api.project?.name,
      diagnostics: async () => {
        await queryClient.invalidateQueries({
          queryKey: ["diagnostics", projectId],
        });
        return api.client.diagnostics(projectId!);
      },
      documents: () => api.client.listDocumentSummaries(projectId!),
      compile: (format) => api.client.compileProject(projectId!, { format }),
      packs: () => api.client.listPacks(),
      setPack: async (id, enabled) => {
        const pack = await api.client.updatePack(id, enabled);
        log(
          "Packs",
          `${pack.name} ${enabled ? "enabled" : "disabled"} from the console.`,
        );
        await queryClient.invalidateQueries({ queryKey: ["packs"] });
        await queryClient.invalidateQueries({ queryKey: ["diagnostics"] });
        return pack;
      },
      facts: () => api.client.listFacts(projectId!),
      entities: () => api.client.listEntities(projectId!),
      snapshot: async (documentId, label) => {
        const snap = await api.client.createSnapshot(
          projectId!,
          documentId,
          label,
        );
        await queryClient.invalidateQueries({ queryKey: ["snapshots"] });
        log("Versions", `Committed from the console: ${label}`);
        return snap;
      },
      openDocumentId: () => api.openDocumentId,
    });
    setBusy(false);
    if (result.artifact) {
      log("Compile", `Compiled ${result.artifact.filename} from the console.`);
      downloadArtifact(result.artifact);
    }
    setLines((all) => (result.clear ? [] : [...all, ...result.lines]));
  }

  return (
    <div
      className="console"
      onClick={() => document.getElementById("wb-console-input")?.focus()}
    >
      <div className="console-lines" role="log" aria-live="polite">
        {lines.map((line, i) => (
          <div key={i} className={`con-${line.kind}`}>
            {line.kind === "input" ? (
              <>
                <span className="con-prompt">asten&gt;</span> {line.text}
              </>
            ) : (
              line.text
            )}
          </div>
        ))}
        <div className="console-input-row" ref={end}>
          <span className="con-prompt">asten&gt;</span>
          <input
            id="wb-console-input"
            value={input}
            disabled={busy}
            spellCheck={false}
            autoComplete="off"
            aria-label="Console command"
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") void submit();
              else if (e.key === "ArrowUp") {
                e.preventDefault();
                const next = Math.min(history.length - 1, cursor + 1);
                if (history[next] !== undefined) {
                  setCursor(next);
                  setInput(history[next]);
                }
              } else if (e.key === "ArrowDown") {
                e.preventDefault();
                const next = cursor - 1;
                setCursor(Math.max(-1, next));
                setInput(next >= 0 ? history[next] : "");
              } else if (e.key === "l" && e.ctrlKey) {
                e.preventDefault();
                setLines([]);
              }
            }}
          />
        </div>
      </div>
    </div>
  );
}

function PageTimeline() {
  const api = useWorkbench();
  const id = api.openDocumentId;
  const doc = api.documents.find((d) => d.id === id);
  const events = useQuery({
    queryKey: ["history", api.projectId, id, doc?.head_seq],
    queryFn: () => api.client.history(api.projectId!, id!),
    enabled: Boolean(api.projectId && id),
  });
  const snapshots = useQuery({
    queryKey: ["snapshots", api.projectId, id],
    queryFn: () => api.client.listSnapshots(api.projectId!, id!),
    enabled: Boolean(api.projectId && id),
  });
  if (!id || !doc)
    return <p className="pane-empty">Open a page to see its timeline.</p>;
  const items = [
    ...(events.data ?? []).map((e) => ({
      at: e.at,
      kind: "edit" as const,
      text: `#${e.seq} ${e.payload.type} by ${e.actor}`,
      hash: e.chain_hash,
    })),
    ...(snapshots.data ?? []).map((s) => ({
      at: s.created_at,
      kind: "version" as const,
      text: s.label ?? "Version",
      hash: s.chain_hash,
    })),
  ].sort((a, b) => b.at.localeCompare(a.at));
  return (
    <div className="page-timeline">
      <p className="muted small">
        {doc.title} · {items.length} entries · head #{doc.head_seq}
      </p>
      <ol>
        {items.map((item, i) => (
          <li key={i} className={`tl-${item.kind}`}>
            <Icon
              name={item.kind === "version" ? "commit" : "edit"}
              size={14}
            />
            <span>{item.text}</span>
            <code className="muted">{item.hash.slice(0, 10)}</code>
            <small className="muted">{relativeTime(item.at)}</small>
          </li>
        ))}
      </ol>
      <button
        type="button"
        className="wb-btn wb-btn--small"
        onClick={() => api.setView("history")}
      >
        Open History (verify, blame, drafts)
      </button>
    </div>
  );
}
