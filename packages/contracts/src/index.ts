export type ProjectId = string;
export type DocumentId = string;

export type ProjectKind = "novel" | "screenplay" | "non_fiction" | "other";

export interface Project {
  id: ProjectId;
  name: string;
  kind: ProjectKind;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateProject {
  name: string;
  kind: ProjectKind;
  description?: string | null;
}

export type DocumentKind =
  "manuscript" | "chapter" | "scene" | "wiki" | "note" | "source";

export interface MarginDocument {
  id: DocumentId;
  project_id: ProjectId;
  parent_id: DocumentId | null;
  kind: DocumentKind;
  title: string;
  content_json: unknown;
  position: number;
  revision: number;
  created_at: string;
  updated_at: string;
}

export interface CreateDocument {
  project_id: ProjectId;
  parent_id?: DocumentId | null;
  kind: DocumentKind;
  title: string;
  content_json?: unknown;
  position?: number;
}

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
  };
}
