// Shared TypeScript contracts for the Asten API and the Tauri shell. These
// mirror the Rust serde shapes in asten-domain, asten-ir, asten-diagnostics and
// asten-ls. Keep field names snake_case to match the wire.

export type ProjectId = string;
export type DocumentId = string;
export type FactId = string;
export type WorldEntryId = string;

// Generated from the Rust types (`pnpm contracts:gen`); `drift.ts` checks that
// every handwritten shape below agrees with its generated twin.
import type { JsonValue } from "./generated/serde_json/JsonValue";
import type { RichTextPatch } from "./generated/blocks/RichTextPatch";
export type { JsonValue, RichTextPatch };
export type * as generated from "./generated";

/** The family a diagnostic code belongs to. */
export type Area =
  | "attributes"
  | "lifecycles"
  | "time"
  | "space"
  | "knowledge"
  | "relations"
  | "rules"
  | "names"
  | "ledger"
  | "craft";

// ───────────────────────── projects and documents ─────────────────────────

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

/** The epistemic frame a statement lives in. */
export type Context =
  | { kind: "world" }
  | { kind: "narration" }
  | { kind: "says"; speaker: string | null }
  | { kind: "believes"; who: string }
  | { kind: "memory" }
  | { kind: "dream"; key: string }
  | { kind: "vision"; key: string }
  | { kind: "frame"; key: string }
  | { kind: "hypothetical" };

export interface ScenePlacement {
  story_time?: string | null;
  tick?: number | null;
  tick_lo?: number | null;
  tick_hi?: number | null;
  context: Context;
  continuity_run?: string | null;
  language?: string | null;
  introduces: string[];
}

export interface AstenDocument {
  id: DocumentId;
  project_id: ProjectId;
  parent_id: DocumentId | null;
  kind: DocumentKind;
  title: string;
  content_json: JsonValue;
  position: number;
  revision: number;
  exclude_from_compile: boolean;
  scene: ScenePlacement;
  head_seq: number;
  chain_hash: string;
  created_at: string;
  updated_at: string;
}

/**
 * A document without its body: what `GET /documents?fields=summary` lists.
 * `words` is the body's plain text split on Unicode whitespace, the same rule
 * the desk's `countWords` applies to the open page.
 */
export interface DocumentSummary {
  id: DocumentId;
  project_id: ProjectId;
  parent_id: DocumentId | null;
  kind: DocumentKind;
  title: string;
  position: number;
  revision: number;
  exclude_from_compile: boolean;
  scene: ScenePlacement;
  head_seq: number;
  chain_hash: string;
  words: number;
  created_at: string;
  updated_at: string;
}

export interface CreateDocument {
  project_id?: ProjectId;
  parent_id?: DocumentId | null;
  kind: DocumentKind;
  title: string;
  content_json?: JsonValue;
  position?: number;
  scene?: ScenePlacement | null;
}

export interface UpdateDocument {
  title?: string | null;
  content_json?: JsonValue | null;
  parent_id?: DocumentId | null;
  position?: number | null;
  exclude_from_compile?: boolean | null;
  scene?: ScenePlacement | null;
  actor?: string | null;
  expected_head_seq?: number | null;
}

// ───────────────────────── anchors, facts, events ─────────────────────────

export interface AnchorRevision {
  seq: number;
  fingerprint: string;
}

export interface Anchor {
  document: string;
  block: string;
  start: number;
  end: number;
  revision?: AnchorRevision | null;
  quote: string;
  context_before: string;
  context_after: string;
  stale: boolean;
}

export type Provenance =
  | { kind: "author"; action: string }
  | { kind: "text"; anchor: Anchor }
  | { kind: "wiki"; document: string }
  | {
      kind: "model";
      id: string;
      version: string;
      anchor?: Anchor | null;
      input_fingerprint: string;
    }
  | { kind: "rule"; code: string; premises: string[] }
  | { kind: "import"; job: string; anchor?: Anchor | null };

export type FactModality =
  "established" | "inferred" | "contested" | "author_override";

export type FactStatus = "proposed" | "confirmed" | "deprecated" | "rejected";

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
  source?: SourceRange | null;
  context: Context;
  confidence: number;
  anchor?: Anchor | null;
  provenance: Provenance;
  settled_by?: string | null;
  settled_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateFact {
  project_id?: ProjectId;
  entity_key: string;
  predicate: string;
  value: string;
  modality?: FactModality;
  status?: FactStatus;
  valid_from?: string | null;
  valid_to?: string | null;
  source?: SourceRange | null;
  context?: Context | null;
  anchor?: Anchor | null;
  confidence?: number | null;
  provenance?: Provenance | null;
}

