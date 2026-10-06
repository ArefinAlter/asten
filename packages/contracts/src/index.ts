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

export interface AstenDocument {
  id: DocumentId;
  project_id: ProjectId;
  parent_id: DocumentId | null;
  kind: DocumentKind;
  title: string;
  content_json: unknown;
  position: number;
  revision: number;
  exclude_from_compile: boolean;
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
  parent_id?: DocumentId | null;
  position?: number;
  exclude_from_compile?: boolean;
  actor?: string | null;
}

export type FactModality =
  "established" | "inferred" | "contested" | "author_override";

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

export interface AstenDiagnostic {
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

export type EntityKind =
  "character" | "place" | "event" | "item" | "rule" | "source" | "other";

export interface CanonEntity {
  id: string;
  project_id: ProjectId;
  kind: EntityKind;
  name: string;
  aliases: string[];
  notes: string;
  created_at: string;
  updated_at: string;
}

export interface CreateCanonEntity {
  project_id: ProjectId;
  kind: EntityKind;
  name: string;
  aliases?: string[];
  notes?: string;
}

export interface UpdateCanonEntity {
  kind?: EntityKind;
  name?: string;
  aliases?: string[];
  notes?: string;
}

export interface TimelineEvent {
  id: string;
  project_id: ProjectId;
  entity_id: string | null;
  document_id: DocumentId | null;
  title: string;
  story_time: string | null;
  reading_order: number | null;
  duration: string | null;
  notes: string;
  created_at: string;
  updated_at: string;
}

export interface CreateTimelineEvent {
  project_id: ProjectId;
  title: string;
  entity_id?: string | null;
  document_id?: DocumentId | null;
  story_time?: string | null;
  reading_order?: number | null;
  duration?: string | null;
  notes?: string;
}

export interface UpdateTimelineEvent {
  title?: string;
  entity_id?: string | null;
  document_id?: DocumentId | null;
  story_time?: string | null;
  reading_order?: number | null;
  duration?: string | null;
  notes?: string;
}

export interface Comment {
  id: string;
  document_id: DocumentId;
  block_id: string | null;
  from_offset: number;
  to_offset: number;
  body: string;
  resolved: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateComment {
  document_id?: DocumentId;
  block_id?: string | null;
  from_offset?: number;
  to_offset?: number;
  body: string;
}

export interface UpdateComment {
  body?: string;
  resolved?: boolean;
}

export interface DocumentOp {
  id: string;
  document_id: DocumentId;
  seq: number;
  kind: string;
  op_json: unknown;
  actor: string;
  created_at: string;
}

export interface DocumentSnapshot {
  id: string;
  document_id: DocumentId;
  revision: number;
  label: string | null;
  content_json: unknown;
  created_at: string;
}

export interface Suppression {
  id: string;
  project_id: ProjectId;
  rule_code: string;
  document_id: DocumentId | null;
  entity_key: string | null;
  reason: string;
  created_at: string;
}

export interface MethodPack {
  id: string;
  pack_key: string;
  name: string;
  version: string;
  summary: string;
  enabled: boolean;
  manifest: unknown;
  created_at: string;
}

export type TrackedKind = "insert" | "delete";

export interface TrackedChange {
  id: string;
  document_id: DocumentId;
  kind: TrackedKind;
  block_id: string | null;
  text: string;
  actor: string;
  accepted: boolean | null;
  created_at: string;
}

export type CompileFormat = "markdown" | "plain" | "fountain" | "html" | "docx";

export interface CompileRequest {
  format?: CompileFormat;
  document_ids?: DocumentId[];
}

export interface CompileArtifact {
  filename: string;
  mime: string;
  encoding: string;
  body: string;
}

export type ImportFormat = "markdown" | "plain" | "fountain" | "html" | "docx";

export interface ImportRequest {
  filename: string;
  format: ImportFormat;
  body: string;
  encoding?: string | null;
  parent_id?: DocumentId | null;
  kind?: DocumentKind;
}
