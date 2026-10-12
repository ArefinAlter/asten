//! The story language server protocol (§12.4).
//!
//! Shaped like the Language Server Protocol, adapted from text documents to
//! blocks. Every client speaks it: the desk in-process through Tauri, and the
//! browser, the Word task pane and the Docs sidebar over the loopback HTTP API
//! with server-sent events for push.

use asten_diagnostics::{Diagnostic, MarksDiff, Repair};
use asten_ir::{BlockId, DocId, EntityId, Sort, TimeBounds};
use serde::{Deserialize, Serialize};

/// `story/diagnostics`: the full set at a revision.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct DiagnosticsSnapshot {
    pub revision: String,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub ledger: Vec<Obligation>,
    #[serde(default)]
    pub tiers: Vec<LanguageTier>,
    /// Suppressions whose evidence changed; their marks are back.
    #[serde(default)]
    pub stale_suppressions: Vec<String>,
    #[serde(default)]
    pub timings_ms: Timings,
    /// Count of blocks whose perception came from the cache.
    #[serde(default)]
    pub cached_blocks: usize,
    #[serde(default)]
    pub total_blocks: usize,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct Timings {
    pub perceive: u64,
    pub analyze: u64,
    pub total: u64,
}

/// `story/diagnostics` as a diff since a revision.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct DiagnosticsDelta {
    pub from_revision: String,
    pub revision: String,
    #[serde(flatten)]
    pub diff: MarksDiff,
}

/// A pending obligation from the setup/payoff ledger.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct Obligation {
    pub invariant: String,
    pub label: String,
    pub since: DocId,
    pub since_title: String,
}

/// The support tier for a script seen in the project (§13.2).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct LanguageTier {
    pub script: String,
    pub tier: String,
    pub blocks: usize,
}

/// `story/factCard`: hovering a name.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct FactCard {
    pub entity: EntityId,
    pub name: String,
    pub sort: Option<Sort>,
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Facts that hold at the hovered scene's time, settled first.
    #[serde(default)]
    pub facts: Vec<FactSummary>,
    #[serde(default)]
    pub at_label: String,
    pub mention_count: usize,
    #[serde(default)]
    pub open_marks: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct FactSummary {
    pub id: String,
    pub predicate: String,
    pub value: String,
    pub status: String,
    pub context: String,
    #[serde(default)]
    pub valid_label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settled_in: Option<DocId>,
}

/// `story/mentions`: every mention of an entity.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct MentionRef {
    pub document: DocId,
    pub document_title: String,
    pub block: BlockId,
    pub start: u32,
    pub end: u32,
    pub quote: String,
    pub reading_pos: i64,
    pub confidence: f32,
    pub context: String,
}

/// `story/whereSettled`: jump to where a fact was settled.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct WhereSettled {
    pub fact: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<DocId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block: Option<BlockId>,
    pub start: u32,
    pub end: u32,
    #[serde(default)]
    pub quote: String,
    #[serde(default)]
    pub settled_by: String,
    #[serde(default)]
    pub settled_at: String,
}

/// `story/rename` preview: every change the rename would make.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct RenamePreview {
    pub entity: EntityId,
    pub from: String,
    pub to: String,
    pub changes: Vec<RenameChange>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct RenameChange {
    pub document: DocId,
    pub document_title: String,
    pub block: BlockId,
    pub start: u32,
    pub end: u32,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct RenameRequest {
    pub to: String,
    /// Apply the rename as patch events (true) or only preview (false).
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub apply: bool,
}

/// `story/sceneSummary`: the line above a scene.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct SceneSummary {
    pub document: DocId,
    pub title: String,
    pub reading_pos: i64,
    pub time_label: String,
    pub bounds: TimeBounds,
    pub bounds_label: String,
    pub context: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_run: Option<String>,
    pub mentions: usize,
    pub entities: Vec<EntityId>,
    pub events: usize,
    pub observations: usize,
    pub marks: usize,
    pub script: String,
    pub tier: String,
    pub words: usize,
}

/// `story/castSearch`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct CastSearchHit {
    pub entity: EntityId,
    pub name: String,
    pub sort: Sort,
    pub score: f32,
    pub mentions: usize,
}

/// `story/repairs` application request.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct ApplyRepairRequest {
    pub diagnostic_id: String,
    pub repair_index: usize,
    /// For `Suppress`: the author's reason.
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub reason: String,
    /// For `Suppress`: scope. Defaults to this occurrence.
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct ApplyRepairResponse {
    pub applied: Repair,
    pub message: String,
}

/// `story/bisect`: which edit introduced a mark.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct BisectResult {
    pub diagnostic_id: String,
    pub found: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<DocId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<i64>,
    #[serde(default)]
    pub actor: String,
    #[serde(default)]
    pub at: String,
    #[serde(default)]
    pub event_type: String,
    pub probes: usize,
}

/// A baseline: the marks the author has accepted as known.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
pub struct Baseline {
    pub name: String,
    pub created_at: String,
    pub diagnostic_ids: Vec<String>,
    pub revision: String,
}

/// Server-sent events on the project stream.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, export_to = "ls/"))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// A new revision is available; clients fetch or receive the delta.
    Revision {
        revision: String,
    },
    Diagnostics(DiagnosticsDelta),
    /// A document changed (by this or another client).
    Document {
        document: DocId,
        seq: i64,
    },
    /// The canon changed (facts, entities, events, settings).
    Canon {
        what: String,
    },
    Heartbeat,
}