export interface UpdateFact {
  value?: string | null;
  predicate?: string | null;
  status?: FactStatus | null;
  modality?: FactModality | null;
  valid_from?: string | null;
  valid_to?: string | null;
  context?: Context | null;
  anchor?: Anchor | null;
  actor?: string | null;
}

export type Sort =
  | "character"
  | "place"
  | "item"
  | "group"
  | "event"
  | "concept"
  | "rule"
  | "source"
  | "other";

export type IrValue =
  | { kind: "concept"; id: string; label: string }
  | { kind: "quantity"; lo: number; hi: number; label: string }
  | { kind: "entity"; id: string }
  | { kind: "text"; norm: string; label: string }
  | { kind: "bool"; value: boolean };

export interface Fluent {
  predicate: string;
  subject: string;
  value: IrValue;
}

export type EventKind =
  | { kind: "born" }
  | { kind: "dies" }
  | { kind: "resurrected" }
  | { kind: "destroyed" }
  | { kind: "repaired" }
  | { kind: "travels"; from: string | null; to: string | null }
  | { kind: "arrives"; place: string }
  | { kind: "acquires"; item: string }
  | { kind: "loses"; item: string }
  | { kind: "learns"; secret: string }
  | { kind: "tells"; to: string; secret: string }
  | { kind: "reveals"; secret: string }
  | { kind: "uses_knowledge"; secret: string }
  | { kind: "marries"; to: string }
  | { kind: "divorces"; to: string }
  | { kind: "changes"; fluent: Fluent }
  | { kind: "acts" }
  | { kind: "other"; label: string };

export type EntityKind =
  "character" | "place" | "event" | "item" | "rule" | "source" | "other";

export interface CanonEntity {
  id: string;
  project_id: ProjectId;
  kind: EntityKind;
  name: string;
  aliases: string[];
  notes: string;
  introduced_at?: number | null;
  created_at: string;
  updated_at: string;
}

export interface CreateCanonEntity {
  project_id?: ProjectId;
  kind: EntityKind;
  name: string;
  aliases?: string[];
  notes?: string;
  introduced_at?: number | null;
}

export interface UpdateCanonEntity {
  kind?: EntityKind | null;
  name?: string | null;
  aliases?: string[] | null;
  notes?: string | null;
  introduced_at?: number | null;
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
  kind: EventKind;
  participants: string[];
  context: Context;
  status: FactStatus;
  confidence: number;
  anchor?: Anchor | null;
  tick?: number | null;
  tick_lo?: number | null;
  tick_hi?: number | null;
  provenance: Provenance;
  created_at: string;
  updated_at: string;
}

export interface CreateTimelineEvent {
  project_id?: ProjectId;
  title: string;
  entity_id?: string | null;
  document_id?: DocumentId | null;
  story_time?: string | null;
  reading_order?: number | null;
  duration?: string | null;
  notes?: string;
  kind?: EventKind | null;
  participants?: string[];
  context?: Context | null;
  status?: FactStatus | null;
  tick?: number | null;
  tick_lo?: number | null;
  tick_hi?: number | null;
  anchor?: Anchor | null;
}

export interface UpdateTimelineEvent {
  title?: string | null;
  entity_id?: string | null;
  document_id?: DocumentId | null;
  story_time?: string | null;
  reading_order?: number | null;
  duration?: string | null;
  notes?: string | null;
  kind?: EventKind | null;
  participants?: string[] | null;
  context?: Context | null;
  status?: FactStatus | null;
  tick?: number | null;
  tick_lo?: number | null;
  tick_hi?: number | null;
}

// ───────────────────────── diagnostics ─────────────────────────

export type Severity = "error" | "warning" | "information" | "hint";
export type Tier = "must" | "may";

export interface Location {
  document: string;
  block?: string | null;
  start: number;
  end: number;
  quote: string;
  label: string;
}

export type StatementKind =
  | "fact"
  | "observation"
  | "event"
  | "scene"
  | "constraint"
  | "mention"
  | "invariant"
  | "rule";

export interface Statement {
  kind: StatementKind;
  id: string;
  text: string;
  location?: Location | null;
  status: "settled" | "proposed" | "deprecated" | "rejected";
  fault_prior: number;
  confidence: number;
}

