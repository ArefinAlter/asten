import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import type { AstenDiagnostic } from "@asten/contracts";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import { buildVault } from "../knowledge/vault";
import { log } from "../output";
import { deskStore } from "../../lib/storage";
import { setPrefs, usePrefs } from "../prefs";
import { respond, type Card, type Chip, type Turn } from "./script";
import { SKILLS } from "./skills";
import type { Access } from "./tools";

interface Message {
  id: string;
  role: "person" | "assistant";
  text: string;
  chips?: Chip[];
  turn?: Turn;
  at: string;
}

interface Thread {
  id: string;
  title: string;
  access: Access;
  messages: Message[];
}

const THREADS_KEY = "asten-workbench:threads:v1";

let serial = 0;
const id = () => `t${Date.now().toString(36)}${(serial += 1).toString(36)}`;

export const FIXED_RULES = [
  "Never writes, rewrites, continues or autocompletes your prose.",
  "Never settles canon or applies a repair by itself: you press every card.",
  "Never raises its own access, changes settings or reads provider keys.",
  "Summarises source and research notes only, never manuscript scenes.",
  "Every tool call is listed, including the ones refused.",
];

export function AgentPanel() {
  const api = useWorkbench();
  const prefs = usePrefs();
  const queryClient = useQueryClient();
  const [threads, setThreads] = useState<Thread[]>(() => {
    const kept = deskStore.getJson<Thread[]>(THREADS_KEY);
    return Array.isArray(kept) && kept.length
      ? kept
      : [
          {
            id: id(),
            title: "New thread",
            access: prefs.agentAccess,
            messages: [],
          },
        ];
  });
  // Threads are kept like preferences (desk storage, or the hosting
  // service): the newest twenty, eighty messages each.
  useEffect(() => {
    deskStore.setJson(
      THREADS_KEY,
      threads
        .slice(-20)
        .map((t) => ({ ...t, messages: t.messages.slice(-80) })),
    );
  }, [threads]);
  const [activeId, setActiveId] = useState(threads[0].id);
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const [chips, setChips] = useState<Chip[]>([]);
  const [showRules, setShowRules] = useState(false);
  const [showThreads, setShowThreads] = useState(false);
  const scroller = useRef<HTMLDivElement>(null);
  const thread = threads.find((t) => t.id === activeId) ?? threads[0];

  // Chips the person can attach: the open scene, the selected mark, the open note.
  const suggestions = useMemo(() => {
    const out: Chip[] = [];
    const doc = api.documents.find((d) => d.id === api.openDocumentId);
    if (doc) out.push({ kind: "scene", id: doc.id, label: doc.title });
    const mark = api.diagnostics?.diagnostics.find(
      (d) => d.id === api.selectedMarkId,
    );
    if (mark) out.push({ kind: "mark", id: mark.id, label: mark.code });
    if (api.selectedNoteId)
      out.push({
        kind: "note",
        id: api.selectedNoteId,
        label: api.selectedNoteId.replace(/^\w+:/, ""),
      });
    return out.filter((c) => !chips.some((x) => x.id === c.id));
  }, [
    api.documents,
    api.openDocumentId,
    api.diagnostics,
    api.selectedMarkId,
    api.selectedNoteId,
    chips,
  ]);

  useEffect(() => {
    scroller.current?.scrollTo({ top: scroller.current.scrollHeight });
  }, [thread.messages.length, busy]);

  const update = (patch: Partial<Thread>) =>
    setThreads((all) =>
      all.map((t) => (t.id === thread.id ? { ...t, ...patch } : t)),
    );

  async function send(text = draft) {
    const trimmed = text.trim();
    if (!trimmed || busy) return;
    if (!api.projectId) {
      api.toast("Open a project first.");
      return;
    }
    const person: Message = {
      id: id(),
      role: "person",
      text: trimmed,
      chips,
      at: new Date().toISOString(),
    };
    const title = thread.messages.length ? thread.title : trimmed.slice(0, 48);
    setThreads((all) =>
      all.map((t) =>
        t.id === thread.id
          ? { ...t, title, messages: [...t.messages, person] }
          : t,
      ),
    );
    setDraft("");
    setBusy(true);
    const projectId = api.projectId;
    const client = api.client;
    const fetchFacts = () =>
      queryClient.fetchQuery({
        queryKey: ["facts", projectId],
        queryFn: () => client.listFacts(projectId),
      });
    const fetchEntities = () =>
      queryClient.fetchQuery({
        queryKey: ["entities", projectId],
        queryFn: () => client.listEntities(projectId),
      });
    const turn = await respond(trimmed, {
      projectName: api.project?.name,
      access: thread.access,
      disabledTools: new Set(prefs.disabledTools),
      chips,
      diagnostics: async () =>
        (await client.diagnostics(projectId)).diagnostics,
      rerunChecks: async () => {
        await queryClient.invalidateQueries({
          queryKey: ["diagnostics", projectId],
        });
        return (await client.diagnostics(projectId)).diagnostics;
      },
      documents: () => client.listDocumentSummaries(projectId),
      facts: fetchFacts,
      entities: fetchEntities,
      events: () => client.listEvents(projectId),
      vault: async () => {
        const [documents, entities] = await Promise.all([
          queryClient.fetchQuery({
            queryKey: ["documents-full", projectId],
            queryFn: () => client.listDocuments(projectId),
            staleTime: 5000,
          }),
          fetchEntities(),
        ]);
        return buildVault(documents, entities);
      },
    });
    for (const call of turn.calls)
      log(
        "Assistant",
        `${call.allowed ? "called" : "refused"} ${call.tool}(${call.input})${call.reason ? ` — ${call.reason}` : call.output ? ` → ${call.output}` : ""}`,
        call.allowed ? "info" : "warn",
      );
    const reply: Message = {
      id: id(),
      role: "assistant",
      text: turn.reply,
      turn,
      at: new Date().toISOString(),
    };
    setThreads((all) =>
      all.map((t) =>
        t.id === thread.id ? { ...t, messages: [...t.messages, reply] } : t,
      ),
    );
    setChips([]);
    setBusy(false);
  }

  if (!prefs.agentEnabled)
    return (
      <div className="agent-off">
        <Icon name="agent" size={28} />
        <p>The assistant is switched off.</p>
        <button
          type="button"
          className="wb-btn"
          onClick={() => setPrefs({ agentEnabled: true })}
        >
          Switch it on
        </button>
      </div>
    );

  return (
    <div className="agent">
      <div className="agent-head">
        <button
          type="button"
          className="agent-thread-btn"
          onClick={() => setShowThreads((v) => !v)}
          aria-expanded={showThreads}
        >
          <span>{thread.title}</span>
          <Icon name="chevronDown" />
        </button>
        <button
          type="button"
          className="wb-icon-btn"
          title="New thread"
          aria-label="New thread"
          onClick={() => {
            const t: Thread = {
              id: id(),
              title: "New thread",
              access: prefs.agentAccess,
              messages: [],
            };
            setThreads((all) => [t, ...all]);
            setActiveId(t.id);
          }}
        >
          <Icon name="plus" />
        </button>
        <button
          type="button"
          className={`wb-icon-btn${showRules ? " active" : ""}`}
          title="What the assistant never does"
          aria-label="Fixed rules"
          onClick={() => setShowRules((v) => !v)}
        >
          <Icon name="shield" />
        </button>
      </div>
      {showThreads && (
        <div className="agent-threads" role="listbox" aria-label="Threads">
          {threads.map((t) => (
            <button
              key={t.id}
              type="button"
              role="option"
              aria-selected={t.id === thread.id}
              className={t.id === thread.id ? "active" : ""}
              onClick={() => {
                setActiveId(t.id);
                setShowThreads(false);
              }}
            >
              <span>{t.title}</span>
              <small>
                {t.access === "read" ? "Read only" : "Propose"} ·{" "}
                {t.messages.length} msg
              </small>
            </button>
          ))}
        </div>
      )}
      {showRules && <RulesCard />}
      <div className="agent-timeline" ref={scroller}>
        {thread.messages.length === 0 ? (
          <div className="agent-empty">
            <h3>What should I look at?</h3>
            <p className="muted">
              I'm an editor's assistant: I check, explain, search and propose. I
              never write your prose.
            </p>
            <div className="agent-starts">
              {SKILLS.filter((s) => prefs.skills[s.id]?.enabled !== false)
                .slice(0, 5)
                .map((skill) => (
                  <button
                    key={skill.id}
                    type="button"
                    onClick={() => void send(skill.prompt)}
                  >
                    <Icon name="skill" />
                    <span>
                      <strong>{skill.name}</strong>
                      <small>{skill.prompt}</small>
                    </span>
                  </button>
                ))}
            </div>
          </div>
        ) : (
          thread.messages.map((message) =>
            message.role === "person" ? (
              <div key={message.id} className="msg-person">
                {message.chips?.length ? (
                  <div className="chip-row">
                    {message.chips.map((c) => (
                      <span key={c.id} className="ctx-chip">
                        <Icon name={chipIcon(c)} size={12} />
                        {c.label}
                      </span>
                    ))}
                  </div>
                ) : null}
                <p>{message.text}</p>
              </div>
            ) : (
              <div key={message.id} className="msg-assistant">
                {message.turn && <ToolLog turn={message.turn} />}
                <Prose text={message.text} />
                {message.turn?.cards.map((card, index) => (
                  <CardView key={index} card={card} access={thread.access} />
                ))}
              </div>
            ),
          )
        )}
        {busy && <p className="muted agent-working">Working…</p>}
      </div>
      <div className="composer">
        {(chips.length > 0 || suggestions.length > 0) && (
          <div className="chip-row" aria-label="Context">
            {chips.map((c) => (
              <button
                key={c.id}
                type="button"
                className="ctx-chip attached"
                title="Remove from context"
                onClick={() => setChips(chips.filter((x) => x.id !== c.id))}
              >
                <Icon name={chipIcon(c)} size={12} />
                {c.label}
                <Icon name="x" size={10} />
              </button>
            ))}
            {suggestions.map((c) => (
              <button
                key={c.id}
                type="button"
                className="ctx-chip suggested"
                title="Add to context"
                onClick={() => setChips([...chips, c])}
              >
                <Icon name="plus" size={10} />
                {c.label}
              </button>
            ))}
          </div>
        )}
        <textarea
          value={draft}
          rows={2}
          placeholder="Ask about marks, canon, links or sources…"
          aria-label="Message the assistant"
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              void send();
            }
          }}
        />
        <div className="composer-row">
          <select
            className="composer-pick"
            value="rules"
            aria-label="Model"
            onChange={() => api.openSettings("providers")}
            title="No model is called in this release"
          >
            <option value="rules">Asten rules</option>
            {prefs.aiEnabled &&
              prefs.providers
                .filter((p) => p.enabled && p.consent)
                .map((p) => (
                  <option key={p.id} value={p.id} disabled>
                    {p.name} · not called in this release
                  </option>
                ))}
            <option value="settings">Providers…</option>
          </select>
          <select
            className="composer-pick"
            aria-label="Access in this thread"
            value={thread.access}
            onChange={(e) => update({ access: e.target.value as Access })}
            title="Access for this thread. The assistant never raises its own."
          >
            <option value="read">Read only</option>
            <option value="propose">Propose</option>
          </select>
          <span className="spacer" />
          <button
            type="button"
            className="send-btn"
            aria-label="Send"
            disabled={!draft.trim() || busy}
            onClick={() => void send()}
          >
            <Icon name="send" />
          </button>
        </div>
      </div>
    </div>
  );
}

