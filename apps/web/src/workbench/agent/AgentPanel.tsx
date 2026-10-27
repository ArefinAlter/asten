import { useQueryClient } from "@tanstack/react-query";
import type { Editor } from "@tiptap/react";
import { useEffect, useMemo, useRef, useState } from "react";
import type { AstenDiagnostic } from "@asten/contracts";
import { useWorkbench } from "../context";
import { Icon } from "../icons";
import { buildVault } from "../knowledge/vault";
import { log } from "../output";
import { deskStore } from "../../lib/storage";
import { setPrefs, usePrefs } from "../prefs";
import {
  ai,
  aiAvailable,
  applyEvent,
  emptyTurn,
  forStorage,
  newRunId,
  refreshStatus,
  spendLine,
  useAi,
  type Attachment,
  type ModelTurn,
  type Preview,
  type RunRequest,
} from "../ai/bridge";
import { PreviewDialog } from "../ai/PreviewDialog";
import { resolveCard } from "./cards";
import { respond, type Card, type Chip, type Turn } from "./script";
import { SKILLS, type Skill } from "./skills";
import type { Access } from "./tools";

interface Message {
  id: string;
  role: "person" | "assistant";
  text: string;
  chips?: Chip[];
  /** An answer from Asten's own tools. */
  turn?: Turn;
  /** An answer from a model, with what it called and what was sent. */
  model?: ModelTurn;
  at: string;
}

interface Thread {
  id: string;
  title: string;
  access: Access;
  messages: Message[];
}

const THREADS_KEY = "asten-workbench:threads:v1";
const MAX_PASSAGE = 8000;

let serial = 0;
const id = () => `t${Date.now().toString(36)}${(serial += 1).toString(36)}`;

export const FIXED_RULES = [
  "Never writes, rewrites, continues or autocompletes your prose.",
  "Never settles canon or applies a repair by itself: you press every card.",
  "Never raises its own access, changes settings or reads provider keys.",
  "Summarises source and research notes only, never manuscript scenes.",
  "Every tool call is listed, including the ones refused.",
];

function describe(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "Something went wrong";
}

/** The writer's selection in the page, as text, while one exists. */
function useSelection(editor: Editor | null) {
  const [selection, setSelection] = useState<string>("");
  useEffect(() => {
    if (!editor) {
      setSelection("");
      return;
    }
    const update = () => {
      const { from, to, empty } = editor.state.selection;
      if (empty || to - from < 3) return;
      const text = editor.state.doc.textBetween(from, to, "\n", " ").trim();
      if (text) setSelection(text.slice(0, MAX_PASSAGE));
    };
    editor.on("selectionUpdate", update);
    update();
    return () => {
      editor.off("selectionUpdate", update);
    };
  }, [editor]);
  return selection;
}

