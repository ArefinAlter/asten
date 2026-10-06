import type {
  ApiErrorBody,
  CreateDocument,
  CreateFact,
  CreateProject,
  CreateWorldEntry,
  Fact,
  MarginDiagnostic,
  MarginDocument,
  Project,
  ProjectId,
  UpdateDocument,
  UpdateWorldEntry,
  WorldEntry,
  WorldPulse,
} from "@margin/contracts";

export interface MarginClient {
  listProjects(): Promise<Project[]>;
  createProject(input: CreateProject): Promise<Project>;
  listDocuments(projectId: ProjectId): Promise<MarginDocument[]>;
  createDocument(input: CreateDocument): Promise<MarginDocument>;
  updateDocument(
    projectId: ProjectId,
    documentId: string,
    input: UpdateDocument,
  ): Promise<MarginDocument>;
  listFacts(projectId: ProjectId): Promise<Fact[]>;
  createFact(input: CreateFact): Promise<Fact>;
  listDiagnostics(projectId: ProjectId): Promise<MarginDiagnostic[]>;
  listWorldEntries(projectId: ProjectId): Promise<WorldEntry[]>;
  createWorldEntry(input: CreateWorldEntry): Promise<WorldEntry>;
  updateWorldEntry(
    projectId: ProjectId,
    entryId: string,
    input: UpdateWorldEntry,
  ): Promise<WorldEntry>;
  worldPulse(projectId: ProjectId): Promise<WorldPulse>;
}

export function createMarginClient(): MarginClient {
  return isTauri() ? new TauriClient() : new HttpClient();
}

function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

class HttpClient implements MarginClient {
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

  listDocuments(projectId: ProjectId): Promise<MarginDocument[]> {
    return this.request(`/projects/${projectId}/documents`);
  }

  createDocument(input: CreateDocument): Promise<MarginDocument> {
    return this.request(`/projects/${input.project_id}/documents`, {
      method: "POST",
      body: JSON.stringify(input),
    });
  }

  updateDocument(
    projectId: ProjectId,
    documentId: string,
    input: UpdateDocument,
  ): Promise<MarginDocument> {
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

  listDiagnostics(projectId: ProjectId): Promise<MarginDiagnostic[]> {
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

class TauriClient implements MarginClient {
  async listProjects(): Promise<Project[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<Project[]>("list_projects");
  }

  async createProject(input: CreateProject): Promise<Project> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<Project>("create_project", { input });
  }

  async listDocuments(projectId: ProjectId): Promise<MarginDocument[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<MarginDocument[]>("list_documents", { projectId });
  }

  async createDocument(input: CreateDocument): Promise<MarginDocument> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<MarginDocument>("create_document", { input });
  }

  async updateDocument(
    _projectId: ProjectId,
    documentId: string,
    input: UpdateDocument,
  ): Promise<MarginDocument> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<MarginDocument>("update_document", { documentId, input });
  }

  async listFacts(projectId: ProjectId): Promise<Fact[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<Fact[]>("list_facts", { projectId });
  }

  async createFact(input: CreateFact): Promise<Fact> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<Fact>("create_fact", { input });
  }

  async listDiagnostics(projectId: ProjectId): Promise<MarginDiagnostic[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<MarginDiagnostic[]>("list_diagnostics", { projectId });
  }

  async listWorldEntries(projectId: ProjectId): Promise<WorldEntry[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<WorldEntry[]>("list_world_entries", { projectId });
  }

  async createWorldEntry(input: CreateWorldEntry): Promise<WorldEntry> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<WorldEntry>("create_world_entry", { input });
  }

  async updateWorldEntry(
    _projectId: ProjectId,
    entryId: string,
    input: UpdateWorldEntry,
  ): Promise<WorldEntry> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<WorldEntry>("update_world_entry", { entryId, input });
  }

  async worldPulse(projectId: ProjectId): Promise<WorldPulse> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<WorldPulse>("world_pulse", { projectId });
  }
}