function chipIcon(chip: Chip) {
  return chip.kind === "mark"
    ? "problems"
    : chip.kind === "scene"
      ? "file"
      : chip.kind === "note"
        ? "note"
        : "select";
}

export function RulesCard() {
  return (
    <div className="rules-card">
      <strong>
        <Icon name="shield" /> Editor, not writer
      </strong>
      <ul>
        {FIXED_RULES.map((rule) => (
          <li key={rule}>{rule}</li>
        ))}
      </ul>
      <small className="muted">
        Fixed rules. No setting, skill or access level changes them.
      </small>
    </div>
  );
}

function ToolLog({ turn }: { turn: Turn }) {
  const [open, setOpen] = useState(false);
  if (!turn.calls.length) return null;
  const refused = turn.calls.filter((c) => !c.allowed).length;
  const ms = turn.calls.reduce((sum, c) => sum + c.ms, 0);
  return (
    <div className={`tool-log${refused ? " has-refused" : ""}`}>
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
      >
        <Icon name={open ? "chevronDown" : "chevronRight"} size={12} />
        Worked · {turn.calls.length} tool call
        {turn.calls.length === 1 ? "" : "s"}
        {refused ? ` · ${refused} refused` : ""} · {ms} ms
      </button>
      {open && (
        <ol>
          {turn.calls.map((call, index) => (
            <li key={index} className={call.allowed ? "" : "refused"}>
              <code>{call.tool}</code>{" "}
              <span className="muted">{call.input}</span>
              <div>
                {call.allowed ? (
                  <>→ {call.output}</>
                ) : (
                  <>
                    <strong>refused</strong> · {call.reason}
                  </>
                )}
              </div>
            </li>
          ))}
        </ol>
      )}
    </div>
  );
}