export function AgentPanel() {
  const api = useWorkbench();
  const prefs = usePrefs();
  const queryClient = useQueryClient();
  const desktopAi = aiAvailable();
  const { status } = useAi();
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
  // service): the newest twenty, eighty messages each. Request bodies are
  // never kept: they can hold a passage the writer picked.
  useEffect(() => {
    deskStore.setJson(
      THREADS_KEY,
      threads.slice(-20).map((t) => ({
        ...t,
        messages: t.messages
          .slice(-80)
          .map((m) => (m.model ? { ...m, model: forStorage(m.model) } : m)),
      })),
    );
  }, [threads]);
  const [activeId, setActiveId] = useState(threads[0].id);
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const [chips, setChips] = useState<Chip[]>([]);
  const [showRules, setShowRules] = useState(false);
  const [showThreads, setShowThreads] = useState(false);
  const [live, setLive] = useState<{ text: string; turn: ModelTurn }>();
  const [preview, setPreview] = useState<Preview>();
  const [notice, setNotice] = useState<string>();
  const runId = useRef<string | undefined>(undefined);
  const scroller = useRef<HTMLDivElement>(null);
  const thread = threads.find((t) => t.id === activeId) ?? threads[0];
  const selection = useSelection(api.editor);

  // Which model answers: Asten's own tools, or a provider the writer chose.
  const usable = status?.providers.filter((p) => p.enabled) ?? [];
  const picked =
    desktopAi && prefs.agentModel !== "rules" && status
      ? prefs.agentModel === "auto" ||
        usable.some((p) => p.id === prefs.agentModel)
        ? prefs.agentModel
        : "rules"
      : "rules";
  const modelProvider = picked === "auto" ? null : picked;
  const usingModel = picked !== "rules";

  // Chips the person can attach: the open scene, the selected mark, the open
  // note, and the words they have selected (only they can pick a passage).
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
    if (selection && usingModel)
      out.push({
        kind: "selection",
        id: "selection",
        label: `Selected passage · ${selection.length} characters`,
        text: selection,
        documentId: doc?.id,
      });
    return out.filter((c) => !chips.some((x) => x.id === c.id));
  }, [
    api.documents,
    api.openDocumentId,
    api.diagnostics,
    api.selectedMarkId,
    api.selectedNoteId,
    selection,
    usingModel,
    chips,
  ]);

  useEffect(() => {
    scroller.current?.scrollTo({ top: scroller.current.scrollHeight });
  }, [thread.messages.length, busy, live?.text]);

  const update = (patch: Partial<Thread>) =>
    setThreads((all) =>
      all.map((t) => (t.id === thread.id ? { ...t, ...patch } : t)),
    );

  const runRequest = (text: string, skill?: Skill): RunRequest => {
    const attachments: Attachment[] = chips.map((chip): Attachment => {
      switch (chip.kind) {
        case "mark":
          return { kind: "mark", id: chip.id, label: chip.label };
        case "scene":
          return { kind: "scene", id: chip.id, label: chip.label };
        case "note":
          return { kind: "note", id: chip.id, label: chip.label };
        default:
          return {
            kind: "selection",
            document_id: chip.documentId ?? null,
            label:
              api.documents.find((d) => d.id === chip.documentId)?.title ??
              "the open page",
            text: chip.text ?? "",
          };
      }
    });
    return {
      run_id: runId.current ?? newRunId(),
      thread_id: thread.id,
      provider: modelProvider,
      project_id: api.projectId!,
      access: thread.access,
      disabled_tools: prefs.disabledTools,
      skill: skill
        ? {
            id: skill.id,
            name: skill.name,
            tools: skill.tools,
            body: prefs.skills[skill.id]?.body ?? skill.body,
          }
        : null,
      text,
      history: thread.messages
        .filter((m) => m.text.trim())
        .slice(-8)
        .map((m) => ({
          role: m.role === "person" ? "user" : "assistant",
          text: m.text,
        })),
      attachments,
    };
  };

  const skillFor = (text: string): Skill | undefined =>
    SKILLS.find(
      (s) => s.prompt === text && prefs.skills[s.id]?.enabled !== false,
    );

  async function sendToModel(trimmed: string) {
    const person: Message = {
      id: id(),
      role: "person",
      text: trimmed,
      chips,
      at: new Date().toISOString(),
    };
    const title = thread.messages.length ? thread.title : trimmed.slice(0, 48);
    const threadId = thread.id;
    const request = runRequest(trimmed, skillFor(trimmed));
    setThreads((all) =>
      all.map((t) =>
        t.id === threadId
          ? { ...t, title, messages: [...t.messages, person] }
          : t,
      ),
    );
    setDraft("");
    setBusy(true);
    setNotice(undefined);
    runId.current = request.run_id;
    let turn = emptyTurn();
    let text = "";
    setLive({ text, turn });
    let frame = 0;
    const flush = () => {
      frame = 0;
      setLive({ text, turn });
    };
    try {
      await ai.run(request, (event) => {
        ({ turn, text } = applyEvent(turn, text, event));
        if (!frame) frame = requestAnimationFrame(flush);
      });
    } catch (failure) {
      turn = { ...turn, error: turn.error ?? describe(failure) };
    } finally {
      if (frame) cancelAnimationFrame(frame);
    }
    for (const call of turn.calls)
      log(
        "Assistant",
        `${call.allowed ? "called" : "refused"} ${call.tool}(${JSON.stringify(call.input)})${call.reason ? ` — ${call.reason}` : call.summary ? ` → ${call.summary}` : ""}`,
        call.allowed ? "info" : "warn",
      );
    const reply: Message = {
      id: id(),
      role: "assistant",
      text: text || (turn.error ? "" : "The model sent no text."),
      model: turn,
      at: new Date().toISOString(),
    };
    setThreads((all) =>
      all.map((t) =>
        t.id === threadId ? { ...t, messages: [...t.messages, reply] } : t,
      ),
    );
    if (turn.error) setNotice(turn.error);
    setLive(undefined);
    setChips([]);
    setBusy(false);
    runId.current = undefined;
    void refreshStatus(threadId);
    void queryClient.invalidateQueries({
      queryKey: ["diagnostics", api.projectId],
    });
  }

  async function showPreview() {
    const text = draft.trim();
    if (!text || !api.projectId) return;
    try {
      setPreview(await ai.preview(runRequest(text, skillFor(text))));
    } catch (failure) {
      setNotice(describe(failure));
    }
  }

  async function send(text = draft) {
    const trimmed = text.trim();
    if (!trimmed || busy) return;
    if (!api.projectId) {
      api.toast("Open a project first.");
      return;
    }
    if (usingModel) {
      await sendToModel(trimmed);
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

  const readyReason =
    usingModel && status
      ? picked === "auto"
        ? usable.some((p) => !p.not_ready) && status.enabled
          ? undefined
          : status.enabled
            ? "No provider is ready."
            : "AI providers are switched off in Settings."
        : (usable.find((p) => p.id === picked)?.not_ready ?? undefined)
      : undefined;

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
        {desktopAi && status && (
          <>
            <button
              type="button"
              className={`wb-icon-btn${status.offline ? " active" : ""}`}
              title={
                status.offline
                  ? "Offline mode is on: nothing leaves this machine. Click to turn it off."
                  : "Offline mode: block every cloud provider"
              }
              aria-label="Offline mode"
              aria-pressed={status.offline}
              onClick={() => void ai.update({ offline: !status.offline })}
            >
              <Icon name="globe" />
            </button>
            <button
              type="button"
              className="wb-icon-btn"
              title={`${spendLine(status.totals, status.caps)}. Open AI providers.`}
              aria-label="Spend today"
              onClick={() => api.openSettings("providers")}
            >
              <Icon name="scales" />
            </button>
          </>
        )}
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
        {thread.messages.length === 0 && !live ? (
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
                {message.model && <ModelLog model={message.model} />}
                <Prose text={message.text} />
                {message.turn?.cards.map((card, index) => (
                  <CardView key={index} card={card} access={thread.access} />
                ))}
                {message.model?.cards.map((desc, index) => (
                  <ResolvedCard
                    key={index}
                    desc={desc}
                    access={thread.access}
                  />
                ))}
                {message.model?.error && (
                  <p className="agent-error" role="alert">
                    {message.model.error}
                  </p>
                )}
              </div>
            ),
          )
        )}
        {live && (
          <div className="msg-assistant live">
            <ModelLog model={live.turn} running />
            {live.text ? (
              <Prose text={live.text} />
            ) : (
              <p className="muted agent-working">
                {live.turn.requests.length
                  ? "Waiting for the model…"
                  : "Starting…"}
              </p>
            )}
            {live.turn.cards.map((desc, index) => (
              <ResolvedCard key={index} desc={desc} access={thread.access} />
            ))}
          </div>
        )}
        {busy && !live && <p className="muted agent-working">Working…</p>}
      </div>
      <div className="composer">
        {notice && (
          <p className="agent-error" role="alert">
            {notice}{" "}
            <button
              type="button"
              className="linkish"
              onClick={() => api.openSettings("providers")}
            >
              AI providers
            </button>
          </p>
        )}
        {!notice && readyReason && (
          <p className="agent-error" role="status">
            {readyReason}{" "}
            <button
              type="button"
              className="linkish"
              onClick={() => api.openSettings("providers")}
            >
              Open AI providers
            </button>
          </p>
        )}
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
                title={
                  c.kind === "selection"
                    ? "Send these words to the model with your question. Nothing else from the page is sent."
                    : "Add to context"
                }
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
            value={picked}
            aria-label="Model"
            onChange={(e) => {
              if (e.target.value === "settings") api.openSettings("providers");
              else setPrefs({ agentModel: e.target.value });
            }}
            title={
              desktopAi
                ? "Who answers: Asten's own tools, or a provider you switched on"
                : "No model is called in this copy"
            }
          >
            <option value="rules">Asten rules · no model</option>
            {desktopAi && usable.length > 0 && (
              <option value="auto">Auto · first ready provider</option>
            )}
            {desktopAi &&
              usable.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.name} · {p.model}
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
          {usingModel && (
            <button
              type="button"
              className="wb-icon-btn"
              aria-label="Show what will be sent"
              title="Show exactly what will be sent to the provider"
              disabled={!draft.trim() || busy}
              onClick={() => void showPreview()}
            >
              <Icon name="eye" />
            </button>
          )}
          {busy && usingModel ? (
            <button
              type="button"
              className="send-btn"
              aria-label="Stop"
              title="Stop this request"
              onClick={() => {
                if (runId.current) void ai.cancel(runId.current);
              }}
            >
              <Icon name="stop" />
            </button>
          ) : (
            <button
              type="button"
              className="send-btn"
              aria-label="Send"
              disabled={!draft.trim() || busy}
              onClick={() => void send()}
            >
              <Icon name="send" />
            </button>
          )}
        </div>
        {usingModel && status && (
          <p className="composer-meter muted">
            {status.offline
              ? "Offline mode: only a model on this computer can answer."
              : spendLine(status.totals, status.caps)}
          </p>
        )}
      </div>
      {preview && (
        <PreviewDialog
          preview={preview}
          onClose={() => setPreview(undefined)}
          onSend={() => {
            setPreview(undefined);
            void send();
          }}
        />
      )}
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

