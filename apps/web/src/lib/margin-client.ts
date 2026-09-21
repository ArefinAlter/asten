import type {
  ApiErrorBody,
  CreateDocument,
  CreateProject,
  MarginDocument,
  Project,
  ProjectId,
} from "@margin/contracts";

export interface MarginClient {
  listProjects(): Promise<Project[]>;
  createProject(input: CreateProject): Promise<Project>;
  listDocuments(projectId: ProjectId): Promise<MarginDocument[]>;
  createDocument(input: CreateDocument): Promise<MarginDocument>;
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
}
