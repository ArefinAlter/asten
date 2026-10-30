//! The story language server behind the desk: building engine inputs from the
//! stored canon, running the engine, answering hover/mentions/rename/summary
//! queries, applying repairs, baselines, settings, invariants, routes and bisect.

use std::collections::{HashMap, HashSet};

use asten_blocks::{block_nodes, blocks_of, doc_from_state, fingerprint, plain_text, replace_text};
use asten_diagnostics::{
    RepairKind, Suppression as DiagSuppression, SuppressionScope as DiagScope,
};
use asten_domain::{
    Baseline, BaselineId, CanonEntity, Document, DocumentId, EntityKind, FactModality, FactStatus,
    InvariantId, ProjectId, ProjectSettings, RouteId, ScenePlacement, StoredInvariant, StoredRoute,
    SuppressionScope, WorldKind, entity_key_from_title,
};
use asten_engine::{BlockInput, CraftHint, Engine, ProjectInputs};
use asten_ir::{
    Alias, Anchor, Context, DocId, Entity, EntityId as IrEntityId, Event, EventId, EventKind,
    Fact as IrFact, FactId as IrFactId, Fluent, Invariant, InvariantPattern, PredicateId,
    Provenance, Route, Scene, Settings, Sort, Status, TemporalConstraint, TimeBounds, TimePoint,
    TimeRef, TraceClock,
};
use asten_ls::{
    ApplyRepairRequest, ApplyRepairResponse, BisectResult, CastSearchHit, DiagnosticsSnapshot,
    FactCard, MentionRef, RenamePreview, RenameRequest, SceneSummary, StreamEvent, WhereSettled,
};
use asten_time::{Clock, Parsed, Relation};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{Application, ApplicationError, Result};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct UpdateSettings {
    pub strictness: Option<asten_ir::Strictness>,
    pub narrator_reliable: Option<bool>,
    pub clock: Option<Clock>,
    pub min_parent_age_days: Option<Option<i64>>,
    pub resurrection_allowed: Option<Vec<Sort>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateInvariant {
    pub label: String,
    pub pattern: InvariantPattern,
    #[serde(default)]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub clock: TraceClock,
    #[serde(default = "yes")]
    #[cfg_attr(feature = "ts", ts(as = "Option<_>", optional))]
    pub enabled: bool,
}

fn yes() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateRoute {
    pub route: Route,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(
    feature = "ts",
    derive(ts_rs::TS),
    ts(export, export_to = "application/")
)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "ts", ts(optional_fields = nullable))]
pub struct CreateBaseline {
    pub name: String,
}

impl Application {
    // ───────────────────────── settings ─────────────────────────

    pub async fn get_settings(&self, project_id: ProjectId) -> Result<ProjectSettings> {
        self.get_project(project_id).await?;
        Ok(self
            .repository
            .get_settings(project_id)
            .await?
            .unwrap_or_else(|| ProjectSettings::for_project(project_id)))
    }

    pub async fn update_settings(
        &self,
        project_id: ProjectId,
        input: UpdateSettings,
    ) -> Result<ProjectSettings> {
        let mut settings = self.get_settings(project_id).await?;
        if let Some(strictness) = input.strictness {
            settings.strictness = strictness;
        }
        if let Some(reliable) = input.narrator_reliable {
            settings.narrator_reliable = reliable;
        }
        if let Some(clock) = input.clock {
            if let asten_time::calendar::Calendar::Fictional(calendar) = &clock.calendar {
                calendar.validate().map_err(ApplicationError::Validation)?;
            }
            settings.clock = clock;
        }
        if let Some(min) = input.min_parent_age_days {
            settings.min_parent_age_days = min;
        }
        if let Some(sorts) = input.resurrection_allowed {
            settings.resurrection_allowed = sorts;
        }
        settings.updated_at = Utc::now();
        self.repository.put_settings(&settings).await?;
        self.schedule_refresh(project_id);
        Ok(settings)
    }

    // ───────────────────────── invariants, routes, baselines ─────────────────────────

    pub async fn list_invariants(&self, project_id: ProjectId) -> Result<Vec<StoredInvariant>> {
        self.get_project(project_id).await?;
        self.repository.list_invariants(project_id).await
    }

    pub async fn create_invariant(
        &self,
        project_id: ProjectId,
        input: CreateInvariant,
    ) -> Result<StoredInvariant> {
        self.get_project(project_id).await?;
        let label = input.label.trim();
        if label.is_empty() {
            return Err(ApplicationError::Validation("give the rule a name".into()));
        }
        let id = InvariantId::new();
        let stored = StoredInvariant {
            id,
            project_id,
            invariant: Invariant {
                id: id.to_string(),
                label: label.to_owned(),
                pattern: input.pattern,
                clock: input.clock,
                enabled: input.enabled,
            },
            created_at: Utc::now(),
        };
        self.repository.insert_invariant(&stored).await?;
        self.schedule_refresh(project_id);
        Ok(stored)
    }

    pub async fn update_invariant(
        &self,
        project_id: ProjectId,
        id: InvariantId,
        invariant: Invariant,
    ) -> Result<StoredInvariant> {
        let stored = self
            .repository
            .list_invariants(project_id)
            .await?
            .into_iter()
            .find(|stored| stored.id == id)
            .ok_or(ApplicationError::NotFound {
                entity: "invariant",
            })?;
        let updated = StoredInvariant {
            invariant: Invariant {
                id: id.to_string(),
                ..invariant
            },
            ..stored
        };
        self.repository.update_invariant(&updated).await?;
        self.schedule_refresh(project_id);
        Ok(updated)
    }

    pub async fn delete_invariant(&self, project_id: ProjectId, id: InvariantId) -> Result<()> {
        self.repository.delete_invariant(id).await?;
        self.schedule_refresh(project_id);
        Ok(())
    }

    pub async fn list_routes(&self, project_id: ProjectId) -> Result<Vec<StoredRoute>> {
        self.get_project(project_id).await?;
        self.repository.list_routes(project_id).await
    }

    pub async fn create_route(
        &self,
        project_id: ProjectId,
        input: CreateRoute,
    ) -> Result<StoredRoute> {
        self.get_project(project_id).await?;
        if input.route.min_ticks < 0 {
            return Err(ApplicationError::Validation(
                "travel time cannot be negative".into(),
            ));
        }
        let stored = StoredRoute {
            id: RouteId::new(),
            project_id,
            route: input.route,
            created_at: Utc::now(),
        };
        self.repository.insert_route(&stored).await?;
        self.schedule_refresh(project_id);
        Ok(stored)
    }

    pub async fn delete_route(&self, project_id: ProjectId, id: RouteId) -> Result<()> {
        self.repository.delete_route(id).await?;
        self.schedule_refresh(project_id);
        Ok(())
    }

    pub async fn list_baselines(&self, project_id: ProjectId) -> Result<Vec<Baseline>> {
        self.get_project(project_id).await?;
        self.repository.list_baselines(project_id).await
    }