export type RepairKind =
  | {
      kind: "add_change_event";
      entity: string;
      predicate: string;
      to_value: JsonValue;
      after_tick?: number | null;
      before_tick?: number | null;
    }
  | { kind: "mark_context"; document: string; context: Context }
  | {
      kind: "set_validity";
      fact: string;
      from_tick?: number | null;
      to_tick?: number | null;
    }
  | {
      kind: "place_scene";
      document: string;
      lo?: number | null;
      hi?: number | null;
    }
  | {
      kind: "move_event";
      event: string;
      lo?: number | null;
      hi?: number | null;
    }
  | { kind: "deprecate_fact"; fact: string }
  | { kind: "settle_fact"; fact: string }
  | { kind: "add_entity"; name: string; sort: Sort }
  | { kind: "add_alias"; entity: string; alias: string }
  | { kind: "clear_continuity"; document: string }
  | { kind: "suppress" }
  | { kind: "keep" };

export interface Repair {
  kind: RepairKind;
  label: string;
  cost: number;
}

export interface Diagnosis {
  statement_id: string;
  probability: number;
}

export interface DiagnosticProvenance {
  rules: string[];
  packs: string[];
  revision: string;
  engine_version: string;
}

export interface AstenDiagnostic {
  id: string;
  code: string;
  severity: Severity;
  tier: Tier;
  probability: number;
  title: string;
  message: string;
  primary?: Location | null;
  related: [Location, string][];
  explanation: Statement[];
  culprit: Diagnosis[];
  repairs: Repair[];
  provenance: DiagnosticProvenance;
  stale: boolean;
  suppressed_by?: string | null;
  new_since_baseline: boolean;
  entity?: string | null;
  details: string;
  evidence_fingerprint: string;
}

export interface Obligation {
  invariant: string;
  label: string;
  since: string;
  since_title: string;
}

export interface LanguageTier {
  script: string;
  tier: string;
  blocks: number;
}

export interface Timings {
  perceive: number;
  analyze: number;
  total: number;
}

export interface DiagnosticsSnapshot {
  revision: string;
  diagnostics: AstenDiagnostic[];
  ledger: Obligation[];
  tiers: LanguageTier[];
  stale_suppressions: string[];
  timings_ms: Timings;
  cached_blocks: number;
  total_blocks: number;
}

export interface MarksDiff {
  added: AstenDiagnostic[];
  removed: string[];
  changed: AstenDiagnostic[];
}

export interface DiagnosticsDelta extends MarksDiff {
  from_revision: string;
  revision: string;
}

export type StreamEvent =
  | { type: "revision"; revision: string }
  | ({ type: "diagnostics" } & DiagnosticsDelta)
  | { type: "document"; document: string; seq: number }
  | { type: "canon"; what: string }
  | { type: "heartbeat" };

export interface CodeInfo {
  code: string;
  area: Area;
  title: string;
  summary: string;
  pack: string;
}

// ───────────────────────── story language server ─────────────────────────

export interface FactSummary {
  id: string;
  predicate: string;
  value: string;
  status: string;
  context: string;
  valid_label: string;
  settled_in?: string | null;
}

export interface FactCard {
  entity: string;
  name: string;
  sort: Sort | null;
  aliases: string[];
  facts: FactSummary[];
  at_label: string;
  mention_count: number;
  open_marks: number;
}

export interface MentionRef {
  document: string;
  document_title: string;
  block: string;
  start: number;
  end: number;
  quote: string;
  reading_pos: number;
  confidence: number;
  context: string;
}

export interface WhereSettled {
  fact: string;
  document?: string | null;
  block?: string | null;
  start: number;
  end: number;
  quote: string;
  settled_by: string;
  settled_at: string;
}

export interface RenameChange {
  document: string;
  document_title: string;
  block: string;
  start: number;
  end: number;
  before: string;
  after: string;
}

export interface RenamePreview {
  entity: string;
  from: string;
  to: string;
  changes: RenameChange[];
}

export interface TimeBounds {
  lo?: number | null;
  hi?: number | null;
}

export interface SceneSummary {
  document: string;
  title: string;
  reading_pos: number;
  time_label: string;
  bounds: TimeBounds;
  bounds_label: string;
  context: string;
  continuity_run?: string | null;
  mentions: number;
  entities: string[];
  events: number;
  observations: number;
  marks: number;
  script: string;
  tier: string;
  words: number;
}

