import type {
  ApiErrorBody,
  ApplyRepairRequest,
  ApplyRepairResponse,
  AuthStatus,
  AuthenticatedClient,
  Baseline,
  BisectResult,
  BlameSpan,
  CanonEntity,
  CastSearchHit,
  CodeInfo,
  Comment,
  CompileArtifact,
  CompileRequest,
  CreateCanonEntity,
  CreateComment,
  CreateDocument,
  CreateFact,
  CreateInvariant,
  CreateProject,
  CreateSuppression,
  CreateTimelineEvent,
  CreateWorldEntry,
  DiagnosticsSnapshot,
  AstenDocument,
  DocumentSnapshot,
  Draft,
  EventRecord,
  Fact,
  FactCard,
  ImportRequest,
  MentionRef,
  MergePreview,
  MethodPack,
  PairingComplete,
  PairingStart,
  StreamTicket,
  Project,
  ProjectId,
  ProjectSettings,
  RenamePreview,
  Route,
  SceneSummary,
  StoredInvariant,
  StoredRoute,
  StreamEvent,
  Suppression,
  TimelineEvent,
  TokenView,
  TrackedChange,
  UpdateCanonEntity,
  UpdateComment,
  UpdateDocument,
  UpdateFact,
  UpdateSettings,
  UpdateTimelineEvent,
  UpdateWorldEntry,
  VerifyReport,
  WhereSettled,
  WorldEntry,
  WorldPulse,
  Invariant,
} from "@asten/contracts";

/**
 * One client for every surface. In the browser it talks to the loopback API
 * with a session cookie (or a bearer token for guests). Inside the desktop
 * shell it asks Tauri for the embedded API's origin and desk token, then uses
 * the very same HTTP paths, so the two transports cannot drift apart.
 */
export interface Session {
  origin: string;
  token: string | null;
}

const TOKEN_KEY = "asten-token";

function readStoredToken(): string | null {
  try {
    return localStorage.getItem(TOKEN_KEY);
  } catch {
    return null;
  }
}