function truncate(text: string, max: number) {
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

/**
 * What a model run did: who answered, every request that left (with its size
 * and digest, and the exact body on demand), every tool it called, including
 * the refusals, and what it cost.
 */
function ModelLog({ model, running }: { model: ModelTurn; running?: boolean }) {
  const [open, setOpen] = useState(Boolean(running));
  const refused = model.calls.filter((c) => !c.allowed).length;
  if (
    !model.requests.length &&
    !model.calls.length &&
    !model.refused &&
    !model.notices.length
  )
    return null;
  const bytes = model.requests.reduce((sum, r) => sum + r.bytes, 0);
  return (
    <div className={`tool-log model-log${refused ? " has-refused" : ""}`}>
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
      >
        <Icon name={open ? "chevronDown" : "chevronRight"} size={12} />
        {model.refused
          ? "Refused before any request"
          : `${model.providerName ?? "Model"} · ${model.model ?? ""} · ${model.requests.length} request${model.requests.length === 1 ? "" : "s"}`}
        {model.calls.length
          ? ` · ${model.calls.length} tool call${model.calls.length === 1 ? "" : "s"}`
          : ""}
        {refused ? ` · ${refused} refused` : ""}
        {model.usage
          ? ` · ${model.usage.input + model.usage.output} tokens${model.usage.costUsd != null ? ` · about $${model.usage.costUsd.toFixed(4)}` : ""}`
          : ""}
      </button>
      {open && (
        <div className="model-log-body">
          {model.refused && (
            <p>
              Asked for prose (“{model.refused.matched}”), which would need{" "}
              <code>{model.refused.tool}</code>. That tool is closed, so no
              provider was contacted.
            </p>
          )}
          {model.host && (
            <p className="muted small">
              {model.local
                ? "Answered by a model on this computer; nothing left it."
                : `Requests went from this computer to ${model.host}.`}
            </p>
          )}
          {model.notices.map((notice, index) => (
            <p key={index} className="muted small">
              {notice}
            </p>
          ))}
          {model.requests.length > 0 && (
            <ol className="request-list">
              {model.requests.map((request) => (
                <li key={request.n}>
                  Request {request.n} · {request.host} ·{" "}
                  {request.bytes.toLocaleString("en")} bytes · sha256{" "}
                  <code>{request.sha256.slice(0, 12)}…</code>
                  {request.body && (
                    <details>
                      <summary>The exact body</summary>
                      <pre className="preview-body">{request.body}</pre>
                    </details>
                  )}
                </li>
              ))}
            </ol>
          )}
          {bytes > 0 && (
            <p className="muted small">
              {bytes.toLocaleString("en")} bytes sent in all.
            </p>
          )}
          {model.calls.length > 0 && (
            <ol>
              {model.calls.map((call) => (
                <li key={call.id} className={call.allowed ? "" : "refused"}>
                  <code>{call.tool}</code>{" "}
                  <span className="muted">
                    {truncate(JSON.stringify(call.input ?? {}), 120)}
                  </span>
                  <div>
                    {call.allowed ? (
                      <>
                        → {call.summary ?? "running…"}
                        {call.ms ? ` · ${call.ms} ms` : ""}
                      </>
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
          {model.withheld && (
            <p className="muted small">
              The reply was withheld because it read like new prose for your
              story.
            </p>
          )}
        </div>
      )}
    </div>
  );
}

/** A safe subset of Markdown: paragraphs, lists, **bold** and `code`. No HTML, no links. */
function Prose({ text }: { text: string }) {
  const inline = (line: string) =>
    line
      .split(/(\*\*[^*]+\*\*|`[^`]+`)/)
      .map((part, j) =>
        part.startsWith("**") && part.endsWith("**") && part.length > 4 ? (
          <strong key={j}>{part.slice(2, -2)}</strong>
        ) : part.startsWith("`") && part.endsWith("`") && part.length > 2 ? (
          <code key={j}>{part.slice(1, -1)}</code>
        ) : (
          <span key={j}>{part}</span>
        ),
      );
  const bullet = /^\s*([-*•]|\d+[.)])\s+/;
  return (
    <div className="prose-reply">
      {text.split(/\n\n+/).map((para, i) => {
        const lines = para.split("\n");
        if (lines.every((l) => bullet.test(l)))
          return (
            <ul key={i}>
              {lines.map((l, k) => (
                <li key={k}>{inline(l.replace(bullet, ""))}</li>
              ))}
            </ul>
          );
        return (
          <p key={i}>
            {lines.map((l, k) => (
              <span key={k}>
                {k > 0 && <br />}
                {inline(l)}
              </span>
            ))}
          </p>
        );
      })}
    </div>
  );
}

/** A card the shell described by ids, drawn from the desk's own data. */
function ResolvedCard({
  desc,
  access,
}: {
  desc: Parameters<typeof resolveCard>[0];
  access: Access;
}) {
  const api = useWorkbench();
  const queryClient = useQueryClient();
  const [card, setCard] = useState<Card>();
  useEffect(() => {
    let alive = true;
    if (!api.projectId) return;
    void resolveCard(desc, {
      client: api.client,
      projectId: api.projectId,
      queryClient,
      access,
    })
      .then((c) => {
        if (alive) setCard(c);
      })
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [desc, api.client, api.projectId, queryClient, access]);
  return card ? <CardView card={card} access={access} /> : null;
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
    case "proposed":
      return (
        <div className="agent-card">
          <header>Proposed facts · you decide</header>
          {card.items.map((item, index) => {
            const key = `${item.entity_key}/${item.predicate}/${index}`;
            const add = async (status: "confirmed" | "proposed") => {
              try {
                await api.client.createFact({
                  project_id: api.projectId!,
                  entity_key: item.entity_key,
                  predicate: item.predicate,
                  value: item.value,
                  status,
                });
                setDone((d) => ({
                  ...d,
                  [key]:
                    status === "confirmed"
                      ? "settled by you"
                      : "kept as proposal",
                }));
                if (status === "confirmed")
                  log(
                    "Checks",
                    `Fact settled by you: ${item.entity} ${item.predicate} = ${item.value}`,
                  );
                await refresh();
              } catch (error) {
                api.toast(describe(error), "error");
              }
            };
            return (
              <div key={key} className="card-row static">
                <span className="card-row-main">
                  <strong>{item.entity}</strong> {item.predicate} ={" "}
                  <em>{item.value}</em>
                  {item.reason && (
                    <small className="muted"> · {item.reason}</small>
                  )}
                </span>
                {done[key] ? (
                  <span className="wb-pill">{done[key]}</span>
                ) : item.exists ? (
                  <span className="wb-pill">already in the canon</span>
                ) : !item.entity_id ? (
                  <span className="muted small">
                    not in the cast yet; add the name first
                  </span>
                ) : access === "propose" ? (
                  <>
                    <button
                      type="button"
                      className="wb-btn wb-btn--small wb-btn--primary"
                      onClick={() => void add("confirmed")}
                    >
                      Settle
                    </button>
                    <button
                      type="button"
                      className="wb-btn wb-btn--small"
                      onClick={() => void add("proposed")}
                    >
                      Keep as proposal
                    </button>
                    <button
                      type="button"
                      className="wb-btn wb-btn--small wb-btn--ghost"
                      onClick={() =>
                        setDone((d) => ({ ...d, [key]: "dismissed" }))
                      }
                    >
                      Dismiss
                    </button>
                  </>
                ) : (
                  <span className="muted">proposed</span>
                )}
              </div>
            );
          })}
        </div>
      );
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
            {card.note.title}{" "}
            <small className="muted">
              · {card.points.length ? "opening lines" : "note"}
            </small>
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
