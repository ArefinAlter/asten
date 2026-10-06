export type ProjectId = string;
export type DocumentId = string;
export type FactId = string;
export type DiagnosticId = string;
export type WorldEntryId = string;

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

export interface UpdateDocument {
  title?: string | null;
  content_json?: unknown;
}

export type FactModality =
  | "established"
  | "inferred"
  | "contested"
  | "author_override";

export type FactStatus = "proposed" | "confirmed" | "deprecated";

export type Severity = "error" | "warning" | "information" | "hint";

export interface SourceRange {
  document_id: DocumentId;
  start: number;
  end: number;
}

export interface Fact {
  id: FactId;
  project_id: ProjectId;
  entity_key: string;
  predicate: string;
  value: string;
  modality: FactModality;
  status: FactStatus;
  valid_from: string | null;
  valid_to: string | null;
  source: SourceRange | null;
  created_at: string;
  updated_at: string;
}

export interface CreateFact {
  project_id: ProjectId;
  entity_key: string;
  predicate: string;
  value: string;
  modality?: FactModality;
  status?: FactStatus;
  valid_from?: string | null;
  valid_to?: string | null;
}

export interface MarginDiagnostic {
  id: DiagnosticId;
  code: string;
  severity: Severity;
  message: string;
  source: SourceRange;
  details: string | null;
}

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
  };
}

export type WorldKind =
  | "place"
  | "culture"
  | "system"
  | "creature"
  | "item"
  | "question"
  | "fragment";

export type WorldStatus = "spark" | "sketched" | "settled";

export type IdeaPrompt = "why" | "how" | "what_result";

export interface WorldEntry {
  id: WorldEntryId;
  project_id: ProjectId;
  kind: WorldKind;
  title: string;
  summary: string;
  prompt: IdeaPrompt | null;
  status: WorldStatus;
  board_x: number;
  board_y: number;
  map_x: number | null;
  map_y: number | null;
  hue: string;
  created_at: string;
  updated_at: string;
}

export interface CreateWorldEntry {
  project_id: ProjectId;
  kind: WorldKind;
  title: string;
  summary?: string;
  prompt?: IdeaPrompt | null;
  status?: WorldStatus;
  board_x?: number;
  board_y?: number;
  map_x?: number | null;
  map_y?: number | null;
  hue?: string;
}

export interface UpdateWorldEntry {
  kind?: WorldKind;
  title?: string;
  summary?: string;
  prompt?: IdeaPrompt | null;
  status?: WorldStatus;
  board_x?: number;
  board_y?: number;
  map_x?: number | null;
  map_y?: number | null;
  hue?: string;
}

export interface WorldPulse {
  sparks: number;
  sketched: number;
  settled: number;
  questions: number;
  places: number;
  systems: number;
  scenes: number;
  underwater: number;
  iceberg: number;
  discoveries: string[];
}