/** A safe subset of Markdown: paragraphs and **bold**. No HTML, no links. */
function Prose({ text }: { text: string }) {
  return (
    <div className="prose-reply">
      {text.split(/\n\n+/).map((para, i) => (
        <p key={i}>
          {para
            .split(/(\*\*[^*]+\*\*)/)
            .map((part, j) =>
              part.startsWith("**") ? (
                <strong key={j}>{part.slice(2, -2)}</strong>
              ) : (
                <span key={j}>{part}</span>
              ),
            )}
        </p>
      ))}
    </div>
  );
}

function CardView({ card, access }: { card: Card; access: Access }) {
  const api = useWorkbench();
  const queryClient = useQueryClient();
  const [done, setDone] = useState<Record<string, string>>({});
  const refresh = async () => {
    await Promise.all([
      queryClient.invalidateQueries({
        queryKey: ["diagnostics", api.projectId],
      }),
      queryClient.invalidateQueries({ queryKey: ["facts", api.projectId] }),
      queryClient.invalidateQueries({ queryKey: ["entities", api.projectId] }),
    ]);
  };
  const jump = (mark: AstenDiagnostic) => {
    api.selectMark(mark.id);
    if (mark.primary) api.jumpTo(mark.primary);
  };
  switch (card.type) {
    case "rules":
      return <RulesCard />;
    case "marks":
      return (
        <div className="agent-card">
          <header>Open marks</header>
          {card.marks.slice(0, 8).map((mark) => (
            <button
              key={mark.id}
              type="button"
              className="card-row"
              onClick={() => jump(mark)}
            >
              <span className={`sev-dot sev-${mark.severity}`} />
              <code>{mark.code}</code>
              <span className="card-row-main">{mark.title}</span>
              <small className="muted">
                {card.titles[mark.primary?.document ?? ""] ?? ""}
              </small>
            </button>
          ))}
        </div>
      );
    case "explain":
      return (
        <div className="agent-card">
          <header>
            <code>{card.mark.code}</code> {card.mark.title}{" "}
            <small className="muted">· {card.title}</small>
          </header>
          <ol className="evidence">
            {card.mark.explanation.map((s) => (
              <li key={s.id}>
                {s.text} <small className="muted">({s.status})</small>
              </li>
            ))}
          </ol>
          <div className="card-actions">
            <button
              type="button"
              className="wb-btn wb-btn--small"
              onClick={() => jump(card.mark)}
            >
              Go to the words
            </button>
            {card.mark.repairs.map((repair, index) =>
              card.canRepair && access === "propose" ? (
                <button
                  key={index}
                  type="button"
                  className="wb-btn wb-btn--small"
                  disabled={Boolean(done.repair)}
                  onClick={async () => {
                    try {
                      const result = await api.client.applyRepair(
                        api.projectId!,
                        {
                          diagnostic_id: card.mark.id,
                          repair_index: index,
                          reason: "Chosen by the author in the assistant panel",
                        },
                      );
                      setDone({ repair: result.message });
                      log("Checks", `Repair applied by you: ${repair.label}`);
                      await refresh();
                    } catch (error) {
                      api.toast(
                        error instanceof Error ? error.message : "Failed",
                        "error",
                      );
                    }
                  }}
                >
                  {repair.label} · as you
                </button>
              ) : (
                <span key={index} className="wb-pill">
                  {repair.label}
                </span>
              ),
            )}
          </div>
          {done.repair && <p className="ok-line">{done.repair}</p>}
        </div>
      );
    case "facts":
      return (
        <div className="agent-card">
          <header>Proposed facts · you settle</header>
          {card.facts.map((fact) => (
            <div key={fact.id} className="card-row static">
              <span className="card-row-main">
                <strong>
                  {card.names[fact.entity_key] ?? fact.entity_key}
                </strong>{" "}
                {fact.predicate} = <em>{fact.value}</em>
              </span>
              {done[fact.id] ? (
                <span className="wb-pill">{done[fact.id]}</span>
              ) : card.canPropose && access === "propose" ? (
                <>
                  <button
                    type="button"
                    className="wb-btn wb-btn--small wb-btn--primary"
                    onClick={async () => {
                      await api.client.updateFact(api.projectId!, fact.id, {
                        status: "confirmed",
                        actor: "author",
                      });
                      setDone((d) => ({ ...d, [fact.id]: "settled by you" }));
                      log(
                        "Checks",
                        `Fact settled by you: ${fact.predicate} = ${fact.value}`,
                      );
                      await refresh();
                    }}
                  >
                    Settle
                  </button>
                  <button
                    type="button"
                    className="wb-btn wb-btn--small"
                    onClick={async () => {
                      await api.client.updateFact(api.projectId!, fact.id, {
                        status: "rejected",
                        actor: "author",
                      });
                      setDone((d) => ({ ...d, [fact.id]: "rejected by you" }));
                      await refresh();
                    }}
                  >
                    Reject
                  </button>
                </>
              ) : (
                <span className="muted">proposed</span>
              )}
            </div>
          ))}
        </div>
      );
    case "canon":
      return (
        <div className="agent-card">
          <header>
            {card.entity.name}{" "}
            <small className="muted">· {card.entity.kind}</small>
          </header>
          <table className="wb-table compact">
            <tbody>
              {card.facts.map((f) => (
                <tr key={f.id}>
                  <td>{f.predicate}</td>
                  <td>{f.value}</td>
                  <td className="muted">
                    {f.status === "confirmed" ? "settled" : f.status}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <div className="card-actions">
            <button
              type="button"
              className="wb-btn wb-btn--small"
              onClick={() => api.openNote(`entity:${card.entity.id}`)}
            >
              Open note
            </button>
          </div>
        </div>
      );
    case "graph":
      return (
        <div className="agent-card">
          <header>{card.note.title}</header>
          <p className="muted small">Links to</p>
          <div className="chip-row">
            {card.out.map((n) => (
              <button
                key={n.id}
                type="button"
                className="ctx-chip"
                onClick={() => api.openNote(n.id)}
              >
                {n.title}
              </button>
            ))}
          </div>
          <p className="muted small">Linked from</p>
          <div className="chip-row">
            {card.back.map((n) => (
              <button
                key={n.id}
                type="button"
                className="ctx-chip"
                onClick={() => api.openNote(n.id)}
              >
                {n.title}
              </button>
            ))}
          </div>
        </div>
      );
    case "source":
      return (
        <div className="agent-card">
          <header>
            {card.note.title} <small className="muted">· extractive</small>
          </header>
          <ul>
            {card.points.map((p, i) => (
              <li key={i}>{p}</li>
            ))}
          </ul>
          {card.links.length > 0 && (
            <p className="muted small">Links: {card.links.join(", ")}</p>
          )}
          <div className="card-actions">
            <button
              type="button"
              className="wb-btn wb-btn--small"
              onClick={() => api.openNote(card.note.id)}
            >
              Open note
            </button>
          </div>
        </div>
      );
    case "timeline":
      return (
        <div className="agent-card">
          <header>Timeline</header>
          <table className="wb-table compact">
            <tbody>
              {card.events.map((e) => (
                <tr key={e.id}>
                  <td className="mono">{e.tick ?? "?"}</td>
                  <td>{e.title}</td>
                  <td className="muted">
                    {card.titles[e.document_id ?? ""] ?? ""}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
    default:
      return null;
  }
}
