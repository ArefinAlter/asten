/**
 * The page's side of the assistant. Everything that touches a key, a provider
 * or the spend ledger happens in the desktop shell's Rust process; this module
 * only calls its commands (`ai_*`) and mirrors their types. A key is typed once
 * into `setKey`, which sends it to Rust and forgets it: no command returns one,
 * and nothing here stores one (no state, no storage, no log).
 *
 * The hosted pre-alpha and the browser desk have no such process, so there
 * `aiAvailable()` is false and the assistant answers from Asten's own tools.
 */
import { useSyncExternalStore } from "react";
import { isTauri } from "../../lib/asten-client";

export type Access = "read" | "propose";

export interface ConsentView {
  accepted_at: string;
  host: string;
}

export interface ProviderView {
  id: string;
  name: string;
  kind: "local" | "cloud";
  protocol: "anthropic" | "openai" | "gemini";
  family: string;
  model: string;
  endpoint: string;
  host: string;
  enabled: boolean;
  consent: ConsentView | null;
  consent_valid: boolean;
  key_stored: boolean;
  key_last4: string | null;
  tool_mode: "native" | "json";
  max_output_tokens: number;
  suggested_models: string[];
  /** Nothing leaves this machine when this provider is used. */
  local: boolean;
  price: { input_per_mtok: number; output_per_mtok: number } | null;
  /** Why it cannot be used now; null when it is ready. */
  not_ready: string | null;
  custom: boolean;
  /** CLI discovery only; no sign-in or entitlement claim. */
  installed?: boolean | null;
}

export interface Caps {
  daily_tokens: number;
  thread_tokens: number;
  daily_usd: number;
}

export interface Totals {
  day_tokens: number;
  day_input: number;
  day_output: number;
  day_usd: number;
  day_calls: number;
  thread_tokens: number;
  thread_usd: number;
  day_has_unpriced: boolean;
}

export interface VaultState {
  backend: "os" | "file" | "memory";
  locked: boolean;
  needs_setup: boolean;
  note: string | null;
}

export interface AiTool {
  name: string;
  title: string;
  description: string;
  open: boolean;
  closed_because: string;
}

export interface AiStatus {
  enabled: boolean;
  offline: boolean;
  local_first: boolean;
  caps: Caps;
  providers: ProviderView[];
  vault: VaultState;
  totals: Totals;
  notice_version: number;
  rules: string[];
  tools: AiTool[];
}

export interface SkillRef {
  id: string;
  name: string;
  tools: string[];
  body: string;
}

export type Attachment =
  | { kind: "mark"; id: string; label: string }
  | { kind: "scene"; id: string; label: string }
  | { kind: "note"; id: string; label: string }
  | {
      kind: "selection";
      document_id: string | null;
      label: string;
      text: string;
    };

export interface RunRequest {
  run_id: string;
  thread_id: string;
  provider: string | null;
  project_id: string;
  access: Access;
  disabled_tools: string[];
  skill?: SkillRef | null;
  text: string;
  history: { role: "user" | "assistant"; text: string }[];
  attachments: Attachment[];
  share_quotes?: boolean;
}

export interface Preview {
  refused: string | null;
  provider_id: string;
  provider_name: string;
  model: string;
  host: string;
  local: boolean;
  url: string;
  headers: [string, string][];
  body: string;
  bytes: number;
  sha256: string;
  tokens_estimate: number;
  contains_manuscript_text: boolean;
  passages: { label: string; chars: number }[];
  tools_offered: string[];
  json_protocol: boolean;
  notes: string[];
}

export interface RunSummary {
  text: string;
  requests: number;
  tool_calls: number;
  refused: boolean;
  withheld: boolean;
}

export type CardDesc =
  | { type: "marks"; ids: string[] }
  | { type: "explain"; mark_id: string }
  | { type: "canon"; entity_id: string }
  | { type: "graph"; note_id: string }
  | { type: "source"; note_id: string }
  | { type: "timeline" }
  | {
      type: "proposed_facts";
      items: {
        entity: string;
        entity_key: string;
        entity_id: string | null;
        predicate: string;
        value: string;
        reason: string | null;
        exists: boolean;
      }[];
    };

