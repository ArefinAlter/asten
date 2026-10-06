import type {
  ApiErrorBody,
  CanonEntity,
  Comment,
  CompileArtifact,
  CompileRequest,
  CreateCanonEntity,
  CreateComment,
  CreateDocument,
  CreateFact,
  CreateProject,
  CreateTimelineEvent,
  CreateWorldEntry,
  DocumentOp,
  DocumentSnapshot,
  Fact,
  ImportRequest,
  AstenDiagnostic,
  AstenDocument,
  MethodPack,
  Project,
  ProjectId,
  Suppression,
  TimelineEvent,
  TrackedChange,
  UpdateCanonEntity,
  UpdateComment,
  UpdateDocument,
  UpdateTimelineEvent,
  UpdateWorldEntry,
  WorldEntry,
  WorldPulse,
} from "@asten/contracts";

export interface AstenClient {
  listProjects(): Promise<Project[]>;
  createProject(input: CreateProject): Promise<Project>;
  listDocuments(projectId: ProjectId): Promise<AstenDocument[]>;
  createDocument(input: CreateDocument): Promise<AstenDocument>;
  updateDocument(
    projectId: ProjectId,
    documentId: string,
    input: UpdateDocument,
  ): Promise<AstenDocument>;
  listFacts(projectId: ProjectId): Promise<Fact[]>;
  createFact(input: CreateFact): Promise<Fact>;
  listDiagnostics(projectId: ProjectId): Promise<AstenDiagnostic[]>;
  listWorldEntries(projectId: ProjectId): Promise<WorldEntry[]>;
  createWorldEntry(input: CreateWorldEntry): Promise<WorldEntry>;
  updateWorldEntry(
    projectId: ProjectId,
    entryId: string,
    input: UpdateWorldEntry,
  ): Promise<WorldEntry>;
  worldPulse(projectId: ProjectId): Promise<WorldPulse>;
  listEntities(projectId: ProjectId): Promise<CanonEntity[]>;
  createEntity(input: CreateCanonEntity): Promise<CanonEntity>;
  updateEntity(
    projectId: ProjectId,
    entityId: string,
    input: UpdateCanonEntity,
  ): Promise<CanonEntity>;
  listEvents(projectId: ProjectId): Promise<TimelineEvent[]>;
  createEvent(input: CreateTimelineEvent): Promise<TimelineEvent>;
  updateEvent(
    projectId: ProjectId,
    eventId: string,
    input: UpdateTimelineEvent,
  ): Promise<TimelineEvent>;
  listComments(projectId: ProjectId, documentId: string): Promise<Comment[]>;
  createComment(
    projectId: ProjectId,
    documentId: string,
    input: CreateComment,
  ): Promise<Comment>;
  updateComment(
    projectId: ProjectId,
    commentId: string,
    input: UpdateComment,
  ): Promise<Comment>;
  listOps(projectId: ProjectId, documentId: string): Promise<DocumentOp[]>;
  listSnapshots(
    projectId: ProjectId,
    documentId: string,
  ): Promise<DocumentSnapshot[]>;
  createSnapshot(
    projectId: ProjectId,
    documentId: string,
    label?: string,
  ): Promise<DocumentSnapshot>;
  restoreSnapshot(
    projectId: ProjectId,
    snapshotId: string,
  ): Promise<AstenDocument>;
  listSuppressions(projectId: ProjectId): Promise<Suppression[]>;
  createSuppression(
    projectId: ProjectId,
    input: {
      rule_code: string;
      document_id?: string | null;
      entity_key?: string | null;
      reason?: string;
    },
  ): Promise<Suppression>;
  listPacks(): Promise<MethodPack[]>;
  updatePack(packId: string, enabled: boolean): Promise<MethodPack>;
  listTrackedChanges(
    projectId: ProjectId,
    documentId: string,
  ): Promise<TrackedChange[]>;
  createTrackedChange(
    projectId: ProjectId,
    documentId: string,
    input: { kind: "insert" | "delete"; text: string; block_id?: string },
  ): Promise<TrackedChange>;
  resolveTrackedChange(
    projectId: ProjectId,
    changeId: string,
    accepted: boolean,
  ): Promise<TrackedChange>;
  compileProject(
    projectId: ProjectId,
    request: CompileRequest,
  ): Promise<CompileArtifact>;
  importDocument(
    projectId: ProjectId,
    request: ImportRequest,
  ): Promise<AstenDocument>;
}

