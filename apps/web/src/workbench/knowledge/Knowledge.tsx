import { useQuery } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import {
  DEFAULT_FORCES,
  seedBodies,
  step,
  type Body,
  type Forces,
} from "./force";
import {
  DEFAULT_FILTER,
  backlinks,
  buildVault,
  graphOf,
  outgoing,
  properties,
  splitWikilinks,
  type GraphFilter,
  type NoteKind,
  type Vault,
  type VaultNote,
} from "./vault";

export function useVault(): { vault?: Vault; loading: boolean } {
  const api = useWorkbench();
  const projectId = api.projectId;
  const documents = useQuery({
    queryKey: ["documents-full", projectId],
    queryFn: () => api.client.listDocuments(projectId!),
    enabled: Boolean(projectId),
    staleTime: 5000,
  });
  // The summaries refresh on every save; follow them so the vault does too.
  const heads = api.documents.map((d) => `${d.id}:${d.head_seq}`).join(",");
  const refetch = documents.refetch;
  useEffect(() => {
    if (projectId) void refetch();
  }, [heads, projectId, refetch]);
  const vault = useMemo(
    () =>
      documents.data ? buildVault(documents.data, api.entities) : undefined,
    [documents.data, api.entities],
  );
  return { vault, loading: documents.isPending };
}

export function useFacts() {
  const api = useWorkbench();
  return useQuery({
    queryKey: ["facts", api.projectId],
    queryFn: () => api.client.listFacts(api.projectId!),
    enabled: Boolean(api.projectId),
  });
}

const KIND_ICON: Record<NoteKind, string> = {
  character: "person",
  place: "pin",
  item: "key",
  note: "note",
  source: "source",
  scene: "file",
  unresolved: "link",
};

const FOLDERS = [
  "Characters",
  "Places",
  "Items",
  "Notes",
  "Research",
  "Manuscript",
  "Unresolved",
];