export type AiEvent =
  | {
      type: "started";
      run_id: string;
      provider: string;
      name: string;
      model: string;
      host: string;
      local: boolean;
    }
  | { type: "text"; delta: string }
  | {
      type: "tool_call";
      call_id: string;
      tool: string;
      input: unknown;
      allowed: boolean;
      reason: string | null;
    }
  | {
      type: "tool_result";
      call_id: string;
      tool: string;
      summary: string;
      ms: number;
      is_error: boolean;
    }
  | { type: "card"; card: CardDesc }
  | {
      type: "request";
      n: number;
      provider: string;
      host: string;
      bytes: number;
      sha256: string;
      tokens_estimate: number;
      body: string;
    }
  | {
      type: "usage";
      input: number;
      output: number;
      cost_usd: number | null;
      estimated: boolean;
      totals: Totals;
    }
  | { type: "notice"; message: string }
  | { type: "refused"; matched: string; tool: string; reply: string }
  | { type: "final"; text: string; withheld: boolean }
  | { type: "error"; message: string }
  | { type: "done"; run_id: string; requests: number; tool_calls: number };

export interface ProviderPatch {
  id: string;
  name?: string;
  model?: string;
  endpoint?: string;
  enabled?: boolean;
  tool_mode?: "native" | "json";
  max_output_tokens?: number;
}

export interface Patch {
  enabled?: boolean;
  offline?: boolean;
  local_first?: boolean;
  caps?: Caps;
  order?: string[];
  prices?: Record<
    string,
    { input_per_mtok: number; output_per_mtok: number } | null
  >;
  providers?: ProviderPatch[];
}

export interface LedgerView {
  totals: Totals;
  recent: {
    at: string;
    run_id: string;
    thread_id: string;
    provider: string;
    model: string;
    input_tokens: number;
    output_tokens: number;
    cost_usd: number | null;
    estimated: boolean;
  }[];
}

async function call<T>(command: string, args?: Record<string, unknown>) {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<T>(command, args);
}

/** True in the desktop shell, where the Rust side of the assistant exists. */
export function aiAvailable(): boolean {
  return isTauri();
}

// ── one cached status, shared by the settings screen and the panel ──

interface Store {
  status?: AiStatus;
  error?: string;
  loaded: boolean;
}

let store: Store = { loaded: false };
const listeners = new Set<() => void>();
let inflight: Promise<void> | undefined;

function publish(next: Store) {
  store = next;
  listeners.forEach((l) => l());
}

/** Take a status that a command returned. */
export function applyStatus(status: AiStatus) {
  publish({ status, loaded: true });
}

export function refreshStatus(threadId?: string): Promise<void> {
  if (!aiAvailable()) {
    publish({ loaded: true, error: "not available" });
    return Promise.resolve();
  }
  inflight ??= call<AiStatus>("ai_status", { threadId: threadId ?? null })
    .then((status) => publish({ status, loaded: true }))
    .catch((error: unknown) =>
      publish({
        loaded: true,
        error: error instanceof Error ? error.message : String(error),
      }),
    )
    .finally(() => {
      inflight = undefined;
    });
  return inflight;
}

export function useAi(): Store {
  const snapshot = useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => store,
    () => store,
  );
  if (!snapshot.loaded && !inflight) void refreshStatus();
  return snapshot;
}

async function viaStatus(
  command: string,
  args?: Record<string, unknown>,
): Promise<AiStatus> {
  const status = await call<AiStatus>(command, args);
  applyStatus(status);
  return status;
}

export const ai = {
  update: (patch: Patch) => viaStatus("ai_update", { patch }),
  addProvider: (name: string, endpoint: string, model: string) =>
    viaStatus("ai_add_provider", { name, endpoint, model }),
  removeProvider: (id: string) => viaStatus("ai_remove_provider", { id }),
  /** Send a key to the shell. Nothing comes back but the status. */
  setKey: (provider: string, key: string) =>
    viaStatus("ai_set_key", { provider, key }),
  forgetKey: (provider: string) => viaStatus("ai_forget_key", { provider }),
  unlock: (passphrase: string) => viaStatus("ai_unlock", { passphrase }),
  consent: (provider: string, accept: boolean) =>
    viaStatus("ai_consent", { provider, accept }),
  withdrawAll: () => viaStatus("ai_withdraw_all"),
  preview: (request: RunRequest) => call<Preview>("ai_preview", { request }),
  cancel: (runId: string) => call<void>("ai_cancel", { runId }),
  test: (provider: string) => call<string>("ai_test", { provider }),
  localModels: (provider: string) =>
    call<string[]>("ai_local_models", { provider }),
  ledger: (threadId?: string) =>
    call<LedgerView>("ai_ledger", { threadId: threadId ?? null }),
  clearLedger: () => call<void>("ai_clear_ledger"),
  /** Run a request; events arrive on `onEvent` as the shell produces them. */
  async run(
    request: RunRequest,
    onEvent: (event: AiEvent) => void,
  ): Promise<RunSummary> {
    const { invoke, Channel } = await import("@tauri-apps/api/core");
    const channel = new Channel<AiEvent>();
    channel.onmessage = onEvent;
    return invoke<RunSummary>("ai_run", { request, onEvent: channel });
  },
};