    /// Freeze today's marks: only new ones are highlighted from now on.
    pub async fn create_baseline(
        &self,
        project_id: ProjectId,
        input: CreateBaseline,
    ) -> Result<Baseline> {
        let snapshot = self.analyze_project(project_id).await?;
        let name = input.name.trim();
        let baseline = Baseline {
            id: BaselineId::new(),
            project_id,
            name: if name.is_empty() {
                format!("Baseline {}", Utc::now().format("%Y-%m-%d"))
            } else {
                name.to_owned()
            },
            revision: snapshot.revision,
            diagnostic_ids: snapshot
                .diagnostics
                .iter()
                .map(|mark| mark.id.clone())
                .collect(),
            active: true,
            created_at: Utc::now(),
        };
        for mut other in self.repository.list_baselines(project_id).await? {
            if other.active {
                other.active = false;
                self.repository.update_baseline(&other).await?;
            }
        }
        self.repository.insert_baseline(&baseline).await?;
        self.schedule_refresh(project_id);
        Ok(baseline)
    }

    pub async fn activate_baseline(
        &self,
        project_id: ProjectId,
        id: Option<BaselineId>,
    ) -> Result<Vec<Baseline>> {
        let mut baselines = self.repository.list_baselines(project_id).await?;
        for baseline in &mut baselines {
            let active = Some(baseline.id) == id;
            if baseline.active != active {
                baseline.active = active;
                self.repository.update_baseline(baseline).await?;
            }
        }
        self.schedule_refresh(project_id);
        Ok(baselines)
    }

    pub async fn delete_baseline(&self, project_id: ProjectId, id: BaselineId) -> Result<()> {
        self.repository.delete_baseline(id).await?;
        self.schedule_refresh(project_id);
        Ok(())
    }

    // ───────────────────────── inputs ─────────────────────────

    /// Everything the engine reads, built from storage.
    pub async fn build_inputs(&self, project_id: ProjectId) -> Result<ProjectInputs> {
        self.get_project(project_id).await?;
        let documents = self.repository.list_documents(project_id).await?;
        let entities = self.repository.list_entities(project_id).await?;
        let world = self.repository.list_world_entries(project_id).await?;
        let facts = self.repository.list_facts(project_id).await?;
        let events = self.repository.list_events(project_id).await?;
        let settings = self.get_settings(project_id).await?;
        let invariants = self.repository.list_invariants(project_id).await?;
        let routes = self.repository.list_routes(project_id).await?;
        let suppressions = self.repository.list_suppressions(project_id).await?;
        let baselines = self.repository.list_baselines(project_id).await?;
        let packs = self.repository.list_packs().await?;
        Ok(assemble_inputs(
            &documents,
            &entities,
            &world,
            &facts,
            &events,
            &settings,
            &invariants,
            &routes,
            &suppressions,
            &baselines,
            &packs,
        ))
    }

    /// Analyse the project with its cached engine and push the delta.
    pub async fn analyze_project(&self, project_id: ProjectId) -> Result<DiagnosticsSnapshot> {
        let inputs = self.build_inputs(project_id).await?;
        let engines = self.engines.clone();
        let outcome = tokio::task::spawn_blocking(move || {
            let mut engines = engines
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let engine = engines.entry(project_id).or_default();
            engine.analyze(&inputs)
        })
        .await
        .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        // Stale suppressions are recorded so the desk can show "previously silenced".
        if !outcome.snapshot.stale_suppressions.is_empty() {
            for mut suppression in self.repository.list_suppressions(project_id).await? {
                let stale = outcome
                    .snapshot
                    .stale_suppressions
                    .contains(&suppression.id.to_string());
                if stale != suppression.stale {
                    suppression.stale = stale;
                    self.repository.update_suppression(&suppression).await?;
                }
            }
        }
        // A read re-analyses the revision the engine already had. Announcing
        // that as a new revision would send every subscriber back to fetch,
        // which re-analyses, which announces again, without end.
        let revision_changed = outcome
            .delta
            .as_ref()
            .is_none_or(|delta| delta.from_revision != delta.revision);
        if let Some(delta) = outcome.delta {
            if !(delta.diff.added.is_empty()
                && delta.diff.removed.is_empty()
                && delta.diff.changed.is_empty())
            {
                self.notify(project_id, StreamEvent::Diagnostics(delta));
            }
        }
        if revision_changed {
            self.notify(
                project_id,
                StreamEvent::Revision {
                    revision: outcome.snapshot.revision.clone(),
                },
            );
        }
        Ok(outcome.snapshot)
    }

    /// Analyse on a private engine without updating suppressions, shared caches, or streams.
    /// MCP callers use this because they must not persist story changes.
    pub async fn analyze_project_readonly(
        &self,
        project_id: ProjectId,
    ) -> Result<DiagnosticsSnapshot> {
        let inputs = self.build_inputs(project_id).await?;
        tokio::task::spawn_blocking(move || Engine::new().analyze(&inputs).snapshot)
            .await
            .map_err(|error| ApplicationError::Storage(error.to_string()))
    }

    /// Query cast data without changing the shared analysis engine or repository.
    pub async fn cast_search_readonly(
        &self,
        project_id: ProjectId,
        query: &str,
    ) -> Result<Vec<CastSearchHit>> {
        let inputs = self.build_inputs(project_id).await?;
        let query = query.to_owned();
        tokio::task::spawn_blocking(move || {
            let mut engine = Engine::new();
            engine.analyze(&inputs);
            engine
                .last_ir()
                .map(|ir| asten_engine::cast_search(ir, &query, 12))
                .unwrap_or_default()
        })
        .await
        .map_err(|error| ApplicationError::Storage(error.to_string()))
    }

    /// Summarize a scene without changing the shared analysis engine or repository.
    pub async fn scene_summary_readonly(
        &self,
        project_id: ProjectId,
        document_id: DocumentId,
    ) -> Result<SceneSummary> {
        let inputs = self.build_inputs(project_id).await?;
        let document = DocId::new(document_id.to_string());
        tokio::task::spawn_blocking(move || {
            let mut engine = Engine::new();
            engine.analyze(&inputs);
            engine
                .last_ir()
                .and_then(|ir| {
                    asten_engine::scene_summary(
                        ir,
                        &inputs,
                        engine.last_perceptions(),
                        engine.last_diagnostics(),
                        &document,
                    )
                })
                .ok_or(ApplicationError::NotFound { entity: "scene" })
        })
        .await
        .map_err(|error| ApplicationError::Storage(error.to_string()))?
    }

    /// Analyse with one document's content replaced, without touching the
    /// project's engine or its delta. Used by merge previews and bisect.
    pub async fn analyze_with_override(
        &self,
        project_id: ProjectId,
        document_id: DocumentId,
        content: serde_json::Value,
    ) -> Result<DiagnosticsSnapshot> {
        let mut inputs = self.build_inputs(project_id).await?;
        override_document(&mut inputs, document_id, &content);
        let outcome = tokio::task::spawn_blocking(move || Engine::new().analyze(&inputs))
            .await
            .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        Ok(outcome.snapshot)
    }

    async fn ensure_analyzed(&self, project_id: ProjectId) -> Result<()> {
        let has_ir = {
            let engines = self
                .engines
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            engines
                .get(&project_id)
                .is_some_and(|engine| engine.last_ir().is_some())
        };
        if !has_ir {
            self.analyze_project(project_id).await?;
        }
        Ok(())
    }

    fn with_engine<R>(&self, project_id: ProjectId, f: impl FnOnce(&Engine) -> R) -> Option<R> {
        let engines = self
            .engines
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        engines.get(&project_id).map(f)
    }

    // ───────────────────────── queries ─────────────────────────

