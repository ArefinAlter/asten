/**
 * Drift check: the handwritten contracts in `index.ts` must agree with the
 * types generated from Rust in `generated/`. Each line below is a compile-time
 * assertion that the two are mutually assignable; `tsc` fails when a field is
 * added, removed, renamed or retyped on one side only.
 *
 * Regenerate with `pnpm contracts:gen` after changing a Rust type, then fix
 * the handwritten side (or the Rust side) until this file compiles.
 */
import type * as H from "./index";
import type * as G from "./generated";

type Mutual<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;
// `Check<Mutual<A, B>>` compiles only when A and B are mutually assignable.
type Check<T extends true> = T;

// Identities and projects
export type _Project = Check<Mutual<H.Project, G.domain.Project>>;
export type _CreateProject = Check<
  Mutual<H.CreateProject, G.application.CreateProject>
>;

// Documents
export type _Document = Check<Mutual<H.AstenDocument, G.domain.Document>>;
export type _DocumentSummary = Check<
  Mutual<H.DocumentSummary, G.domain.DocumentSummary>
>;
export type _ScenePlacement = Check<
  Mutual<H.ScenePlacement, G.domain.ScenePlacement>
>;
export type _CreateDocument = Check<
  Mutual<H.CreateDocument, G.application.CreateDocument>
>;
export type _UpdateDocument = Check<
  Mutual<H.UpdateDocument, G.application.UpdateDocument>
>;

// Canon
export type _Fact = Check<Mutual<H.Fact, G.domain.Fact>>;
export type _CreateFact = Check<Mutual<H.CreateFact, G.application.CreateFact>>;
export type _UpdateFact = Check<Mutual<H.UpdateFact, G.application.UpdateFact>>;
export type _CanonEntity = Check<Mutual<H.CanonEntity, G.domain.CanonEntity>>;
export type _CreateCanonEntity = Check<
  Mutual<H.CreateCanonEntity, G.application.CreateCanonEntity>
>;
export type _UpdateCanonEntity = Check<
  Mutual<H.UpdateCanonEntity, G.application.UpdateCanonEntity>
>;
export type _TimelineEvent = Check<
  Mutual<H.TimelineEvent, G.domain.TimelineEvent>
>;
export type _CreateTimelineEvent = Check<
  Mutual<H.CreateTimelineEvent, G.application.CreateTimelineEvent>
>;
export type _UpdateTimelineEvent = Check<
  Mutual<H.UpdateTimelineEvent, G.application.UpdateTimelineEvent>
>;
export type _Suppression = Check<Mutual<H.Suppression, G.domain.Suppression>>;
export type _CreateSuppression = Check<
  Mutual<H.CreateSuppression, G.application.CreateSuppression>
>;
export type _Baseline = Check<Mutual<H.Baseline, G.domain.Baseline>>;
export type _StoredInvariant = Check<
  Mutual<H.StoredInvariant, G.domain.StoredInvariant>
>;
export type _CreateInvariant = Check<
  Mutual<H.CreateInvariant, G.application.CreateInvariant>
>;
export type _StoredRoute = Check<Mutual<H.StoredRoute, G.domain.StoredRoute>>;
export type _CreateRoute = Check<
  Mutual<H.CreateRoute, G.application.CreateRoute>
>;
export type _ProjectSettings = Check<
  Mutual<H.ProjectSettings, G.domain.ProjectSettings>
>;
export type _UpdateSettings = Check<
  Mutual<H.UpdateSettings, G.application.UpdateSettings>
>;

// Story IR pieces that cross the wire
export type _Context = Check<Mutual<H.Context, G.ir.Context>>;
export type _Anchor = Check<Mutual<H.Anchor, G.ir.Anchor>>;
export type _Provenance = Check<Mutual<H.Provenance, G.ir.Provenance>>;
export type _EventKind = Check<Mutual<H.EventKind, G.ir.EventKind>>;
export type _Clock = Check<Mutual<H.Clock, G.time.Clock>>;
export type _Calendar = Check<Mutual<H.Calendar, G.time.Calendar>>;

// Diagnostics and the story language server
export type _Diagnostic = Check<
  Mutual<H.AstenDiagnostic, G.diagnostics.Diagnostic>
>;
export type _Location = Check<Mutual<H.Location, G.diagnostics.Location>>;
export type _Statement = Check<Mutual<H.Statement, G.diagnostics.Statement>>;
export type _Repair = Check<Mutual<H.Repair, G.diagnostics.Repair>>;
export type _RepairKind = Check<Mutual<H.RepairKind, G.diagnostics.RepairKind>>;
export type _Severity = Check<Mutual<H.Severity, G.diagnostics.Severity>>;
export type _DiagnosticsSnapshot = Check<
  Mutual<H.DiagnosticsSnapshot, G.ls.DiagnosticsSnapshot>