export function createAstenClient(): AstenClient {
  return isTauri() ? new TauriClient() : new HttpClient();
}

function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

class HttpClient implements AstenClient {
  private readonly baseUrl: string;

  constructor(baseUrl = "/api/v1") {
    this.baseUrl = baseUrl;
  }

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

  listFacts(projectId: ProjectId): Promise<Fact[]> {
    return this.request(`/projects/${projectId}/facts`);
  }

  createFact(input: CreateFact): Promise<Fact> {
    return this.request(`/projects/${input.project_id}/facts`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }

  listDiagnostics(projectId: ProjectId): Promise<AstenDiagnostic[]> {
    return this.request(`/projects/${projectId}/diagnostics`);
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

  listOps(projectId: ProjectId, documentId: string): Promise<DocumentOp[]> {
    return this.request(`/projects/${projectId}/documents/${documentId}/ops`);
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
    input: {
      rule_code: string;
      document_id?: string | null;
      entity_key?: string | null;
      reason?: string;
    },
  ): Promise<Suppression> {
    return this.request(`/projects/${projectId}/suppressions`, {
      method: "POST",
      body: JSON.stringify(input),
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

  private async request<T>(path: string, init?: RequestInit): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      ...init,
      headers: {
        "content-type": "application/json",
        ...init?.headers,
      },
    });
    if (!response.ok) {
      const body = (await response.json()) as ApiErrorBody;
      throw new Error(
        body.error?.message ?? `Request failed: ${response.status}`,
      );
    }
    return response.json() as Promise<T>;
  }
}

class TauriClient implements AstenClient {
  private async invoke<T>(command: string, args?: Record<string, unknown>) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(command, args);
  }

  listProjects(): Promise<Project[]> {
    return this.invoke("list_projects");
  }
  createProject(input: CreateProject): Promise<Project> {
    return this.invoke("create_project", { input });
  }
  listDocuments(projectId: ProjectId): Promise<AstenDocument[]> {
    return this.invoke("list_documents", { projectId });
  }
  createDocument(input: CreateDocument): Promise<AstenDocument> {
    return this.invoke("create_document", { input });
  }
  updateDocument(
    _projectId: ProjectId,
    documentId: string,
    input: UpdateDocument,
  ): Promise<AstenDocument> {
    return this.invoke("update_document", { documentId, input });
  }
  listFacts(projectId: ProjectId): Promise<Fact[]> {
    return this.invoke("list_facts", { projectId });
  }
  createFact(input: CreateFact): Promise<Fact> {
    return this.invoke("create_fact", { input });
  }
  listDiagnostics(projectId: ProjectId): Promise<AstenDiagnostic[]> {
    return this.invoke("list_diagnostics", { projectId });
  }
  listWorldEntries(projectId: ProjectId): Promise<WorldEntry[]> {
    return this.invoke("list_world_entries", { projectId });
  }
  createWorldEntry(input: CreateWorldEntry): Promise<WorldEntry> {
    return this.invoke("create_world_entry", { input });
  }
  updateWorldEntry(
    _projectId: ProjectId,
    entryId: string,
    input: UpdateWorldEntry,
  ): Promise<WorldEntry> {
    return this.invoke("update_world_entry", { entryId, input });
  }
  worldPulse(projectId: ProjectId): Promise<WorldPulse> {
    return this.invoke("world_pulse", { projectId });
  }
  listEntities(projectId: ProjectId): Promise<CanonEntity[]> {
    return this.invoke("list_entities", { projectId });
  }
  createEntity(input: CreateCanonEntity): Promise<CanonEntity> {
    return this.invoke("create_entity", { input });
  }
  updateEntity(
    _projectId: ProjectId,
    entityId: string,
    input: UpdateCanonEntity,
  ): Promise<CanonEntity> {
    return this.invoke("update_entity", { entityId, input });
  }
  listEvents(projectId: ProjectId): Promise<TimelineEvent[]> {
    return this.invoke("list_events", { projectId });
  }
  createEvent(input: CreateTimelineEvent): Promise<TimelineEvent> {
    return this.invoke("create_event", { input });
  }
  updateEvent(
    _projectId: ProjectId,
    eventId: string,
    input: UpdateTimelineEvent,
  ): Promise<TimelineEvent> {
    return this.invoke("update_event", { eventId, input });
  }
  listComments(_projectId: ProjectId, documentId: string): Promise<Comment[]> {
    return this.invoke("list_comments", { documentId });
  }
  createComment(
    _projectId: ProjectId,
    documentId: string,
    input: CreateComment,
  ): Promise<Comment> {
    return this.invoke("create_comment", {
      input: { ...input, document_id: documentId },
    });
  }
  updateComment(
    _projectId: ProjectId,
    commentId: string,
    input: UpdateComment,
  ): Promise<Comment> {
    return this.invoke("update_comment", { commentId, input });
  }
  listOps(_projectId: ProjectId, documentId: string): Promise<DocumentOp[]> {
    return this.invoke("list_ops", { documentId });
  }
  listSnapshots(
    _projectId: ProjectId,
    documentId: string,
  ): Promise<DocumentSnapshot[]> {
    return this.invoke("list_snapshots", { documentId });
  }
  createSnapshot(
    _projectId: ProjectId,
    documentId: string,
    label?: string,
  ): Promise<DocumentSnapshot> {
    return this.invoke("create_snapshot", { input: { document_id: documentId, label } });
  }
  restoreSnapshot(
    _projectId: ProjectId,
    snapshotId: string,
  ): Promise<AstenDocument> {
    return this.invoke("restore_snapshot", { snapshotId });
  }
  listSuppressions(projectId: ProjectId): Promise<Suppression[]> {
    return this.invoke("list_suppressions", { projectId });
  }
  createSuppression(
    projectId: ProjectId,
    input: {
      rule_code: string;
      document_id?: string | null;
      entity_key?: string | null;
      reason?: string;
    },
  ): Promise<Suppression> {
    return this.invoke("create_suppression", {
      input: { ...input, project_id: projectId },
    });
  }
  listPacks(): Promise<MethodPack[]> {
    return this.invoke("list_packs");
  }
  updatePack(packId: string, enabled: boolean): Promise<MethodPack> {
    return this.invoke("update_pack", { packId, input: { enabled } });
  }
  listTrackedChanges(
    _projectId: ProjectId,
    documentId: string,
  ): Promise<TrackedChange[]> {
    return this.invoke("list_tracked_changes", { documentId });
  }
  createTrackedChange(
    _projectId: ProjectId,
    documentId: string,
    input: { kind: "insert" | "delete"; text: string; block_id?: string },
  ): Promise<TrackedChange> {
    return this.invoke("create_tracked_change", {
      input: { ...input, document_id: documentId },
    });
  }
  resolveTrackedChange(
    _projectId: ProjectId,
    changeId: string,
    accepted: boolean,
  ): Promise<TrackedChange> {
    return this.invoke("resolve_tracked_change", {
      changeId,
      input: { accepted },
    });
  }
  compileProject(
    projectId: ProjectId,
    request: CompileRequest,
  ): Promise<CompileArtifact> {
    return this.invoke("compile_project", { projectId, request });
  }
  importDocument(
    projectId: ProjectId,
    request: ImportRequest,
  ): Promise<AstenDocument> {
    return this.invoke("import_document", { projectId, request });
  }
}