    pub async fn fact_card(
        &self,
        project_id: ProjectId,
        entity_key: &str,
        at_document: Option<DocumentId>,
    ) -> Result<FactCard> {
        self.ensure_analyzed(project_id).await?;
        let key = IrEntityId::new(entity_key_from_title(entity_key));
        let at = at_document.map(|id| DocId::new(id.to_string()));
        self.with_engine(project_id, |engine| {
            engine.last_ir().and_then(|ir| {
                asten_engine::fact_card(ir, engine.last_diagnostics(), &key, at.as_ref())
            })
        })
        .flatten()
        .ok_or(ApplicationError::NotFound { entity: "entity" })
    }

    pub async fn mentions(
        &self,
        project_id: ProjectId,
        entity_key: &str,
    ) -> Result<Vec<MentionRef>> {
        self.ensure_analyzed(project_id).await?;
        let key = IrEntityId::new(entity_key_from_title(entity_key));
        Ok(self
            .with_engine(project_id, |engine| {
                engine
                    .last_ir()
                    .map(|ir| asten_engine::mentions_of(ir, &key))
            })
            .flatten()
            .unwrap_or_default())
    }

    pub async fn where_settled(
        &self,
        project_id: ProjectId,
        fact_id: &str,
    ) -> Result<WhereSettled> {
        self.ensure_analyzed(project_id).await?;
        self.with_engine(project_id, |engine| {
            engine
                .last_ir()
                .and_then(|ir| asten_engine::where_settled(ir, fact_id))
        })
        .flatten()
        .ok_or(ApplicationError::NotFound { entity: "fact" })
    }

    pub async fn scene_summary(
        &self,
        project_id: ProjectId,
        document_id: DocumentId,
    ) -> Result<SceneSummary> {
        self.ensure_analyzed(project_id).await?;
        let inputs = self.build_inputs(project_id).await?;
        let document = DocId::new(document_id.to_string());
        self.with_engine(project_id, |engine| {
            engine.last_ir().and_then(|ir| {
                asten_engine::scene_summary(
                    ir,
                    &inputs,
                    engine.last_perceptions(),
                    engine.last_diagnostics(),
                    &document,
                )
            })
        })
        .flatten()
        .ok_or(ApplicationError::NotFound { entity: "scene" })
    }

    pub async fn cast_search(
        &self,
        project_id: ProjectId,
        query: &str,
    ) -> Result<Vec<CastSearchHit>> {
        self.ensure_analyzed(project_id).await?;
        Ok(self
            .with_engine(project_id, |engine| {
                engine
                    .last_ir()
                    .map(|ir| asten_engine::cast_search(ir, query, 12))
            })
            .flatten()
            .unwrap_or_default())
    }

    /// Rename an entity everywhere. With `apply`, the prose changes land as
    /// ordinary patch events and the cast entry is renamed.
    pub async fn rename_entity(
        &self,
        project_id: ProjectId,
        entity_id: asten_domain::EntityId,
        request: RenameRequest,
    ) -> Result<RenamePreview> {
        let entity = self.get_entity(entity_id).await?;
        let to = request.to.trim();
        if to.is_empty() {
            return Err(ApplicationError::Validation(
                "a new name is required".into(),
            ));
        }
        self.ensure_analyzed(project_id).await?;
        let inputs = self.build_inputs(project_id).await?;
        let key = IrEntityId::new(entity.key());
        let preview = self
            .with_engine(project_id, |engine| {
                engine
                    .last_ir()
                    .and_then(|ir| asten_engine::rename_preview(ir, &inputs, &key, to))
            })
            .flatten()
            .ok_or(ApplicationError::NotFound { entity: "entity" })?;
        if !request.apply {
            return Ok(preview);
        }
        // Group changes per document; apply right-to-left inside each block so
        // earlier offsets stay valid.
        let mut per_document: HashMap<String, Vec<&asten_ls::RenameChange>> = HashMap::new();
        for change in &preview.changes {
            per_document
                .entry(change.document.0.clone())
                .or_default()
                .push(change);
        }
        for (document_key, mut changes) in per_document {
            let Ok(uuid) = document_key.parse::<uuid::Uuid>() else {
                continue;
            };
            let document_id = DocumentId(uuid);
            let document = self.get_document(document_id).await?;
            let mut content = document.content_json.clone();
            changes.sort_by(|a, b| {
                b.block
                    .0
                    .cmp(&a.block.0)
                    .then_with(|| b.start.cmp(&a.start))
            });
            if let Some(blocks) = content
                .get_mut("content")
                .and_then(serde_json::Value::as_array_mut)
            {
                for change in changes {
                    if let Some(node) = blocks
                        .iter_mut()
                        .find(|node| asten_blocks::node_id(node) == Some(change.block.0.as_str()))
                    {
                        replace_text(node, change.start, change.end, &change.after);
                    }
                }
            }
            self.update_document(
                document_id,
                crate::UpdateDocument {
                    content_json: Some(content),
                    actor: Some(format!("rename:{}→{}", preview.from, to)),
                    ..Default::default()
                },
            )
            .await?;
        }
        // The cast entry: the old name becomes an alias so older mentions still link.
        let mut aliases = entity.aliases.clone();
        if !aliases
            .iter()
            .any(|alias| alias.eq_ignore_ascii_case(&entity.name))
        {
            aliases.push(entity.name.clone());
        }
        self.update_entity(
            entity_id,
            crate::UpdateCanonEntity {
                name: Some(to.to_owned()),
                aliases: Some(aliases),
                ..Default::default()
            },
        )
        .await?;
        Ok(preview)
    }

    // ───────────────────────── repairs ─────────────────────────

