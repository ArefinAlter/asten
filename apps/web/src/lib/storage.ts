/**
 * Where the desk keeps its own small state (preferences, layout, assistant
 * threads, console history, folded binder nodes). Three builds, three homes:
 *
 * - Desktop and local builds: this webview's `localStorage`, on this device.
 * - The hosted pre-alpha (`VITE_ASTEN_HOSTED=1`): the hosting service's
 *   `/svc/v1/state`, kept with the visitor's sandbox and deleted with it. The
 *   values are read once before the desk renders and written through after.
 *   Nothing goes to the browser's storage.
 * - The offline sample build (`VITE_ASTEN_SAMPLE=1`): memory only.
 *
 * Provider keys are never stored by any of them (see `workbench/secrets.ts`).
 */

const HOSTED = import.meta.env.VITE_ASTEN_HOSTED === "1";
const SAMPLE = import.meta.env.VITE_ASTEN_SAMPLE === "1";

/** True on the hosted pre-alpha, where a service holds the state. */
export function isHosted(): boolean {
  return HOSTED;
}

/** Whether this build may use the browser's own storage (desktop and local). */
export function storageAllowed(): boolean {
  return !HOSTED && !SAMPLE;
}

const memory = new Map<string, string>();
const pending = new Map<string, string | null>();
let flushTimer: ReturnType<typeof setTimeout> | undefined;

function flush() {
  flushTimer = undefined;
  const batch = [...pending.entries()];
  pending.clear();
  for (const [key, value] of batch) {
    const url = `/svc/v1/state/${encodeURIComponent(key)}`;
    const request: RequestInit =
      value === null
        ? { method: "DELETE", credentials: "same-origin" }
        : {
            method: "PUT",
            credentials: "same-origin",
            headers: { "Content-Type": "application/json" },
            // Stored as a JSON string so any value round-trips exactly.
            body: JSON.stringify(value),
            keepalive: value.length < 60_000,
          };
    void fetch(url, request).catch(() => undefined);
  }
}

function schedule(key: string, value: string | null) {
  pending.set(key, value);
  if (flushTimer === undefined) flushTimer = setTimeout(flush, 400);
}

if (HOSTED && typeof window !== "undefined") {
  // Writes queued in the last moments before the tab closes still go out.
  window.addEventListener("pagehide", () => {
    if (flushTimer !== undefined) {
      clearTimeout(flushTimer);
      flush();
    }
  });
}

/** Read the hosted state into memory. Call once before the first render. */
export async function hydrateHostedState(): Promise<void> {
  if (!HOSTED) return;
  try {
    const response = await fetch("/svc/v1/state", {
      credentials: "same-origin",
    });
    if (!response.ok) return;
    const body = (await response.json()) as { items?: Record<string, unknown> };
    for (const [key, value] of Object.entries(body.items ?? {})) {
      if (typeof value === "string") memory.set(key, value);
    }
  } catch {
    // The desk still opens; state holds for this visit.
  }
}

/** The desk's key-value store; see the module comment for where it lives. */
export const deskStore = {
  get(key: string): string | null {
    if (storageAllowed()) {
      try {
        return localStorage.getItem(key);
      } catch {
        return null;
      }
    }
    return memory.get(key) ?? null;
  },
  set(key: string, value: string | null) {
    if (storageAllowed()) {
      try {
        if (value === null) localStorage.removeItem(key);
        else localStorage.setItem(key, value);
      } catch {
        // Fine without storage.
      }
      return;
    }
    if (value === null) memory.delete(key);
    else memory.set(key, value);
    if (HOSTED) schedule(key, value);
  },
  getJson<T>(key: string): T | undefined {
    const raw = this.get(key);
    if (raw === null) return undefined;
    try {
      return JSON.parse(raw) as T;
    } catch {
      return undefined;
    }
  },
  setJson(key: string, value: unknown) {
    this.set(key, JSON.stringify(value));
  },
};

/** What the hosting service says about this visit (hosted build only). */
export interface HostedSession {
  created_at: string;
  expires_at: string;
  sandbox_bytes: number;
  limits: {
    ttl_hours: number;
    max_sandbox_bytes: number;
    max_state_total_bytes: number;
  };
  served_by: { api: string[]; service: string[]; browser_memory: string[] };
}

export async function hostedSession(): Promise<HostedSession> {
  const response = await fetch("/svc/v1/session", {
    credentials: "same-origin",
  });
  if (!response.ok) throw new Error(`session: ${response.status}`);
  return (await response.json()) as HostedSession;
}

/** `true` the first time this visitor claims `name`, `false` after. */
export async function hostedClaim(name: string): Promise<boolean> {
  const response = await fetch(`/svc/v1/claim/${encodeURIComponent(name)}`, {
    method: "POST",
    credentials: "same-origin",
  });
  if (!response.ok) return false;
  const body = (await response.json()) as { first?: boolean };
  return body.first === true;
}

/** Delete this visitor's sandbox and stored state, then reload the desk. */
export async function hostedStartOver(): Promise<void> {
  if (flushTimer !== undefined) clearTimeout(flushTimer);
  pending.clear();
  await fetch("/svc/v1/reset", { method: "POST", credentials: "same-origin" });
  window.location.hash = "";
  window.location.reload();
}