export function newRunId(): string {
  return `run-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

// ── the live turn: events folded into what the panel shows ──

export interface ModelCall {
  id: string;
  tool: string;
  input: unknown;
  allowed: boolean;
  reason?: string;
  summary?: string;
  ms?: number;
  isError?: boolean;
}

export interface RequestInfo {
  n: number;
  host: string;
  bytes: number;
  sha256: string;
  tokens: number;
  /** The exact body; kept in memory for the visit, never saved with the thread. */
  body?: string;
}

export interface ModelTurn {
  provider?: string;
  providerName?: string;
  model?: string;
  host?: string;
  local?: boolean;
  calls: ModelCall[];
  requests: RequestInfo[];
  cards: CardDesc[];
  usage?: {
    input: number;
    output: number;
    costUsd: number | null;
    estimated: boolean;
  };
  notices: string[];
  refused?: { matched: string; tool: string };
  withheld?: boolean;
  error?: string;
  done?: boolean;
}

export const emptyTurn = (): ModelTurn => ({
  calls: [],
  requests: [],
  cards: [],
  notices: [],
});

/** Fold one event into the live turn and its text. Pure. */
export function applyEvent(
  turn: ModelTurn,
  text: string,
  event: AiEvent,
): { turn: ModelTurn; text: string } {
  switch (event.type) {
    case "started":
      return {
        turn: {
          ...turn,
          provider: event.provider,
          providerName: event.name,
          model: event.model,
          host: event.host,
          local: event.local,
        },
        text,
      };
    case "text":
      return { turn, text: text + event.delta };
    case "tool_call": {
      const existing = turn.calls.findIndex((c) => c.id === event.call_id);
      const call: ModelCall = {
        id: event.call_id,
        tool: event.tool,
        input: event.input,
        allowed: event.allowed,
        reason: event.reason ?? undefined,
      };
      const calls =
        existing >= 0
          ? turn.calls.map((c, i) => (i === existing ? { ...c, ...call } : c))
          : [...turn.calls, call];
      return { turn: { ...turn, calls }, text };
    }
    case "tool_result":
      return {
        turn: {
          ...turn,
          calls: turn.calls.map((c) =>
            c.id === event.call_id
              ? {
                  ...c,
                  summary: event.summary,
                  ms: event.ms,
                  isError: event.is_error,
                }
              : c,
          ),
        },
        text,
      };
    case "card":
      return { turn: { ...turn, cards: [...turn.cards, event.card] }, text };
    case "request":
      return {
        turn: {
          ...turn,
          requests: [
            ...turn.requests,
            {
              n: event.n,
              host: event.host,
              bytes: event.bytes,
              sha256: event.sha256,
              tokens: event.tokens_estimate,
              body: event.body,
            },
          ],
        },
        text,
      };
    case "usage":
      return {
        turn: {
          ...turn,
          usage: {
            input: (turn.usage?.input ?? 0) + event.input,
            output: (turn.usage?.output ?? 0) + event.output,
            costUsd:
              event.cost_usd === null && turn.usage?.costUsd == null
                ? null
                : (turn.usage?.costUsd ?? 0) + (event.cost_usd ?? 0),
            estimated: (turn.usage?.estimated ?? false) || event.estimated,
          },
        },
        text,
      };
    case "notice":
      return {
        turn: { ...turn, notices: [...turn.notices, event.message] },
        text,
      };
    case "refused":
      return {
        turn: {
          ...turn,
          refused: { matched: event.matched, tool: event.tool },
        },
        text: event.reply,
      };
    case "final":
      return {
        turn: { ...turn, withheld: event.withheld || undefined },
        text: event.text,
      };
    case "error":
      return { turn: { ...turn, error: event.message }, text };
    case "done":
      return { turn: { ...turn, done: true }, text };
  }
}

/** What a thread keeps: the turn without the request bodies. */
export function forStorage(turn: ModelTurn): ModelTurn {
  return {
    ...turn,
    requests: turn.requests.map(({ body: _body, ...rest }) => rest),
  };
}

/** A short, honest cost line. */
export function spendLine(totals: Totals, caps: Caps): string {
  const tokens = totals.day_tokens.toLocaleString("en");
  const cap = caps.daily_tokens.toLocaleString("en");
  const dollars = totals.day_has_unpriced
    ? `${totals.day_usd.toFixed(4)} + unpriced`
    : totals.day_usd.toFixed(4);
  return `Today ${tokens} of ${cap} tokens · about $${dollars}`;
}