    pub async fn apply_repair(
        &self,
        project_id: ProjectId,
        request: ApplyRepairRequest,
    ) -> Result<ApplyRepairResponse> {
        self.ensure_analyzed(project_id).await?;
        let diagnostic = self
            .with_engine(project_id, |engine| {
                engine
                    .last_diagnostics()
                    .iter()
                    .find(|mark| mark.id == request.diagnostic_id)
                    .cloned()
            })
            .flatten()
            .ok_or(ApplicationError::NotFound {
                entity: "diagnostic",
            })?;
        let repair = diagnostic
            .repairs
            .get(request.repair_index)
            .cloned()
            .ok_or(ApplicationError::NotFound { entity: "repair" })?;
        let settings = self.get_settings(project_id).await?;
        let message = match &repair.kind {
            RepairKind::AddChangeEvent {
                entity,
                predicate,
                to_value,
                after_tick,
                before_tick,
            } => {
                let row = self.entity_by_key(project_id, &entity.0).await?;
                let value: asten_ir::Value =
                    serde_json::from_value(to_value.clone()).unwrap_or(asten_ir::Value::Text {
                        norm: to_value.to_string(),
                        label: to_value.to_string(),
                    });
                let tick = before_tick
                    .map(|tick| tick - 1)
                    .or(after_tick.map(|tick| tick + 1));
                let name = row
                    .as_ref()
                    .map(|row| row.name.clone())
                    .unwrap_or_else(|| entity.0.clone());
                self.create_event(crate::CreateTimelineEvent {
                    project_id,
                    title: format!(
                        "{name}'s {} changes to {}",
                        predicate.replace('_', " "),
                        value.label()
                    ),
                    entity_id: row.map(|row| row.id),
                    document_id: diagnostic
                        .primary
                        .as_ref()
                        .and_then(|location| location.document.0.parse().ok())
                        .map(DocumentId),
                    story_time: tick.map(|tick| settings.clock.format(tick)),
                    reading_order: None,
                    duration: None,
                    notes: format!("Added from mark {}", diagnostic.code),
                    kind: Some(EventKind::Changes {
                        fluent: Fluent {
                            predicate: PredicateId::new(predicate.clone()),
                            subject: entity.clone(),
                            value,
                        },
                    }),
                    participants: Vec::new(),
                    context: None,
                    status: Some(FactStatus::Confirmed),
                    tick,
                    tick_lo: None,
                    tick_hi: None,
                    anchor: None,
                })
                .await?;
                "Added a change event. The engine will re-check.".to_owned()
            }
            RepairKind::MarkContext { document, context } => {
                let document_id = parse_doc(document)?;
                let current = self.get_document(document_id).await?;
                self.update_document(
                    document_id,
                    crate::UpdateDocument {
                        scene: Some(ScenePlacement {
                            context: context.clone(),
                            ..current.scene
                        }),
                        ..Default::default()
                    },
                )
                .await?;
                format!("Marked the scene as {}.", context.label())
            }
            RepairKind::SetValidity {
                fact,
                from_tick,
                to_tick,
            } => {
                let fact_id = parse_fact(fact)?;
                self.update_fact(
                    fact_id,
                    crate::UpdateFact {
                        valid_from: from_tick.map(|tick| Some(settings.clock.format(tick))),
                        valid_to: to_tick.map(|tick| Some(settings.clock.format(tick))),
                        ..Default::default()
                    },
                )
                .await?;
                "Bounded the fact in story time.".to_owned()
            }
            RepairKind::PlaceScene { document, lo, hi } => {
                let document_id = parse_doc(document)?;
                let current = self.get_document(document_id).await?;
                let (tick, tick_lo, tick_hi) = match (lo, hi) {
                    (Some(lo), Some(hi)) if lo == hi => (Some(*lo), None, None),
                    _ => (None, *lo, *hi),
                };
                self.update_document(
                    document_id,
                    crate::UpdateDocument {
                        scene: Some(ScenePlacement {
                            tick,
                            tick_lo,
                            tick_hi,
                            ..current.scene
                        }),
                        ..Default::default()
                    },
                )
                .await?;
                "Placed the scene. Refine its time in the scene header.".to_owned()
            }
            RepairKind::MoveEvent { event, lo, hi } => {
                let event_id = parse_event(event)?;
                let (tick, tick_lo, tick_hi) = match (lo, hi) {
                    (Some(lo), Some(hi)) if lo == hi => (Some(Some(*lo)), Some(None), Some(None)),
                    _ => (Some(None), Some(*lo), Some(*hi)),
                };
                self.update_event(
                    event_id,
                    crate::UpdateTimelineEvent {
                        tick,
                        tick_lo,
                        tick_hi,
                        ..Default::default()
                    },
                )
                .await?;
                "Moved the event in story time.".to_owned()
            }
            RepairKind::DeprecateFact { fact } => {
                self.update_fact(
                    parse_fact(fact)?,
                    crate::UpdateFact {
                        status: Some(FactStatus::Deprecated),
                        ..Default::default()
                    },
                )
                .await?;
                "Retired the fact. It stays in history.".to_owned()
            }
            RepairKind::SettleFact { fact } => {
                self.update_fact(
                    parse_fact(fact)?,
                    crate::UpdateFact {
                        status: Some(FactStatus::Confirmed),
                        ..Default::default()
                    },
                )
                .await?;
                "Settled.".to_owned()
            }
            RepairKind::AddEntity { name, sort } => {
                self.create_entity(crate::CreateCanonEntity {
                    project_id,
                    kind: match sort {
                        Sort::Place => EntityKind::Place,
                        Sort::Item => EntityKind::Item,
                        Sort::Event => EntityKind::Event,
                        Sort::Rule => EntityKind::Rule,
                        Sort::Source => EntityKind::Source,
                        Sort::Character | Sort::Group => EntityKind::Character,
                        _ => EntityKind::Other,
                    },
                    name: name.clone(),
                    aliases: Vec::new(),
                    notes: String::new(),
                    introduced_at: None,
                })
                .await?;
                format!("Added {name} to the cast.")
            }
            RepairKind::AddAlias { entity, alias } => {
                let row = self
                    .entity_by_key(project_id, &entity.0)
                    .await?
                    .ok_or(ApplicationError::NotFound { entity: "entity" })?;
                let mut aliases = row.aliases.clone();
                aliases.push(alias.clone());
                self.update_entity(
                    row.id,
                    crate::UpdateCanonEntity {
                        aliases: Some(aliases),
                        ..Default::default()
                    },
                )
                .await?;
                format!("{alias} is now an alias of {}.", row.name)
            }
            RepairKind::ClearContinuity { document } => {
                let document_id = parse_doc(document)?;
                let current = self.get_document(document_id).await?;
                self.update_document(
                    document_id,
                    crate::UpdateDocument {
                        scene: Some(ScenePlacement {
                            continuity_run: None,
                            ..current.scene
                        }),
                        ..Default::default()
                    },
                )
                .await?;
                "Took the scene out of the continuous run.".to_owned()
            }
            RepairKind::Suppress => {
                let scope = match request.scope.as_deref() {
                    Some("entity") => SuppressionScope::Entity,
                    Some("document") => SuppressionScope::Document,
                    Some("project") => SuppressionScope::Project,
                    _ => SuppressionScope::Occurrence,
                };
                self.create_suppression(crate::CreateSuppression {
                    project_id,
                    rule_code: diagnostic.code.clone(),
                    document_id: diagnostic
                        .primary
                        .as_ref()
                        .and_then(|location| location.document.0.parse().ok())
                        .map(DocumentId),
                    entity_key: diagnostic.entity.as_ref().map(|entity| entity.0.clone()),
                    reason: request.reason.clone(),
                    scope: Some(scope),
                    diagnostic_id: Some(diagnostic.id.clone()),
                    evidence_fingerprint: Some(diagnostic.evidence_fingerprint.clone()),
                })
                .await?;
                "Silenced. If the evidence changes, the mark comes back for review.".to_owned()
            }
            RepairKind::Keep => "Kept as written.".to_owned(),
        };
        self.schedule_refresh(project_id);
        Ok(ApplyRepairResponse {
            applied: repair,
            message,
        })
    }

    // ───────────────────────── bisect ─────────────────────────