>;
export type _DiagnosticsDelta = Check<
  Mutual<H.DiagnosticsDelta, G.ls.DiagnosticsDelta>
>;
export type _StreamEvent = Check<Mutual<H.StreamEvent, G.ls.StreamEvent>>;
export type _FactCard = Check<Mutual<H.FactCard, G.ls.FactCard>>;
export type _MentionRef = Check<Mutual<H.MentionRef, G.ls.MentionRef>>;
export type _WhereSettled = Check<Mutual<H.WhereSettled, G.ls.WhereSettled>>;
export type _RenamePreview = Check<Mutual<H.RenamePreview, G.ls.RenamePreview>>;
export type _SceneSummary = Check<Mutual<H.SceneSummary, G.ls.SceneSummary>>;
export type _CastSearchHit = Check<Mutual<H.CastSearchHit, G.ls.CastSearchHit>>;
export type _ApplyRepairRequest = Check<
  Mutual<H.ApplyRepairRequest, G.ls.ApplyRepairRequest>
>;
export type _ApplyRepairResponse = Check<
  Mutual<H.ApplyRepairResponse, G.ls.ApplyRepairResponse>
>;
export type _BisectResult = Check<Mutual<H.BisectResult, G.ls.BisectResult>>;
export type _CodeInfo = Check<Mutual<H.CodeInfo, G.diagnostics.CodeInfo>>;

// History
export type _EventRecord = Check<Mutual<H.EventRecord, G.blocks.EventRecord>>;
export type _EventPayload = Check<
  Mutual<H.EventPayload, G.blocks.EventPayload>
>;
export type _VerifyReport = Check<
  Mutual<H.VerifyReport, G.application.VerifyReport>
>;
export type _BlameSpan = Check<Mutual<H.BlameSpan, G.blocks.BlameSpan>>;
export type _Draft = Check<Mutual<H.Draft, G.domain.Draft>>;
export type _MergePreview = Check<
  Mutual<H.MergePreview, G.application.MergePreview>
>;
export type _MergeConflict = Check<Mutual<H.MergeConflict, G.blocks.Conflict>>;
export type _Route = Check<Mutual<H.Route, G.ir.Route>>;
export type _ScenePlacement2 = Check<Mutual<H.TimeBounds, G.ir.TimeBounds>>;

// Desk extras
export type _Comment = Check<Mutual<H.Comment, G.domain.Comment>>;
export type _DocumentSnapshot = Check<
  Mutual<H.DocumentSnapshot, G.domain.DocumentSnapshot>
>;
export type _MethodPack = Check<Mutual<H.MethodPack, G.domain.MethodPack>>;
export type _TrackedChange = Check<
  Mutual<H.TrackedChange, G.domain.TrackedChange>
>;
export type _CompileRequest = Check<
  Mutual<H.CompileRequest, G.application.CompileRequest>
>;
export type _CompileArtifact = Check<
  Mutual<H.CompileArtifact, G.domain.CompileArtifact>
>;
export type _ImportRequest = Check<
  Mutual<H.ImportRequest, G.application.ImportRequest>
>;
export type _CompileFormat = Check<
  Mutual<H.CompileFormat, G.application.CompileFormat>
>;
export type _ImportFormat = Check<
  Mutual<H.ImportFormat, G.application.ImportFormat>
>;
export type _WorldEntry = Check<Mutual<H.WorldEntry, G.domain.WorldEntry>>;
export type _CreateWorldEntry = Check<
  Mutual<H.CreateWorldEntry, G.application.CreateWorldEntry>
>;
export type _UpdateWorldEntry = Check<
  Mutual<H.UpdateWorldEntry, G.application.UpdateWorldEntry>
>;
export type _WorldPulse = Check<Mutual<H.WorldPulse, G.domain.WorldPulse>>;

// Auth
export type _DeskInfo = Check<Mutual<H.DeskInfo, G.application.DeskInfo>>;
export type _AuthStatus = Check<Mutual<H.AuthStatus, G.application.AuthStatus>>;
export type _PairingStart = Check<
  Mutual<H.PairingStart, G.application.PairingStart>
>;
export type _PairingComplete = Check<
  Mutual<H.PairingComplete, G.application.PairingComplete>
>;
export type _AuthenticatedClient = Check<
  Mutual<H.AuthenticatedClient, G.application.AuthenticatedClient>
>;
export type _StreamTicket = Check<
  Mutual<H.StreamTicket, G.application.StreamTicketIssued>
>;
export type _TokenKind = Check<Mutual<H.TokenKind, G.domain.TokenKind>>;
export type _AuthMode = Check<Mutual<H.AuthMode, G.auth.AuthMode>>;