export function storeToken(token: string | null) {
  try {
    if (token) localStorage.setItem(TOKEN_KEY, token);
    else localStorage.removeItem(TOKEN_KEY);
  } catch {
    // Storage may be unavailable; the cookie still carries the session.
  }
}

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export class ApiError extends Error {
  readonly status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function resolveSession(): Promise<Session> {
  if (isTauri()) {
    const { invoke } = await import("@tauri-apps/api/core");
    const desk = await invoke<{ origin: string; token: string }>(
      "desk_session",
    );
    return { origin: desk.origin, token: desk.token };
  }
  return { origin: "", token: readStoredToken() };
}

export class AstenClient {
  private session: Promise<Session>;
  private overrideToken: string | null | undefined;

  constructor(session?: Promise<Session>) {
    this.session = session ?? resolveSession();
  }

  /** Use a token obtained by pairing in this tab. */
  setToken(token: string | null) {
    this.overrideToken = token;
    storeToken(token);
  }

  async origin(): Promise<string> {
    return (await this.session).origin;
  }

  async token(): Promise<string | null> {
    if (this.overrideToken !== undefined) return this.overrideToken;
    return (await this.session).token;
  }

  // ── auth ──
  authStatus(): Promise<AuthStatus> {
    return this.request("/auth/status");
  }
  me(): Promise<AuthenticatedClient> {
    return this.request("/auth/me");
  }
  async pair(code: string, label: string): Promise<PairingComplete> {
    const paired = await this.request<PairingComplete>("/auth/pair", {
      method: "POST",
      body: JSON.stringify({ code, label }),
    });
    this.setToken(paired.token);
    return paired;
  }
  async login(passphrase: string): Promise<PairingComplete> {
    const session = await this.request<PairingComplete>("/auth/login", {
      method: "POST",
      body: JSON.stringify({ passphrase }),
    });
    this.setToken(session.token);
    return session;
  }
  async logout(): Promise<void> {
    await this.request<void>("/auth/logout", { method: "POST", body: "{}" });
    this.setToken(null);
  }
  setupOwner(passphrase: string): Promise<void> {
    return this.request("/auth/owner/setup", {
      method: "POST",
      body: JSON.stringify({ passphrase }),
    });
  }
  startPairing(
    label: string,
    grants: "guest" | "desk" = "guest",
  ): Promise<PairingStart> {
    return this.request("/auth/pairing-codes", {
      method: "POST",
      body: JSON.stringify({ label, grants }),
    });
  }
  /** A one-minute, single-use ticket for the change stream. */
  streamTicket(projectId: ProjectId): Promise<StreamTicket> {
    return this.request(`/projects/${projectId}/stream-ticket`, {
      method: "POST",
    });
  }
  listTokens(): Promise<TokenView[]> {
    return this.request("/auth/tokens");
  }
  revokeToken(id: string): Promise<void> {
    return this.request(`/auth/tokens/${id}`, { method: "DELETE" });
  }

  // ── projects and documents ──
  listProjects(): Promise<Project[]> {
    return this.request("/projects");
  }
  createProject(input: CreateProject): Promise<Project> {
    return this.request("/projects", {
      method: "POST",
      body: JSON.stringify(input),
    });
  }
  listDocuments(projectId: ProjectId): Promise<AstenDocument[]> {
    return this.request(`/projects/${projectId}/documents`);
  }
  createDocument(input: CreateDocument): Promise<AstenDocument> {
    return this.request(`/projects/${input.project_id}/documents`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }
  updateDocument(
    projectId: ProjectId,
    documentId: string,
    input: UpdateDocument,
  ): Promise<AstenDocument> {
    return this.request(`/projects/${projectId}/documents/${documentId}`, {
      method: "PUT",
      body: JSON.stringify(input),
    });
  }
  deleteDocument(projectId: ProjectId, documentId: string): Promise<void> {
    return this.request(`/projects/${projectId}/documents/${documentId}`, {
      method: "DELETE",
    });
  }

  // ── canon ──
  listFacts(projectId: ProjectId): Promise<Fact[]> {
    return this.request(`/projects/${projectId}/facts`);
  }
  createFact(input: CreateFact): Promise<Fact> {
    return this.request(`/projects/${input.project_id}/facts`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }
  updateFact(
    projectId: ProjectId,
    factId: string,
    input: UpdateFact,
  ): Promise<Fact> {
    return this.request(`/projects/${projectId}/facts/${factId}`, {
      method: "PUT",
      body: JSON.stringify(input),
    });
  }
  deleteFact(projectId: ProjectId, factId: string): Promise<void> {
    return this.request(`/projects/${projectId}/facts/${factId}`, {
      method: "DELETE",
    });
  }
  listProposals(projectId: ProjectId): Promise<Fact[]> {
    return this.request(`/projects/${projectId}/proposals`);
  }
  listEntities(projectId: ProjectId): Promise<CanonEntity[]> {
    return this.request(`/projects/${projectId}/entities`);
  }
  createEntity(input: CreateCanonEntity): Promise<CanonEntity> {
    return this.request(`/projects/${input.project_id}/entities`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }
  updateEntity(
    projectId: ProjectId,
    entityId: string,
    input: UpdateCanonEntity,
  ): Promise<CanonEntity> {
    return this.request(`/projects/${projectId}/entities/${entityId}`, {
      method: "PUT",
      body: JSON.stringify(input),
    });
  }
  deleteEntity(projectId: ProjectId, entityId: string): Promise<void> {
    return this.request(`/projects/${projectId}/entities/${entityId}`, {
      method: "DELETE",
    });
  }
  mergeEntity(
    projectId: ProjectId,
    sourceId: string,
    targetId: string,
  ): Promise<CanonEntity> {
    return this.request(
      `/projects/${projectId}/entities/${sourceId}/merge-into/${targetId}`,
      { method: "POST", body: "{}" },
    );
  }
  renameEntity(
    projectId: ProjectId,
    entityId: string,
    to: string,
    apply: boolean,
  ): Promise<RenamePreview> {
    return this.request(`/projects/${projectId}/entities/${entityId}/rename`, {
      method: "POST",
      body: JSON.stringify({ to, apply }),
    });
  }
  listEvents(projectId: ProjectId): Promise<TimelineEvent[]> {
    return this.request(`/projects/${projectId}/events`);
  }
  createEvent(input: CreateTimelineEvent): Promise<TimelineEvent> {
    return this.request(`/projects/${input.project_id}/events`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }
  updateEvent(
    projectId: ProjectId,
    eventId: string,
    input: UpdateTimelineEvent,
  ): Promise<TimelineEvent> {
    return this.request(`/projects/${projectId}/events/${eventId}`, {
      method: "PUT",
      body: JSON.stringify(input),
    });
  }
  deleteEvent(projectId: ProjectId, eventId: string): Promise<void> {
    return this.request(`/projects/${projectId}/events/${eventId}`, {
      method: "DELETE",
    });
  }

  // ── story language server ──
  diagnostics(projectId: ProjectId): Promise<DiagnosticsSnapshot> {
    return this.request(`/projects/${projectId}/story/diagnostics`);
  }
  factCard(
    projectId: ProjectId,
    entity: string,
    documentId?: string,
  ): Promise<FactCard> {
    const params = new URLSearchParams({ entity });
    if (documentId) params.set("document", documentId);
    return this.request(`/projects/${projectId}/story/fact-card?${params}`);
  }
  mentions(projectId: ProjectId, entity: string): Promise<MentionRef[]> {
    return this.request(
      `/projects/${projectId}/story/mentions?${new URLSearchParams({ entity })}`,
    );
  }
  whereSettled(projectId: ProjectId, fact: string): Promise<WhereSettled> {
    return this.request(
      `/projects/${projectId}/story/where-settled?${new URLSearchParams({ fact })}`,
    );
  }
  sceneSummary(
    projectId: ProjectId,
    documentId: string,
  ): Promise<SceneSummary> {
    return this.request(
      `/projects/${projectId}/story/scene-summary/${documentId}`,
    );
  }
  castSearch(projectId: ProjectId, q: string): Promise<CastSearchHit[]> {
    return this.request(
      `/projects/${projectId}/story/cast-search?${new URLSearchParams({ q })}`,
    );
  }
  applyRepair(
    projectId: ProjectId,
    request: ApplyRepairRequest,
  ): Promise<ApplyRepairResponse> {
    return this.request(`/projects/${projectId}/story/repairs/apply`, {
      method: "POST",
      body: JSON.stringify(request),
    });
  }
  bisect(projectId: ProjectId, diagnosticId: string): Promise<BisectResult> {
    return this.request(`/projects/${projectId}/story/bisect/${diagnosticId}`);
  }
  catalog(projectId: ProjectId): Promise<CodeInfo[]> {
    return this.request(`/projects/${projectId}/story/catalog`);
  }

  // ── settings, invariants, routes, baselines ──
  settings(projectId: ProjectId): Promise<ProjectSettings> {
    return this.request(`/projects/${projectId}/settings`);
  }
  updateSettings(
    projectId: ProjectId,
    input: UpdateSettings,
  ): Promise<ProjectSettings> {
    return this.request(`/projects/${projectId}/settings`, {
      method: "PUT",
      body: JSON.stringify(input),
    });
  }
  listInvariants(projectId: ProjectId): Promise<StoredInvariant[]> {
    return this.request(`/projects/${projectId}/invariants`);
  }
  createInvariant(
    projectId: ProjectId,
    input: CreateInvariant,
  ): Promise<StoredInvariant> {
    return this.request(`/projects/${projectId}/invariants`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }
  updateInvariant(
    projectId: ProjectId,
    id: string,
    invariant: Invariant,
  ): Promise<StoredInvariant> {
    return this.request(`/projects/${projectId}/invariants/${id}`, {
      method: "PUT",
      body: JSON.stringify(invariant),
    });
  }
  deleteInvariant(projectId: ProjectId, id: string): Promise<void> {
    return this.request(`/projects/${projectId}/invariants/${id}`, {
      method: "DELETE",
    });
  }
  listRoutes(projectId: ProjectId): Promise<StoredRoute[]> {
    return this.request(`/projects/${projectId}/routes`);
  }
  createRoute(projectId: ProjectId, route: Route): Promise<StoredRoute> {
    return this.request(`/projects/${projectId}/routes`, {
      method: "POST",
      body: JSON.stringify({ route }),
    });
  }
  deleteRoute(projectId: ProjectId, id: string): Promise<void> {
    return this.request(`/projects/${projectId}/routes/${id}`, {
      method: "DELETE",
    });
  }
  listBaselines(projectId: ProjectId): Promise<Baseline[]> {
    return this.request(`/projects/${projectId}/baselines`);
  }
  createBaseline(projectId: ProjectId, name: string): Promise<Baseline> {
    return this.request(`/projects/${projectId}/baselines`, {
      method: "POST",
      body: JSON.stringify({ name }),
    });
  }
  activateBaseline(projectId: ProjectId, id: string): Promise<Baseline[]> {
    return this.request(`/projects/${projectId}/baselines/${id}/activate`, {
      method: "POST",
      body: "{}",
    });
  }
  clearBaseline(projectId: ProjectId): Promise<Baseline[]> {
    return this.request(`/projects/${projectId}/baselines/clear`, {
      method: "POST",
      body: "{}",
    });
  }
  deleteBaseline(projectId: ProjectId, id: string): Promise<void> {
    return this.request(`/projects/${projectId}/baselines/${id}`, {
      method: "DELETE",
    });
  }

  // ── history and drafts ──
  history(projectId: ProjectId, documentId: string): Promise<EventRecord[]> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/history`,
    );
  }
  verify(projectId: ProjectId, documentId: string): Promise<VerifyReport> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/verify`,
    );
  }
  documentAt(
    projectId: ProjectId,
    documentId: string,
    seq: number,
  ): Promise<unknown> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/at/${seq}`,
    );
  }
  blame(
    projectId: ProjectId,
    documentId: string,
  ): Promise<Record<string, BlameSpan[]>> {
    return this.request(`/projects/${projectId}/documents/${documentId}/blame`);
  }
  listDrafts(projectId: ProjectId, documentId: string): Promise<Draft[]> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/drafts`,
    );
  }
  createDraft(
    projectId: ProjectId,
    documentId: string,
    name: string,
  ): Promise<Draft> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/drafts`,
      { method: "POST", body: JSON.stringify({ name }) },
    );
  }
  updateDraft(
    projectId: ProjectId,
    draftId: string,
    input: { name?: string; content_json?: unknown },
  ): Promise<Draft> {
    return this.request(`/projects/${projectId}/drafts/${draftId}`, {
      method: "PUT",
      body: JSON.stringify(input),
    });
  }
  deleteDraft(projectId: ProjectId, draftId: string): Promise<void> {
    return this.request(`/projects/${projectId}/drafts/${draftId}`, {
      method: "DELETE",
    });
  }
  mergePreview(
    projectId: ProjectId,
    draftId: string,
    resolutions: Record<string, string>,
  ): Promise<MergePreview> {
    return this.request(
      `/projects/${projectId}/drafts/${draftId}/merge-preview`,
      { method: "POST", body: JSON.stringify({ resolutions }) },
    );
  }
  mergeDraft(
    projectId: ProjectId,
    draftId: string,
    resolutions: Record<string, string>,
  ): Promise<AstenDocument> {
    return this.request(`/projects/${projectId}/drafts/${draftId}/merge`, {
      method: "POST",
      body: JSON.stringify({ resolutions }),
    });
  }

  // ── comments, snapshots, suppressions, packs, tracked changes ──
  listComments(projectId: ProjectId, documentId: string): Promise<Comment[]> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/comments`,
    );
  }
  createComment(
    projectId: ProjectId,
    documentId: string,
    input: CreateComment,
  ): Promise<Comment> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/comments`,
      { method: "POST", body: JSON.stringify(input) },
    );
  }
  updateComment(
    projectId: ProjectId,
    commentId: string,
    input: UpdateComment,
  ): Promise<Comment> {
    return this.request(`/projects/${projectId}/comments/${commentId}`, {
      method: "PUT",
      body: JSON.stringify(input),
    });
  }
  listSnapshots(
    projectId: ProjectId,
    documentId: string,
  ): Promise<DocumentSnapshot[]> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/snapshots`,
    );
  }
  createSnapshot(
    projectId: ProjectId,
    documentId: string,
    label?: string,
  ): Promise<DocumentSnapshot> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/snapshots`,
      { method: "POST", body: JSON.stringify({ label }) },
    );
  }
  restoreSnapshot(
    projectId: ProjectId,
    snapshotId: string,
  ): Promise<AstenDocument> {
    return this.request(
      `/projects/${projectId}/snapshots/${snapshotId}/restore`,
      { method: "POST", body: "{}" },
    );
  }
  listSuppressions(projectId: ProjectId): Promise<Suppression[]> {
    return this.request(`/projects/${projectId}/suppressions`);
  }
  createSuppression(
    projectId: ProjectId,
    input: CreateSuppression,
  ): Promise<Suppression> {
    return this.request(`/projects/${projectId}/suppressions`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }
  deleteSuppression(projectId: ProjectId, id: string): Promise<void> {
    return this.request(`/projects/${projectId}/suppressions/${id}`, {
      method: "DELETE",
    });
  }
  listPacks(): Promise<MethodPack[]> {
    return this.request("/packs");
  }
  updatePack(packId: string, enabled: boolean): Promise<MethodPack> {
    return this.request(`/packs/${packId}`, {
      method: "PUT",
      body: JSON.stringify({ enabled }),
    });
  }
  listTrackedChanges(
    projectId: ProjectId,
    documentId: string,
  ): Promise<TrackedChange[]> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/changes`,
    );
  }
  createTrackedChange(
    projectId: ProjectId,
    documentId: string,
    input: { kind: "insert" | "delete"; text: string; block_id?: string },
  ): Promise<TrackedChange> {
    return this.request(
      `/projects/${projectId}/documents/${documentId}/changes`,
      { method: "POST", body: JSON.stringify(input) },
    );
  }
  resolveTrackedChange(
    projectId: ProjectId,
    changeId: string,
    accepted: boolean,
  ): Promise<TrackedChange> {
    return this.request(`/projects/${projectId}/changes/${changeId}/resolve`, {
      method: "POST",
      body: JSON.stringify({ accepted }),
    });
  }

  // ── compile, import, world ──
  compileProject(
    projectId: ProjectId,
    request: CompileRequest,
  ): Promise<CompileArtifact> {
    return this.request(`/projects/${projectId}/compile`, {
      method: "POST",
      body: JSON.stringify(request),
    });
  }
  importDocument(
    projectId: ProjectId,
    request: ImportRequest,
  ): Promise<AstenDocument> {
    return this.request(`/projects/${projectId}/import`, {
      method: "POST",
      body: JSON.stringify(request),
    });
  }
  listWorldEntries(projectId: ProjectId): Promise<WorldEntry[]> {
    return this.request(`/projects/${projectId}/world-entries`);
  }
  createWorldEntry(input: CreateWorldEntry): Promise<WorldEntry> {
    return this.request(`/projects/${input.project_id}/world-entries`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }
  updateWorldEntry(
    projectId: ProjectId,
    entryId: string,
    input: UpdateWorldEntry,
  ): Promise<WorldEntry> {
    return this.request(`/projects/${projectId}/world-entries/${entryId}`, {
      method: "PUT",
      body: JSON.stringify(input),
    });
  }
  worldPulse(projectId: ProjectId): Promise<WorldPulse> {
    return this.request(`/projects/${projectId}/world-pulse`);
  }

  /**
   * Subscribe to the project's change stream. Returns a disposer.
   *
   * `EventSource` cannot send headers, so each connection spends a fresh
   * one-minute ticket instead of putting the bearer token in the URL. When
   * the stream drops, a new ticket is fetched and the connection reopened
   * with backoff until disposed.
   */
  async subscribe(
    projectId: ProjectId,
    onEvent: (event: StreamEvent) => void,
    onState?: (open: boolean) => void,
  ): Promise<() => void> {
    const origin = await this.origin();
    let source: EventSource | undefined;
    let disposed = false;
    let retry: number | undefined;
    let attempt = 0;

    const handler = (message: MessageEvent) => {
      try {
        onEvent(JSON.parse(message.data) as StreamEvent);
      } catch {
        // Heartbeats carry no JSON payload worth parsing.
      }
    };

    const schedule = () => {
      if (disposed) return;
      const delay = Math.min(30_000, 1000 * 2 ** Math.min(attempt, 5));
      attempt += 1;
      retry = window.setTimeout(() => void open(), delay);
    };

    const open = async () => {
      if (disposed) return;
      let ticket: string;
      try {
        ticket = (await this.streamTicket(projectId)).ticket;
      } catch {
        onState?.(false);
        schedule();
        return;
      }
      if (disposed) return;
      source = new EventSource(
        `${origin}/api/v1/projects/${projectId}/stream?ticket=${encodeURIComponent(ticket)}`,
        { withCredentials: true },
      );
      for (const name of [
        "revision",
        "diagnostics",
        "document",
        "canon",
        "heartbeat",
      ]) {
        source.addEventListener(name, handler as EventListener);
      }
      source.onopen = () => {
        attempt = 0;
        onState?.(true);
      };
      source.onerror = () => {
        onState?.(false);
        // A spent ticket cannot be replayed, so a dropped stream is reopened
        // by hand rather than left to EventSource's own retry.
        source?.close();
        source = undefined;
        schedule();
      };
    };

    void open();
    return () => {
      disposed = true;
      window.clearTimeout(retry);
      source?.close();
    };
  }

  private async request<T>(path: string, init?: RequestInit): Promise<T> {
    const origin = await this.origin();
    const token = await this.token();
    const headers: Record<string, string> = { accept: "application/json" };
    if (init?.body !== undefined) headers["content-type"] = "application/json";
    if (token) headers.authorization = `Bearer ${token}`;
    const response = await fetch(`${origin}/api/v1${path}`, {
      ...init,
      credentials: "include",
      headers: {
        ...headers,
        ...(init?.headers as Record<string, string> | undefined),
      },
    });
    if (response.status === 204) return undefined as T;
    if (response.status === 401 && !path.startsWith("/auth/")) {
      // The token was revoked or expired: forget it and let the desk return
      // to the pair screen instead of failing every query quietly.
      this.setToken(null);
      if (typeof window !== "undefined") {
        window.dispatchEvent(new CustomEvent("asten:unauthorized"));
      }
    }
    if (!response.ok) {
      let message = `Request failed: ${response.status}`;
      try {
        const body = (await response.json()) as ApiErrorBody;
        message = body.error?.message ?? message;
      } catch {
        // No JSON body.
      }
      throw new ApiError(response.status, message);
    }
    return response.json() as Promise<T>;
  }
}

let shared: AstenClient | undefined;

/** The one client instance the desk shares. */
export function createAstenClient(): AstenClient {
  shared ??= new AstenClient();
  return shared;
}