export interface CastSearchHit {
  entity: string;
  name: string;
  sort: Sort;
  score: number;
  mentions: number;
}

export interface ApplyRepairRequest {
  diagnostic_id: string;
  repair_index: number;
  reason?: string;
  scope?: string | null;
}

export interface ApplyRepairResponse {
  applied: Repair;
  message: string;
}

export interface BisectResult {
  diagnostic_id: string;
  found: boolean;
  document?: string | null;
  seq?: number | null;
  actor: string;
  at: string;
  event_type: string;
  probes: number;
}

// ───────────────────────── history ─────────────────────────

export type EventPayload =
  | {
      type: "NodeCreated";
      node_id: string;
      node_type: string;
      parent_id?: string | null;
      pos: string;
      fields: JsonValue;
    }
  | {
      type: "NodeMoved";
      node_id: string;
      new_parent_id?: string | null;
      new_pos: string;
    }
  | { type: "NodeDeleted"; node_id: string }
  | { type: "NodeRestored"; node_id: string }
  | { type: "FieldEdited"; node_id: string; field: string; value: JsonValue }
  | { type: "RichTextPatched"; node_id: string; patch: RichTextPatch }
  | { type: "MetaEdited"; field: string; value: JsonValue };

export interface EventRecord {
  id: string;
  seq: number;
  actor: string;
  at: string;
  payload: EventPayload;
  content_hash: string;
  prev_chain_hash: string;
  chain_hash: string;
}

export interface VerifyReport {
  document_id: DocumentId;
  events: number;
  head_seq: number;
  chain_hash: string;
  merkle_root: string;
  ok: boolean;
  error?: string | null;
}

export interface BlameSpan {
  start: number;
  end: number;
  seq: number;
  actor: string;
  at: string;
}

export interface Draft {
  id: string;
  document_id: DocumentId;
  name: string;
  base_seq: number;
  base_state_json: JsonValue;
  content_json: JsonValue;
  created_at: string;
  updated_at: string;
  merged_at?: string | null;
}

export interface MergeConflict {
  key: string;
  node_id: string;
  field: string;
  base: JsonValue;
  ours: JsonValue;
  theirs: JsonValue;
}

export interface MergePreview {
  draft_id: string;
  merged_content: JsonValue;
  conflicts: MergeConflict[];
  ops: number;
  new_marks: AstenDiagnostic[];
  resolved_marks: string[];
}

export interface Baseline {
  id: string;
  project_id: ProjectId;
  name: string;
  revision: string;
  diagnostic_ids: string[];
  active: boolean;
  created_at: string;
}

// ───────────────────────── settings, invariants, routes ─────────────────────────

export type Strictness = "quiet" | "standard" | "thorough";
export type TimeUnit = "minute" | "hour" | "day";

export type Calendar =
  | { kind: "day_count" }
  | { kind: "gregorian" }
  | {
      kind: "fictional";
      months: { name: string; days: number }[];
      leap?: { every: number; extra_days: number; month: number } | null;
      epoch_year: number;
      eras: string[];
      seasons: string[];
    };

export interface Clock {
  unit: TimeUnit;
  calendar: Calendar;
}

export interface ProjectSettings {
  project_id: ProjectId;
  strictness: Strictness;
  narrator_reliable: boolean;
  clock: Clock;
  min_parent_age_days?: number | null;
  resurrection_allowed: Sort[];
  updated_at: string;
}

export interface UpdateSettings {
  strictness?: Strictness | null;
  narrator_reliable?: boolean | null;
  clock?: Clock | null;
  min_parent_age_days?: number | null;
  resurrection_allowed?: Sort[] | null;
}

export type EventKindPattern =
  | "born"
  | "dies"
  | "travels"
  | "acquires"
  | "loses"
  | "learns"
  | "tells"
  | "reveals"
  | "marries"
  | "acts"
  | "any"
  | { other: string };

export type Proposition =
  | { kind: "holds"; fluent: Fluent }
  | { kind: "happens"; subject: string | null; event: EventKindPattern }
  | { kind: "mentioned"; entity: string }
  | { kind: "in_context"; context: Context };

export type InvariantPattern =
  | { pattern: "absence_after"; after: Proposition; never: Proposition }
  | { pattern: "response"; trigger: Proposition; response: Proposition }
  | { pattern: "precedence"; first: Proposition; later: Proposition }
  | {
      pattern: "universality_between";
      from: Proposition;
      until: Proposition;
      always: Proposition;
    }
  | { pattern: "bounded_existence"; event: Proposition; max: number };