/** The vault-style file tree in the side bar. */
export function KnowledgePane() {
  const api = useWorkbench();
  const { vault, loading } = useVault();
  const [query, setQuery] = useState("");
  const [closed, setClosed] = useState<Set<string>>(
    new Set(["Manuscript", "Unresolved"]),
  );
  if (!api.projectId)
    return <p className="pane-empty">Open a project to see its notes.</p>;
  if (loading || !vault) return <p className="pane-empty">Reading notes…</p>;
  const q = query.trim().toLowerCase();
  return (
    <div className="knowledge-pane">
      <div className="pane-tools">
        <button
          type="button"
          className="wb-icon-btn"
          title="New note"
          aria-label="New note"
          onClick={() => api.run("app.newNote")}
        >
          <Icon name="filePlus" />
        </button>
        <button
          type="button"
          className="wb-icon-btn"
          title="Graph view (Ctrl+G)"
          aria-label="Graph view"
          onClick={() => api.setView("graph")}
        >
          <Icon name="graph" />
        </button>
        <button
          type="button"
          className="wb-icon-btn"
          title="Quick switcher (Ctrl+O)"
          aria-label="Quick switcher"
          onClick={() => api.run("switcher.open")}
        >
          <Icon name="search" />
        </button>
        <button
          type="button"
          className="wb-icon-btn"
          title="Collapse all"
          aria-label="Collapse all"
          onClick={() => setClosed(new Set(FOLDERS))}
        >
          <Icon name="chevronUp" />
        </button>
      </div>
      <input
        className="pane-filter"
        placeholder="Filter notes"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        aria-label="Filter notes"
      />
      <div className="vault-tree" role="tree" aria-label="Notes">
        {FOLDERS.map((folder) => {
          const notes = vault.notes.filter(
            (n) =>
              n.folder === folder && (!q || n.title.toLowerCase().includes(q)),
          );
          if (!notes.length) return null;
          const open = !closed.has(folder) || Boolean(q);
          return (
            <div key={folder} role="treeitem" aria-expanded={open}>
              <button
                type="button"
                className="vault-folder"
                onClick={() =>
                  setClosed((s) => {
                    const n = new Set(s);
                    if (n.has(folder)) n.delete(folder);
                    else n.add(folder);
                    return n;
                  })
                }
              >
                <Icon name={open ? "chevronDown" : "chevronRight"} size={12} />{" "}
                {folder} <small>{notes.length}</small>
              </button>
              {open && (
                <div role="group">
                  {notes.map((note) => (
                    <button
                      key={note.id}
                      type="button"
                      role="treeitem"
                      className={`vault-note${api.selectedNoteId === note.id ? " selected" : ""}${note.kind === "unresolved" ? " unresolved" : ""}`}
                      onClick={() => api.openNote(note.id)}
                    >
                      <Icon name={KIND_ICON[note.kind]} size={14} />
                      <span>{note.title}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

/** A note in reading view: properties, the text with live wikilinks, and links. */
export function NoteView() {
  const api = useWorkbench();
  const { vault } = useVault();
  const facts = useFacts();
  const note = api.selectedNoteId
    ? vault?.byId.get(api.selectedNoteId)
    : undefined;
  if (!vault) return <p className="status">Reading notes…</p>;
  if (!note) return <p className="status">That note is gone.</p>;
  const open = (title: string) => {
    const target = vault.notes.find(
      (n) =>
        n.title.toLowerCase() === title.toLowerCase() ||
        n.aliases.some((a) => a.toLowerCase() === title.toLowerCase()),
    );
    if (target) api.openNote(target.id);
    else
      api.toast(
        `“${title}” has no note yet. Create it from the Knowledge pane.`,
      );
  };
  const props = properties(note, facts.data ?? []);
  const back = backlinks(vault, note.id);
  return (
    <article className="note-view">
      <header>
        <p className="eyebrow">{note.folder}</p>
        <h1>{note.title}</h1>
        <div className="note-actions">
          {note.documentId && (
            <button
              type="button"
              className="wb-btn wb-btn--small"
              onClick={() => api.openDocument(note.documentId!)}
            >
              <Icon name="edit" /> Edit
            </button>
          )}
          {note.entityId && (
            <button
              type="button"
              className="wb-btn wb-btn--small"
              onClick={() => {
                api.run("app.openCast", note.entityKey);
              }}
            >
              <Icon name="people" /> Open in Cast
            </button>
          )}
          <button
            type="button"
            className="wb-btn wb-btn--small"
            onClick={() =>
              api.dispatch({ type: "showSecondary", tab: "links" })
            }
          >
            <Icon name="graph" /> Local graph
          </button>
        </div>
      </header>
      <dl className="note-props">
        {props.map((p) => (
          <div key={p.key + p.value}>
            <dt>{p.key}</dt>
            <dd>
              {p.value}
              {p.status && p.status !== "confirmed" ? (
                <span className="wb-pill">{p.status}</span>
              ) : null}
            </dd>
          </div>
        ))}
      </dl>
      <div className="note-body">
        {note.text ? (
          note.text
            .split("\n")
            .filter(Boolean)
            .map((para, i) => (
              <p key={i}>
                {splitWikilinks(para).map((part, j) =>
                  "link" in part ? (
                    <button
                      key={j}
                      type="button"
                      className="wikilink"
                      onClick={() => open(part.link)}
                    >
                      {part.shown}
                    </button>
                  ) : (
                    <span key={j}>{part.text}</span>
                  ),
                )}
              </p>
            ))
        ) : (
          <p className="muted">
            {note.kind === "unresolved"
              ? "Linked to, but not written yet."
              : "No notes yet."}
          </p>
        )}
      </div>
      <section className="note-backlinks">
        <h3>
          Linked mentions <small>{back.length}</small>
        </h3>
        {back.map((b) => (
          <button
            key={b.from}
            type="button"
            className="backlink"
            onClick={() => api.openNote(b.from)}
          >
            <strong>{b.note.title}</strong>
            <span>{b.context}</span>
          </button>
        ))}
      </section>
    </article>
  );
}

/** Backlinks, outgoing links, properties and a local graph, for the right side bar. */
export function LinksPanel() {
  const api = useWorkbench();
  const { vault } = useVault();
  const facts = useFacts();
  if (!api.projectId) return <p className="pane-empty">Open a project.</p>;
  if (!vault) return <p className="pane-empty">Reading notes…</p>;
  const noteId =
    api.selectedNoteId ??
    (api.openDocumentId
      ? vault.notes.find((n) => n.documentId === api.openDocumentId)?.id
      : undefined);
  const note = noteId ? vault.byId.get(noteId) : undefined;
  if (!note)
    return (
      <p className="pane-empty">
        Open a note or a page to see what links to it.
      </p>
    );
  const back = backlinks(vault, note.id);
  const out = outgoing(vault, note.id);
  return (
    <div className="links-panel">
      <h4>{note.title}</h4>
      <div className="local-graph">
        <GraphCanvas
          vault={vault}
          filter={DEFAULT_FILTER}
          center={{ id: note.id, depth: 1 }}
          compact
        />
      </div>
      <details open>
        <summary>Properties</summary>
        <dl className="note-props compact">
          {properties(note, facts.data ?? []).map((p) => (
            <div key={p.key + p.value}>
              <dt>{p.key}</dt>
              <dd>{p.value}</dd>
            </div>
          ))}
        </dl>
      </details>
      <details open>
        <summary>
          Backlinks <small>{back.length}</small>
        </summary>
        {back.map((b) => (
          <button
            key={b.from}
            type="button"
            className="backlink"
            onClick={() => api.openNote(b.from)}
          >
            <strong>{b.note.title}</strong>
            <span>{b.context}</span>
          </button>
        ))}
      </details>
      <details open>
        <summary>
          Outgoing links <small>{out.length}</small>
        </summary>
        {out.map((n) => (
          <button
            key={n.id}
            type="button"
            className="backlink"
            onClick={() => api.openNote(n.id)}
          >
            <strong>{n.title}</strong>
          </button>
        ))}
      </details>
    </div>
  );
}

const GROUP_COLOURS: Record<NoteKind, string> = {
  character: "var(--wb-accent)",
  place: "#2e6b4e",
  item: "#b98419",
  note: "var(--wb-muted-fg)",
  source: "#1f3f77",
  scene: "var(--wb-fg)",
  unresolved: "var(--wb-border-strong)",
};

/** The graph view with Obsidian's controls: filters, groups, display, forces. */
export function GraphView() {
  const api = useWorkbench();
  const { vault } = useVault();
  const [filter, setFilter] = useState<GraphFilter>(DEFAULT_FILTER);
  const [forces, setForces] = useState<Forces>(DEFAULT_FORCES);
  const [display, setDisplay] = useState({
    arrows: false,
    labels: true,
    nodeSize: 1,
    linkWidth: 1,
  });
  const [panel, setPanel] = useState(true);
  if (!api.projectId) return <p className="status">Open a project.</p>;
  if (!vault) return <p className="status">Reading notes…</p>;
  const kindToggle = (kind: NoteKind) =>
    setFilter((f) => ({
      ...f,
      kinds: f.kinds.includes(kind)
        ? f.kinds.filter((k) => k !== kind)
        : [...f.kinds, kind],
    }));
  return (
    <div className="graph-view">
      <GraphCanvas
        vault={vault}
        filter={filter}
        forces={forces}
        display={display}
      />
      <button
        type="button"
        className="wb-icon-btn graph-gear"
        aria-label="Graph controls"
        aria-expanded={panel}
        onClick={() => setPanel((v) => !v)}
      >
        <Icon name="settings" />
      </button>
      {panel && (
        <aside className="graph-controls" aria-label="Graph controls">
          <details open>
            <summary>Filters</summary>
            <input
              placeholder="Search notes…"
              value={filter.search}
              onChange={(e) => setFilter({ ...filter, search: e.target.value })}
              aria-label="Search notes"
            />
            <Toggle
              label="Unresolved links"
              value={filter.showUnresolved}
              onChange={(v) => setFilter({ ...filter, showUnresolved: v })}
            />
            <Toggle
              label="Scenes"
              value={filter.showScenes}
              onChange={(v) => setFilter({ ...filter, showScenes: v })}
            />
            <Toggle
              label="Orphans"
              value={filter.showOrphans}
              onChange={(v) => setFilter({ ...filter, showOrphans: v })}
            />
          </details>
          <details open>
            <summary>Groups</summary>
            {(Object.keys(GROUP_COLOURS) as NoteKind[]).map((kind) => (
              <label key={kind} className="group-row">
                <input
                  type="checkbox"
                  checked={filter.kinds.includes(kind)}
                  onChange={() => kindToggle(kind)}
                />
                <span
                  className="group-dot"
                  style={{ background: GROUP_COLOURS[kind] }}
                />
                {kind}
              </label>
            ))}
          </details>
          <details>
            <summary>Display</summary>
            <Toggle
              label="Arrows"
              value={display.arrows}
              onChange={(v) => setDisplay({ ...display, arrows: v })}
            />
            <Toggle
              label="Labels"
              value={display.labels}
              onChange={(v) => setDisplay({ ...display, labels: v })}
            />
            <Slider
              label="Node size"
              min={0.5}
              max={2}
              step={0.1}
              value={display.nodeSize}
              onChange={(v) => setDisplay({ ...display, nodeSize: v })}
            />
            <Slider
              label="Link thickness"
              min={0.5}
              max={3}
              step={0.1}
              value={display.linkWidth}
              onChange={(v) => setDisplay({ ...display, linkWidth: v })}
            />
          </details>
          <details>
            <summary>Forces</summary>
            <Slider
              label="Center force"
              min={0}
              max={0.2}
              step={0.005}
              value={forces.center}
              onChange={(v) => setForces({ ...forces, center: v })}
            />
            <Slider
              label="Repel force"
              min={100}
              max={3000}
              step={50}
              value={forces.repel}
              onChange={(v) => setForces({ ...forces, repel: v })}
            />
            <Slider
              label="Link force"
              min={0}
              max={0.2}
              step={0.005}
              value={forces.link}
              onChange={(v) => setForces({ ...forces, link: v })}
            />
            <Slider
              label="Link distance"
              min={30}
              max={250}
              step={5}
              value={forces.distance}
              onChange={(v) => setForces({ ...forces, distance: v })}
            />
          </details>
        </aside>
      )}
    </div>
  );
}

function Toggle({
  label,
  value,
  onChange,
}: {
  label: string;
  value: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="ctl-toggle">
      <span>{label}</span>
      <input
        type="checkbox"
        role="switch"
        checked={value}
        onChange={(e) => onChange(e.target.checked)}
      />
    </label>
  );
}

function Slider({
  label,
  value,
  onChange,
  min,
  max,
  step,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min: number;
  max: number;
  step: number;
}) {
  return (
    <label className="ctl-slider">
      <span>{label}</span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  );
}

/** An SVG graph laid out by the force simulation; drag to pan, wheel to zoom, click a node to open it. */
function GraphCanvas({
  vault,
  filter,
  forces = DEFAULT_FORCES,
  display = { arrows: false, labels: true, nodeSize: 1, linkWidth: 1 },
  center,
  compact,
}: {
  vault: Vault;
  filter: GraphFilter;
  forces?: Forces;
  display?: {
    arrows: boolean;
    labels: boolean;
    nodeSize: number;
    linkWidth: number;
  };
  center?: { id: string; depth: number };
  compact?: boolean;
}) {
  const api = useWorkbench();
  const { nodes, edges } = useMemo(
    () => graphOf(vault, filter, center),
    [vault, filter, center?.id, center?.depth],
  ); // eslint-disable-line react-hooks/exhaustive-deps
  const bodiesRef = useRef<Map<string, Body>>(new Map());
  const [, setTick] = useState(0);
  const [view, setView] = useState({ x: 0, y: 0, k: compact ? 0.8 : 1 });
  const [hover, setHover] = useState<string>();
  const drag = useRef<{ x: number; y: number; vx: number; vy: number } | null>(
    null,
  );
  const reduce =
    typeof document !== "undefined" &&
    document.documentElement.dataset.motion === "reduce";

  useEffect(() => {
    const bodies = seedBodies(
      nodes.map((n) => n.id),
      bodiesRef.current,
    );
    bodiesRef.current = new Map(bodies.map((b) => [b.id, b]));
    let frame = 0;
    let raf = 0;
    const run = () => {
      const energy = step(bodies, edges, forces);
      frame += 1;
      if (!reduce) setTick((t) => t + 1);
      if (frame < 400 && energy > 0.05) raf = requestAnimationFrame(run);
      else setTick((t) => t + 1);
    };
    if (reduce) {
      for (let i = 0; i < 300; i += 1) step(bodies, edges, forces);
      setTick((t) => t + 1);
    } else raf = requestAnimationFrame(run);
    return () => cancelAnimationFrame(raf);
  }, [nodes, edges, forces, reduce]);

  // Fit the drawing to the laid-out bodies, so a small graph fills the view.
  const all = [...bodiesRef.current.values()];
  const pad = compact ? 40 : 80;
  const minX = Math.min(-150, ...all.map((b) => b.x)) - pad;
  const maxX = Math.max(150, ...all.map((b) => b.x)) + pad;
  const minY = Math.min(-100, ...all.map((b) => b.y)) - pad;
  const maxY = Math.max(100, ...all.map((b) => b.y)) + pad;
  const neighbours = new Set(
    hover
      ? edges.flatMap((e) =>
          e.from === hover ? [e.to] : e.to === hover ? [e.from] : [],
        )
      : [],
  );
  const pos = (id: string) => bodiesRef.current.get(id);
  const radius = (note: VaultNote) =>
    (4 +
      Math.min(
        8,
        Math.sqrt(
          edges.filter((e) => e.from === note.id || e.to === note.id).length,
        ) * 2,
      )) *
    display.nodeSize;

  return (
    <svg
      className={`graph-canvas${compact ? " compact" : ""}`}
      viewBox={`${minX} ${minY} ${maxX - minX} ${maxY - minY}`}
      role="img"
      aria-label={`Graph of ${nodes.length} notes and ${edges.length} links`}
      onWheel={(e) =>
        setView((v) => ({
          ...v,
          k: Math.max(0.3, Math.min(3, v.k * (e.deltaY < 0 ? 1.1 : 0.9))),
        }))
      }
      onPointerDown={(e) => {
        drag.current = { x: e.clientX, y: e.clientY, vx: view.x, vy: view.y };
      }}
      onPointerMove={(e) => {
        if (drag.current)
          setView((v) => ({
            ...v,
            x: drag.current!.vx + (e.clientX - drag.current!.x) / v.k,
            y: drag.current!.vy + (e.clientY - drag.current!.y) / v.k,
          }));
      }}
      onPointerUp={() => {
        drag.current = null;
      }}
      onPointerLeave={() => {
        drag.current = null;
      }}
    >
      <defs>
        <marker
          id="wb-arrow"
          viewBox="0 0 10 10"
          refX="14"
          refY="5"
          markerWidth="5"
          markerHeight="5"
          orient="auto-start-reverse"
        >
          <path d="M0 0 L10 5 L0 10z" fill="currentColor" />
        </marker>
      </defs>
      <g transform={`scale(${view.k}) translate(${view.x} ${view.y})`}>
        {edges.map((edge) => {
          const a = pos(edge.from);
          const b = pos(edge.to);
          if (!a || !b) return null;
          const lit = hover && (edge.from === hover || edge.to === hover);
          return (
            <line
              key={`${edge.from}-${edge.to}`}
              x1={a.x}
              y1={a.y}
              x2={b.x}
              y2={b.y}
              className={`graph-edge${lit ? " lit" : ""}${hover && !lit ? " dim" : ""}${edge.via === "mention" ? " mention" : ""}`}
              strokeWidth={display.linkWidth}
              markerEnd={display.arrows ? "url(#wb-arrow)" : undefined}
            />
          );
        })}
        {nodes.map((note) => {
          const p = pos(note.id);
          if (!p) return null;
          const dim = hover && hover !== note.id && !neighbours.has(note.id);
          return (
            <g
              key={note.id}
              className={`graph-node${dim ? " dim" : ""}${note.id === center?.id ? " center" : ""}`}
              transform={`translate(${p.x} ${p.y})`}
              onPointerEnter={() => setHover(note.id)}
              onPointerLeave={() => setHover(undefined)}
              onClick={(e) => {
                e.stopPropagation();
                api.openNote(note.id);
              }}
              role="button"
              tabIndex={compact ? -1 : 0}
              aria-label={note.title}
              onKeyDown={(e) => e.key === "Enter" && api.openNote(note.id)}
            >
              <circle r={radius(note)} fill={GROUP_COLOURS[note.kind]} />
              {display.labels &&
                (!compact ||
                  hover === note.id ||
                  note.id === center?.id ||
                  nodes.length < 14) && (
                  <text y={radius(note) + 12} textAnchor="middle">
                    {note.title}
                  </text>
                )}
            </g>
          );
        })}
      </g>
    </svg>
  );
}