    /// Which edit introduced a mark: binary search over the document's history.
    pub async fn bisect(&self, project_id: ProjectId, diagnostic_id: &str) -> Result<BisectResult> {
        self.ensure_analyzed(project_id).await?;
        let diagnostic = self
            .with_engine(project_id, |engine| {
                engine
                    .last_diagnostics()
                    .iter()
                    .find(|mark| mark.id == diagnostic_id)
                    .cloned()
            })
            .flatten()
            .ok_or(ApplicationError::NotFound {
                entity: "diagnostic",
            })?;
        let Some(location) = diagnostic.primary.clone() else {
            return Ok(BisectResult {
                diagnostic_id: diagnostic_id.to_owned(),
                found: false,
                ..BisectResult::default()
            });
        };
        let document_id = parse_doc(&location.document)?;
        let document = self.get_document(document_id).await?;
        let records = self.repository.list_block_events(document_id, 0).await?;
        let mut result = BisectResult {
            diagnostic_id: diagnostic_id.to_owned(),
            document: Some(location.document.clone()),
            ..BisectResult::default()
        };
        if records.is_empty() {
            return Ok(result);
        }
        let mut probes = 0usize;
        let mut present_at = |seq: i64| -> bool {
            let _ = seq;
            false
        };
        let _ = &mut present_at;
        let mut lo = 0i64; // known absent (empty document)
        let mut hi = document.head_seq; // known present
        while hi - lo > 1 && probes < 24 {
            let mid = (lo + hi) / 2;
            let state = self.state_at(document_id, mid).await?;
            let content = doc_from_state(&state);
            let snapshot = self
                .analyze_with_override(project_id, document_id, content)
                .await?;
            probes += 1;
            if snapshot
                .diagnostics
                .iter()
                .any(|mark| mark.id == diagnostic_id)
            {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        result.probes = probes;
        if let Some(record) = records.iter().find(|record| record.seq == hi) {
            result.found = true;
            result.seq = Some(record.seq);
            result.actor = record.actor.clone();
            result.at = record.at.clone();
            result.event_type = record.payload.event_type().to_owned();
        }
        Ok(result)
    }

    /// Facts the engine proposes that the author has not reviewed.
    pub async fn proposals(&self, project_id: ProjectId) -> Result<Vec<asten_domain::Fact>> {
        let facts = self.list_facts(project_id).await?;
        Ok(facts
            .into_iter()
            .filter(|fact| fact.status == FactStatus::Proposed)
            .collect())
    }
}

fn parse_doc(document: &DocId) -> Result<DocumentId> {
    document
        .0
        .parse::<uuid::Uuid>()
        .map(DocumentId)
        .map_err(|_| ApplicationError::Validation("not a document id".into()))
}

fn parse_fact(fact: &str) -> Result<asten_domain::FactId> {
    fact.parse::<uuid::Uuid>()
        .map(asten_domain::FactId)
        .map_err(|_| ApplicationError::Validation("not a fact id".into()))
}

fn parse_event(event: &str) -> Result<asten_domain::TimelineEventId> {
    event
        .parse::<uuid::Uuid>()
        .map(asten_domain::TimelineEventId)
        .map_err(|_| ApplicationError::Validation("not an event id".into()))
}

/// Replace one document's blocks in prepared inputs.
pub fn override_document(
    inputs: &mut ProjectInputs,
    document_id: DocumentId,
    content: &serde_json::Value,
) {
    let doc = DocId::new(document_id.to_string());
    let reading_pos = inputs
        .blocks
        .iter()
        .find(|block| block.document == doc)
        .map(|block| block.reading_pos)
        .or_else(|| {
            inputs
                .scenes
                .iter()
                .find(|scene| scene.document == doc)
                .map(|scene| scene.reading_pos)
        })
        .unwrap_or(0);
    inputs.blocks.retain(|block| block.document != doc);
    inputs
        .blocks
        .extend(blocks_of(content).into_iter().map(|block| BlockInput {
            document: doc.clone(),
            block: asten_ir::BlockId::new(block.id),
            text: block.text,
            fingerprint: fingerprint(&block.node),
            reading_pos,
        }));
    inputs.blocks.sort_by(|a, b| {
        a.reading_pos
            .cmp(&b.reading_pos)
            .then_with(|| a.document.0.cmp(&b.document.0))
    });
    inputs.revision = format!("{}+override", inputs.revision);
}

/// Binder reading order: depth-first by parent and position. Children are
/// grouped once, so a binder of n documents costs n log n rather than n².
fn reading_order(documents: &[Document]) -> Vec<&Document> {
    let mut children: HashMap<Option<DocumentId>, Vec<&Document>> = HashMap::new();
    for document in documents {
        children
            .entry(document.parent_id)
            .or_default()
            .push(document);
    }
    for siblings in children.values_mut() {
        siblings.sort_by(|a, b| {
            a.position
                .cmp(&b.position)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });
    }
    let mut out = Vec::with_capacity(documents.len());
    let mut seen: HashSet<DocumentId> = HashSet::with_capacity(documents.len());
    // A stack of what is still to visit, first sibling on top.
    let mut pending: Vec<&Document> = children
        .get(&None)
        .map(|roots| roots.iter().rev().copied().collect())
        .unwrap_or_default();
    while let Some(document) = pending.pop() {
        if !seen.insert(document.id) {
            continue;
        }
        out.push(document);
        if let Some(descendants) = children.get(&Some(document.id)) {
            pending.extend(descendants.iter().rev().copied());
        }
    }
    // Orphans whose parent is missing still get a place.
    for document in documents {
        if seen.insert(document.id) {
            out.push(document);
        }
    }
    out
}

struct TimeContext<'a> {
    clock: &'a Clock,
    /// Folded label → time point, for resolving "after the siege".
    anchors: Vec<(String, TimePoint)>,
}

impl TimeContext<'_> {
    fn resolve_anchor(&self, phrase: &str) -> TimePoint {
        let folded = asten_perceive::matching_form(phrase);
        self.anchors
            .iter()
            .find(|(label, _)| *label == folded)
            .or_else(|| {
                self.anchors
                    .iter()
                    .find(|(label, _)| label.contains(&folded) || folded.contains(label.as_str()))
            })
            .map(|(_, point)| point.clone())
            .unwrap_or(TimePoint::Named {
                label: phrase.to_owned(),
            })
    }