export type TraceClock = "story_time" | "reading_order";

export interface Invariant {
  id: string;
  label: string;
  pattern: InvariantPattern;
  clock: TraceClock;
  enabled: boolean;
}

export interface StoredInvariant {
  id: string;
  project_id: ProjectId;
  invariant: Invariant;
  created_at: string;
}

export interface CreateInvariant {
  label: string;
  pattern: InvariantPattern;
  clock?: TraceClock;
  enabled?: boolean;
}

export interface Route {
  from: string;
  to: string;
  min_ticks: number;
  max_ticks?: number | null;
  mode: string;
  bidirectional: boolean;
  available: TimeBounds;
}

export interface CreateRoute {
  route: Route;
}

export interface StoredRoute {
  id: string;
  project_id: ProjectId;
  route: Route;
  created_at: string;
}

// ───────────────────────── misc existing surfaces ─────────────────────────

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

export interface DocumentSnapshot {
  id: string;
  document_id: DocumentId;
  revision: number;
  label: string | null;
  content_json: JsonValue;
  seq: number;
  chain_hash: string;
  merkle_root: string;
  created_at: string;
}

export type SuppressionScope = "occurrence" | "entity" | "document" | "project";

export interface Suppression {
  id: string;
  project_id: ProjectId;
  rule_code: string;
  document_id: DocumentId | null;
  entity_key: string | null;
  reason: string;
  scope: SuppressionScope;
  diagnostic_id?: string | null;
  evidence_fingerprint?: string | null;
  stale: boolean;
  created_at: string;
}

export interface CreateSuppression {
  project_id?: ProjectId;
  rule_code: string;
  document_id?: DocumentId | null;
  entity_key?: string | null;
  reason?: string;
  scope?: SuppressionScope | null;
  diagnostic_id?: string | null;
  evidence_fingerprint?: string | null;
}

export interface MethodPack {
  id: string;
  pack_key: string;
  name: string;
  version: string;
  summary: string;
  enabled: boolean;
  manifest: JsonValue;
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
  kind?: DocumentKind | null;
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
  project_id?: ProjectId;
  kind: WorldKind;
  title: string;
  summary?: string;
  prompt?: IdeaPrompt | null;
  status?: WorldStatus | null;
  board_x?: number | null;
  board_y?: number | null;
  map_x?: number | null;
  map_y?: number | null;
  hue?: string | null;
}

export interface UpdateWorldEntry {
  kind?: WorldKind | null;
  title?: string | null;
  summary?: string | null;
  prompt?: IdeaPrompt | null;
  status?: WorldStatus | null;
  board_x?: number | null;
  board_y?: number | null;
  map_x?: number | null;
  map_y?: number | null;
  hue?: string | null;
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

// ───────────────────────── auth ─────────────────────────

export type AuthMode = "off" | "pairing" | "owner";
export type TokenKind = "desk" | "guest" | "session";

/** What the desk shows about the machine running the API. */
export interface DeskInfo {
  version: string;
  host: string;
  embedded: boolean;
  auth_mode: AuthMode;
  database_path?: string | null;
}

export interface AuthStatus {
  mode: AuthMode;
  owner_configured: boolean;
  paired_clients: number;
  requires_auth: boolean;
}

export interface PairingStart {
  id: string;
  code: string;
  label: string;
  /** The kind of token the code grants: `guest` (Word, Docs) or `desk`. */
  grants: TokenKind;
  expires_at: string;
}

export interface StartPairing {
  label: string;
  grants?: "guest" | "desk";
}

/** A one-minute, single-use credential for opening the change stream. */
export interface StreamTicket {
  ticket: string;
  expires_at: string;
}

export interface PairingComplete {
  token: string;
  token_id: string;
  kind: TokenKind;
  label: string;
  expires_at?: string | null;
  cookie?: boolean;
}

export interface AuthenticatedClient {
  token_id: string | null;
  kind: TokenKind;
  label: string;
}

export interface TokenView {
  id: string;
  kind: TokenKind;
  label: string;
  created_at: string;
  expires_at?: string;
  revoked_at?: string;
  last_used_at?: string;
}

export interface ApiErrorBody {
  error: {
    code: string;
    message: string;
  };
}