    /// A relative label becomes a constraint between this point and its anchor.
    #[allow(clippy::too_many_arguments)]
    fn relative_constraint(
        &self,
        this: TimePoint,
        relation: Relation,
        lo: i64,
        hi: i64,
        anchor: Option<&str>,
        previous: Option<&TimePoint>,
        label: &str,
        status: Status,
    ) -> Option<TemporalConstraint> {
        let anchor_point = match anchor {
            Some(phrase) => self.resolve_anchor(phrase),
            None => previous?.clone(),
        };
        let per_day = self.clock.ticks_per_day();
        let (from, to, lo, hi) = match relation {
            Relation::After => (anchor_point, this, Some(lo), Some(hi)),
            Relation::Before => (this, anchor_point, Some(lo), Some(hi)),
            Relation::Within => (anchor_point, this, Some(-(per_day - 1)), Some(per_day - 1)),
            Relation::Same => (anchor_point, this, Some(0), Some(0)),
        };
        Some(TemporalConstraint {
            from,
            to,
            lo,
            hi,
            label: label.to_owned(),
            provenance: Provenance::Author {
                action: "story time".into(),
            },
            status,
            confidence: 1.0,
            anchor: None,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn assemble_inputs(
    documents: &[Document],
    entities: &[CanonEntity],
    world: &[asten_domain::WorldEntry],
    facts: &[asten_domain::Fact],
    events: &[asten_domain::TimelineEvent],
    settings: &ProjectSettings,
    invariants: &[StoredInvariant],
    routes: &[StoredRoute],
    suppressions: &[asten_domain::Suppression],
    baselines: &[Baseline],
    packs: &[asten_domain::MethodPack],
) -> ProjectInputs {
    let clock = &settings.clock;
    let ordered = reading_order(documents);
    let prose: Vec<(&Document, i64)> = ordered
        .iter()
        .filter(|document| document.kind.is_prose() && !document.exclude_from_compile)
        .enumerate()
        .map(|(index, document)| (*document, index as i64 + 1))
        .collect();

    // Anchors for relative labels: event titles and scene titles.
    let mut anchors: Vec<(String, TimePoint)> = Vec::new();
    for event in events {
        anchors.push((
            asten_perceive::matching_form(&event.title),
            TimePoint::Event {
                id: EventId::new(event.id.to_string()),
            },
        ));
    }
    for (document, _) in &prose {
        anchors.push((
            asten_perceive::matching_form(&document.title),
            TimePoint::Scene {
                document: DocId::new(document.id.to_string()),
            },
        ));
    }
    let time = TimeContext { clock, anchors };

    let mut scenes = Vec::new();
    let mut blocks = Vec::new();
    let mut constraints = Vec::new();
    let mut previous_point: Option<TimePoint> = None;
    for (document, reading_pos) in &prose {
        let doc = DocId::new(document.id.to_string());
        let point = TimePoint::Scene {
            document: doc.clone(),
        };
        let placement = &document.scene;
        let at = if let Some(tick) = placement.tick.map(clamp_tick) {
            TimeRef::Tick { tick }
        } else if placement.tick_lo.is_some() || placement.tick_hi.is_some() {
            TimeRef::Bounds {
                lo: placement.tick_lo.map(clamp_tick),
                hi: placement.tick_hi.map(clamp_tick),
            }
        } else {
            match placement
                .story_time
                .as_deref()
                .and_then(|label| clock.parse(label))
            {
                Some(Parsed::Absolute(bounds)) => TimeRef::Bounds {
                    lo: bounds.lo,
                    hi: bounds.hi,
                },
                Some(Parsed::Relative {
                    relation,
                    lo,
                    hi,
                    anchor,
                }) => {
                    if let Some(constraint) = time.relative_constraint(
                        point.clone(),
                        relation,
                        lo,
                        hi,
                        anchor.as_deref(),
                        previous_point.as_ref(),
                        &format!(
                            "{}: \u{201c}{}\u{201d}",
                            document.title,
                            placement.story_time.clone().unwrap_or_default()
                        ),
                        Status::Settled,
                    ) {
                        constraints.push(constraint);
                    }
                    TimeRef::Unknown
                }
                None => TimeRef::Unknown,
            }
        };
        scenes.push(Scene {
            document: doc.clone(),
            title: document.title.clone(),
            reading_pos: *reading_pos,
            at,
            time_label: placement.story_time.clone().unwrap_or_default(),
            context: placement.context.clone(),
            continuity_run: placement.continuity_run.clone(),
            language: placement.language.clone(),
            blocks: Vec::new(),
        });
        // The nodes are read in place: only the id, text and fingerprint of
        // each block are kept.
        for (id, node) in block_nodes(&document.content_json) {
            blocks.push(BlockInput {
                document: doc.clone(),
                block: asten_ir::BlockId::new(id),
                text: plain_text(node),
                fingerprint: fingerprint(node),
                reading_pos: *reading_pos,
            });
        }
        previous_point = Some(point);
    }

    // Entities: the cast plus world places and items. The first prose
    // document that says it introduces a key wins, as reading order has it.
    let mut introduced: HashMap<&str, i64> = HashMap::new();
    for (document, reading_pos) in &prose {
        for key in &document.scene.introduces {
            introduced.entry(key.as_str()).or_insert(*reading_pos);
        }
    }
    let mut ir_entities: Vec<Entity> = entities
        .iter()
        .map(|entity| {
            let key = entity.key();
            Entity {
                introduced_at: entity
                    .introduced_at
                    .or_else(|| introduced.get(key.as_str()).copied()),
                id: IrEntityId::new(key),
                sort: entity.kind.to_sort(),
                name: entity.name.clone(),
                aliases: entity
                    .aliases
                    .iter()
                    .map(|form| Alias {
                        form: form.clone(),
                        from_tick: None,
                        used_by: None,
                    })
                    .collect(),
            }
        })
        .collect();
    let mut known_keys: HashSet<String> = ir_entities
        .iter()
        .map(|entity| entity.id.0.clone())
        .collect();
    for entry in world {
        let sort = match entry.kind {
            WorldKind::Place => Sort::Place,
            WorldKind::Item => Sort::Item,
            WorldKind::Creature => Sort::Other,
            WorldKind::Culture => Sort::Group,
            _ => continue,
        };
        let key = entity_key_from_title(&entry.title);
        if key.is_empty() || !known_keys.insert(key.clone()) {
            continue;
        }
        ir_entities.push(Entity {
            id: IrEntityId::new(key),
            sort,
            name: entry.title.clone(),
            aliases: Vec::new(),
            introduced_at: None,
        });
    }

    let predicates = asten_engine::core_predicates();
    let mut lattices = std::collections::BTreeMap::new();
    for lattice in [
        asten_ir::lattice::color_lattice(),
        asten_ir::lattice::lifecycle_lattice(),
        asten_ir::lattice::material_lattice(),
    ] {
        lattices.insert(lattice.name.clone(), lattice);
    }

    let parse_bound = |label: &Option<String>, lower: bool| -> Option<i64> {
        let label = label.as_deref()?;
        match clock.parse(label)? {
            Parsed::Absolute(bounds) => {
                if lower {
                    bounds.lo
                } else {
                    bounds.hi
                }
            }
            Parsed::Relative { .. } => None,
        }
    };

    let mut ir_facts = Vec::new();
    let mut ir_events: Vec<Event> = Vec::new();
    for fact in facts {
        if fact.modality == FactModality::Contested
            || !matches!(fact.status, FactStatus::Confirmed | FactStatus::Proposed)
        {
            continue;
        }
        if fact.predicate == "kind" {
            continue;
        }
        let predicate = PredicateId::new(fact.predicate.clone());
        let sig = predicates.iter().find(|sig| sig.id == predicate);
        let value = asten_perceive::resolve_value(sig, &lattices, &fact.value);
        let valid = TimeBounds {
            lo: parse_bound(&fact.valid_from, true),
            hi: parse_bound(&fact.valid_to, false),
        };
        let status = if fact.modality == FactModality::AuthorOverride {
            Status::Settled
        } else {
            fact.status.to_ir()
        };
        // Legacy death facts become lifecycle events so the typestate check sees them.
        if matches!(fact.predicate.as_str(), "lifecycle" | "status" | "alive") {
            let dead = matches!(&value, asten_ir::Value::Concept { id, .. } if id.0 == "dead")
                || matches!(&value, asten_ir::Value::Bool { value: false });
            if dead {
                ir_events.push(Event {
                    id: EventId::new(format!("death:{}", fact.id)),
                    kind: EventKind::Dies,
                    label: format!("{} dies", fact.entity_key.replace('_', " ")),
                    subject: Some(IrEntityId::new(fact.entity_key.clone())),
                    participants: Vec::new(),
                    at: match valid.lo {
                        Some(lo) => TimeRef::Tick { tick: lo },
                        None => fact
                            .anchor
                            .as_ref()
                            .map(|anchor| TimeRef::Scene {
                                document: anchor.document.clone(),
                            })
                            .unwrap_or(TimeRef::Unknown),
                    },
                    reading_pos: None,
                    context: fact.context.clone(),
                    status,
                    confidence: fact.confidence,
                    provenance: fact.provenance.clone(),
                    anchor: fact.anchor.clone(),
                    document: fact.anchor.as_ref().map(|anchor| anchor.document.clone()),
                    initiates: Vec::new(),
                    terminates: Vec::new(),
                });
            }
        }
        ir_facts.push(IrFact {
            id: IrFactId::new(fact.id.to_string()),
            fluent: Fluent {
                predicate,
                subject: IrEntityId::new(fact.entity_key.clone()),
                value,
            },
            context: fact.context.clone(),
            valid,
            status,
            provenance: fact.provenance.clone(),
            confidence: fact.confidence,
            anchor: fact.anchor.clone(),
            settled_at: fact.settled_by.as_ref().map(|by| asten_ir::SettledAt {
                actor: by.clone(),
                at: fact
                    .settled_at
                    .map(|at| at.to_rfc3339())
                    .unwrap_or_default(),
            }),
        });
    }

    // Lookups the event loop needs: cast rows by id, scenes by document.
    let entity_keys: HashMap<asten_domain::EntityId, String> = entities
        .iter()
        .map(|entity| (entity.id, entity.key()))
        .collect();
    let scene_positions: HashMap<&str, i64> = scenes
        .iter()
        .map(|scene| (scene.document.0.as_str(), scene.reading_pos))
        .collect();
    for event in events {
        let subject = event
            .entity_id
            .and_then(|id| entity_keys.get(&id))
            .map(|key| IrEntityId::new(key.clone()));
        let point = TimePoint::Event {
            id: EventId::new(event.id.to_string()),
        };
        let doc = event.document_id.map(|id| DocId::new(id.to_string()));
        let at = if let Some(tick) = event.tick.map(clamp_tick) {
            TimeRef::Tick { tick }
        } else if event.tick_lo.is_some() || event.tick_hi.is_some() {
            TimeRef::Bounds {
                lo: event.tick_lo.map(clamp_tick),
                hi: event.tick_hi.map(clamp_tick),
            }
        } else {
            match event
                .story_time
                .as_deref()
                .and_then(|label| clock.parse(label))
            {
                Some(Parsed::Absolute(bounds)) => TimeRef::Bounds {
                    lo: bounds.lo,
                    hi: bounds.hi,
                },
                Some(Parsed::Relative {
                    relation,
                    lo,
                    hi,
                    anchor,
                }) => {
                    let previous = doc.as_ref().map(|document| TimePoint::Scene {
                        document: document.clone(),
                    });
                    if let Some(constraint) = time.relative_constraint(
                        point.clone(),
                        relation,
                        lo,
                        hi,
                        anchor.as_deref(),
                        previous.as_ref(),
                        &format!(
                            "{}: \u{201c}{}\u{201d}",
                            event.title,
                            event.story_time.clone().unwrap_or_default()
                        ),
                        event.status.to_ir(),
                    ) {
                        constraints.push(constraint);
                    }
                    TimeRef::Unknown
                }
                None => match &doc {
                    Some(document) => TimeRef::Scene {
                        document: document.clone(),
                    },
                    None => TimeRef::Unknown,
                },
            }
        };
        let reading_pos = event.reading_order.or_else(|| {
            doc.as_ref()
                .and_then(|document| scene_positions.get(document.0.as_str()))
                .copied()
        });
        ir_events.push(Event {
            id: EventId::new(event.id.to_string()),
            kind: event.kind.clone(),
            label: event.title.clone(),
            subject,
            participants: event
                .participants
                .iter()
                .map(|key| IrEntityId::new(key.clone()))
                .collect(),
            at,
            reading_pos,
            context: event.context.clone(),
            status: event.status.to_ir(),
            confidence: event.confidence,
            provenance: event.provenance.clone(),
            anchor: event.anchor.clone(),
            document: doc,
            initiates: Vec::new(),
            terminates: Vec::new(),
        });
    }

    let ir_suppressions: Vec<DiagSuppression> = suppressions
        .iter()
        .map(|suppression| DiagSuppression {
            id: suppression.id.to_string(),
            code: suppression.rule_code.clone(),
            scope: match (
                &suppression.scope,
                &suppression.diagnostic_id,
                &suppression.entity_key,
                suppression.document_id,
            ) {
                (SuppressionScope::Occurrence, Some(diagnostic_id), _, _) => {
                    DiagScope::Occurrence {
                        diagnostic_id: diagnostic_id.clone(),
                    }
                }
                (SuppressionScope::Entity, _, Some(entity), _) => DiagScope::Entity {
                    entity: IrEntityId::new(entity.clone()),
                },
                (SuppressionScope::Document, _, _, Some(document))
                | (SuppressionScope::Occurrence, None, _, Some(document)) => DiagScope::Document {
                    document: DocId::new(document.to_string()),
                },
                (_, _, Some(entity), _) => DiagScope::Entity {
                    entity: IrEntityId::new(entity.clone()),
                },
                (_, _, _, Some(document)) => DiagScope::Document {
                    document: DocId::new(document.to_string()),
                },
                _ => DiagScope::Project,
            },
            evidence_fingerprint: suppression.evidence_fingerprint.clone(),
            reason: suppression.reason.clone(),
        })
        .collect();

    let baseline = baselines
        .iter()
        .find(|baseline| baseline.active)
        .map(|baseline| baseline.diagnostic_ids.clone())
        .unwrap_or_default();
    let enabled_packs: Vec<String> = packs
        .iter()
        .filter(|pack| pack.enabled)
        .map(|pack| pack.pack_key.clone())
        .collect();
    let craft_hints: Vec<CraftHint> = packs
        .iter()
        .filter(|pack| pack.enabled)
        .flat_map(|pack| {
            pack.manifest
                .get("hints")
                .and_then(|hints| hints.as_array())
                .map(|hints| {
                    hints
                        .iter()
                        .filter_map(|hint| {
                            Some(CraftHint {
                                code: hint.get("code")?.as_str()?.to_owned(),
                                word: hint.get("word")?.as_str()?.to_owned(),
                                message: hint
                                    .get("message")
                                    .and_then(|message| message.as_str())
                                    .unwrap_or("Craft note.")
                                    .to_owned(),
                                pack: pack.pack_key.clone(),
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();

    let ir_settings: Settings = settings.to_ir();
    let ir_invariants: Vec<Invariant> = invariants
        .iter()
        .map(|stored| stored.invariant.clone())
        .collect();
    let ir_routes: Vec<Route> = routes.iter().map(|stored| stored.route.clone()).collect();

    let mut inputs = ProjectInputs {
        revision: String::new(),
        scenes,
        blocks,
        entities: ir_entities,
        predicates,
        lattices,
        facts: ir_facts,
        events: ir_events,
        constraints,
        invariants: ir_invariants,
        routes: ir_routes,
        settings: ir_settings,
        suppressions: ir_suppressions,
        baseline,
        enabled_packs,
        craft_hints,
    };
    inputs.revision = revision_of(&inputs);
    inputs
}

/// A stable revision id over everything but block text (fingerprints stand in).
fn revision_of(inputs: &ProjectInputs) -> String {
    let fingerprints: Vec<&str> = inputs
        .blocks
        .iter()
        .map(|block| block.fingerprint.as_str())
        .collect();
    let rest = serde_json::to_string(&(
        &inputs.scenes,
        &fingerprints,
        &inputs.entities,
        &inputs.facts,
        &inputs.events,
        &inputs.constraints,
        &inputs.invariants,
        &inputs.routes,
        &inputs.settings,
        &inputs.suppressions,
        &inputs.baseline,
        &inputs.enabled_packs,
        &inputs.craft_hints,
    ))
    .unwrap_or_default();
    asten_ir::sha256_hex(rest.as_bytes())[..16].to_owned()
}

#[allow(dead_code)]
fn _anchor_bridge(anchor: &Anchor) -> &Anchor {
    anchor
}

#[allow(dead_code)]
fn _context_bridge(context: &Context) -> &Context {
    context
}

/// Ticks supplied through the API are clamped to ±2^53 so that every offset
/// the analyses add to them stays far from `i64` overflow.
fn clamp_tick(tick: i64) -> i64 {
    const MAX_TICK: i64 = 1 << 53;
    tick.clamp(-MAX_TICK, MAX_TICK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_domain::DocumentKind;
    use std::time::Instant;

    fn paragraph(id: &str, text: &str) -> serde_json::Value {
        serde_json::json!({
            "type": "paragraph",
            "attrs": { "id": id },
            "content": [{ "type": "text", "text": text }]
        })
    }

    fn document(
        project: ProjectId,
        parent: Option<DocumentId>,
        kind: DocumentKind,
        title: &str,
        position: i64,
        blocks: Vec<serde_json::Value>,
    ) -> Document {
        let now = Utc::now();
        Document {
            id: DocumentId::new(),
            project_id: project,
            parent_id: parent,
            kind,
            title: title.to_owned(),
            content_json: serde_json::json!({ "type": "doc", "content": blocks }),
            position,
            revision: 0,
            exclude_from_compile: false,
            scene: ScenePlacement::default(),
            head_seq: 0,
            chain_hash: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    fn entity(project: ProjectId, name: &str) -> CanonEntity {
        let now = Utc::now();
        CanonEntity {
            id: asten_domain::EntityId::new(),
            project_id: project,
            kind: EntityKind::Character,
            name: name.to_owned(),
            aliases: Vec::new(),
            notes: String::new(),
            introduced_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 250k words as the desk stores them: one manuscript, 50 chapters of 50
    /// paragraphs, each paragraph ~100 words.
    fn large_project() -> (Vec<Document>, Vec<CanonEntity>, ProjectSettings) {
        let project = ProjectId::new();
        let sentence = "Mira walked along the harbor wall while Oren counted the boats and the gulls argued over the nets. ";
        let manuscript = document(
            project,
            None,
            DocumentKind::Manuscript,
            "The Harbor",
            0,
            Vec::new(),
        );
        let mut documents = vec![manuscript.clone()];
        for chapter in 0..50 {
            let blocks = (0..50)
                .map(|index| paragraph(&format!("c{chapter}-{index}"), &sentence.repeat(5)))
                .collect();
            documents.push(document(
                project,
                Some(manuscript.id),
                DocumentKind::Chapter,
                &format!("Chapter {chapter}"),
                chapter,
                blocks,
            ));
        }
        let entities = vec![entity(project, "Mira"), entity(project, "Captain Oren")];
        (documents, entities, ProjectSettings::for_project(project))
    }

    fn assemble(
        documents: &[Document],
        entities: &[CanonEntity],
        settings: &ProjectSettings,
    ) -> ProjectInputs {
        assemble_inputs(
            documents,
            entities,
            &[],
            &[],
            &[],
            settings,
            &[],
            &[],
            &[],
            &[],
            &[],
        )
    }

    /// `cargo test -p asten-application -- --ignored --nocapture bench` prints
    /// how long assembling inputs from storage rows and analysing them takes.
    #[test]
    #[ignore = "benchmark; run by hand with --nocapture"]
    fn bench_assemble_and_analyze_250k_words() {
        let (documents, entities, settings) = large_project();
        let started = Instant::now();
        let inputs = assemble(&documents, &entities, &settings);
        let assemble_ms = started.elapsed().as_millis();
        println!(
            "assemble: {assemble_ms} ms ({} blocks, {} scenes)",
            inputs.blocks.len(),
            inputs.scenes.len()
        );
        let mut engine = Engine::new();
        let cold = engine.analyze(&inputs);
        println!(
            "cold: perceive {} ms, analyze {} ms, total {} ms",
            cold.snapshot.timings_ms.perceive,
            cold.snapshot.timings_ms.analyze,
            cold.snapshot.timings_ms.total
        );
        // The round trip the desk pays per keystroke-commit: rebuild inputs
        // from rows, then analyse with one paragraph changed.
        let mut documents = documents;
        for pass in 0..3 {
            let node = &mut documents[25].content_json["content"][10]["content"][0];
            let text = node["text"].as_str().unwrap_or_default().to_owned();
            node["text"] = serde_json::Value::String(format!("{text} Mira smiled."));
            let started = Instant::now();
            let inputs = assemble(&documents, &entities, &settings);
            let assemble_ms = started.elapsed().as_millis();
            let warm = engine.analyze(&inputs);
            println!(
                "edit {pass}: assemble {assemble_ms} ms, perceive {} ms, analyze {} ms, total {} ms ({} cached)",
                warm.snapshot.timings_ms.perceive,
                warm.snapshot.timings_ms.analyze,
                warm.snapshot.timings_ms.total,
                warm.snapshot.cached_blocks
            );
        }
    }

    #[test]
    fn reading_order_is_depth_first_and_keeps_orphans() {
        let project = ProjectId::new();
        let book = document(project, None, DocumentKind::Manuscript, "Book", 0, vec![]);
        let part_two = document(
            project,
            Some(book.id),
            DocumentKind::Chapter,
            "Part two",
            2,
            vec![],
        );
        let part_one = document(
            project,
            Some(book.id),
            DocumentKind::Chapter,
            "Part one",
            1,
            vec![],
        );
        let scene = document(
            project,
            Some(part_one.id),
            DocumentKind::Scene,
            "Scene",
            0,
            vec![],
        );
        let orphan = document(
            project,
            Some(DocumentId::new()),
            DocumentKind::Note,
            "Orphan",
            0,
            vec![],
        );
        let notes = document(project, None, DocumentKind::Note, "Notes", 5, vec![]);
        let documents = vec![
            orphan.clone(),
            part_two.clone(),
            scene.clone(),
            notes.clone(),
            book.clone(),
            part_one.clone(),
        ];
        let titles: Vec<&str> = reading_order(&documents)
            .iter()
            .map(|document| document.title.as_str())
            .collect();
        assert_eq!(
            titles,
            ["Book", "Part one", "Scene", "Part two", "Notes", "Orphan"]
        );
    }

    #[test]
    fn assembled_inputs_number_prose_in_reading_order() {
        let (documents, entities, settings) = large_project();
        let inputs = assemble(&documents, &entities, &settings);
        assert_eq!(inputs.scenes.len(), 51);
        assert_eq!(inputs.blocks.len(), 2500);
        let positions: Vec<i64> = inputs
            .scenes
            .iter()
            .map(|scene| scene.reading_pos)
            .collect();
        assert_eq!(positions, (1..=51).collect::<Vec<i64>>());
        assert!(
            inputs
                .blocks
                .windows(2)
                .all(|pair| pair[0].reading_pos <= pair[1].reading_pos)
        );
    }
}
