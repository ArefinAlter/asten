//! The project file: one project as a single JSON document that another copy of
//! Asten can open.
//!
//! It carries the manuscript (every document with its content, kind, order and
//! scene placement), the cast, facts, timeline, world board, routes, invariants,
//! settings and comments. It does not carry the version history (snapshots,
//! drafts, the hash-chained event log), baselines or suppressions: those belong
//! to the library, and a backup of the library keeps them. Ids are minted again
//! on import, and every reference between records is rewritten to match.

use std::collections::HashMap;

use asten_domain::{CanonEntity, Document, DocumentId, Project, ProjectId, ProjectKind};
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    Application, ApplicationError, CreateComment, CreateDocument, CreateFact, CreateProject,
    CreateWorldEntry, Result, UpdateSettings,
    engine_app::{CreateCanonEntity, CreateTimelineEvent, UpdateComment},
    story::{CreateInvariant, CreateRoute},
};

/// The `format` field of a project file.
pub const FORMAT: &str = "asten-project";
/// The version this build writes and reads.
pub const VERSION: u64 = 1;

const MAX_DOCUMENTS: usize = 20_000;
const MAX_RECORDS: usize = 200_000;

fn invalid(message: impl Into<String>) -> ApplicationError {
    ApplicationError::Validation(message.into())
}

fn to_value<T: serde::Serialize>(value: &T) -> Result<Value> {
    serde_json::to_value(value).map_err(|error| ApplicationError::Storage(error.to_string()))
}

/// Replace every string equal to an old id with the new one.
fn remap(value: &mut Value, ids: &HashMap<String, String>) {
    match value {
        Value::String(text) => {
            if let Some(new) = ids.get(text.as_str()) {
                *text = new.clone();
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| remap(item, ids)),
        Value::Object(map) => map.values_mut().for_each(|item| remap(item, ids)),
        _ => {}
    }
}

fn make<T: DeserializeOwned>(
    mut value: Value,
    project: ProjectId,
    ids: &HashMap<String, String>,
) -> Result<T> {
    remap(&mut value, ids);
    if let Some(map) = value.as_object_mut() {
        map.insert("project_id".into(), json!(project));
    }
    serde_json::from_value(value).map_err(|error| {
        invalid(format!(
            "the project file has a record this version cannot read: {error}"
        ))
    })
}

static EMPTY: Vec<Value> = Vec::new();

fn array<'a>(bundle: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    match bundle.get(key) {
        None => Ok(&EMPTY),
        Some(Value::Array(items)) if items.len() <= MAX_RECORDS => Ok(items),
        Some(Value::Array(_)) => Err(invalid(format!("the project file lists too many {key}"))),
        Some(_) => Err(invalid(format!("the project file's {key} is not a list"))),
    }
}

/// Read every record once before anything is created, so a file with one
/// bad record is refused whole rather than half imported.
fn validate(bundle: &Value) -> Result<()> {
    let none = HashMap::new();
    let pid = ProjectId::new();
    for value in array(bundle, "entities")? {
        serde_json::from_value::<CanonEntity>(value.clone())
            .map_err(|error| invalid(format!("an entity cannot be read: {error}")))?;
    }
    for value in array(bundle, "documents")? {
        serde_json::from_value::<Document>(value.clone())
            .map_err(|error| invalid(format!("a document cannot be read: {error}")))?;
    }
    for value in array(bundle, "facts")? {
        make::<CreateFact>(value.clone(), pid, &none)?;
    }
    for value in array(bundle, "events")? {
        make::<CreateTimelineEvent>(value.clone(), pid, &none)?;
    }
    for value in array(bundle, "world_entries")? {
        make::<CreateWorldEntry>(value.clone(), pid, &none)?;
    }
    for value in array(bundle, "routes")? {
        if let Some(route) = value.get("route") {
            serde_json::from_value::<asten_ir::Route>(route.clone())
                .map_err(|error| invalid(format!("a route cannot be read: {error}")))?;
        }
    }
    for value in array(bundle, "invariants")? {
        if let Some(invariant) = value.get("invariant") {
            serde_json::from_value::<CreateInvariant>(invariant.clone())
                .map_err(|error| invalid(format!("a rule cannot be read: {error}")))?;
        }
    }
    Ok(())
}

impl Application {
    /// One project as a project file.
    pub async fn export_project(&self, project_id: ProjectId) -> Result<Value> {
        let project = self.get_project(project_id).await?;
        let documents = self.list_documents(project_id).await?;
        let mut comments = Vec::new();
        for document in &documents {
            for comment in self.list_comments(document.id).await? {
                comments.push(to_value(&comment)?);
            }
        }
        Ok(json!({
            "format": FORMAT,
            "version": VERSION,
            "exported_at": Utc::now(),
            "app_version": env!("CARGO_PKG_VERSION"),
            "project": {
                "name": project.name,
                "kind": project.kind,
                "description": project.description,
            },
            "settings": to_value(&self.get_settings(project_id).await?)?,
            "documents": to_value(&documents)?,
            "entities": to_value(&self.list_entities(project_id).await?)?,
            "facts": to_value(&self.list_facts(project_id).await?)?,
            "events": to_value(&self.list_events(project_id).await?)?,
            "world_entries": to_value(&self.list_world_entries(project_id).await?)?,
            "routes": to_value(&self.list_routes(project_id).await?)?,
            "invariants": to_value(&self.list_invariants(project_id).await?)?,
            "comments": comments,
        }))
    }

    /// Open a project file as a new project.
    pub async fn import_project_bundle(&self, bundle: Value) -> Result<Project> {
        if bundle.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err(invalid("this is not an Asten project file"));
        }
        match bundle.get("version").and_then(Value::as_u64) {
            Some(VERSION) => {}
            Some(newer) if newer > VERSION => {
                return Err(invalid(
                    "this project file was written by a newer Asten; update Asten to open it",
                ));
            }
            _ => return Err(invalid("the project file has no readable version")),
        }
        let documents = array(&bundle, "documents")?;
        if documents.len() > MAX_DOCUMENTS {
            return Err(invalid("the project file holds too many documents"));
        }
        let name = bundle
            .pointer("/project/name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| invalid("the project file has no project name"))?;
        let kind: ProjectKind = bundle
            .pointer("/project/kind")
            .cloned()
            .and_then(|kind| serde_json::from_value(kind).ok())
            .unwrap_or(ProjectKind::Novel);
        validate(&bundle)?;
        let project = self
            .create_project(CreateProject {
                name: name.to_owned(),
                kind,
                description: bundle
                    .pointer("/project/description")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            })
            .await?;
        let pid = project.id;
        // If anything below fails the half-made project is removed again.
        match self.fill_from_bundle(pid, &bundle).await {
            Ok(()) => Ok(project),
            Err(error) => {
                let _ = self.delete_project(pid).await;
                Err(error)
            }
        }
    }

    async fn fill_from_bundle(&self, pid: ProjectId, bundle: &Value) -> Result<()> {
        let mut ids: HashMap<String, String> = HashMap::new();

        if let Some(settings) = bundle.get("settings").cloned() {
            if let Ok(update) = serde_json::from_value::<UpdateSettings>(settings.clone()) {
                // `min_parent_age_days` is optional in the file; the update type wants it doubly so.
                let mut update = update;
                update.min_parent_age_days = settings
                    .get("min_parent_age_days")
                    .and_then(Value::as_i64)
                    .map(Some);
                self.update_settings(pid, update).await?;
            }
        }

        // The cast first: content and events refer to it.
        for value in array(bundle, "entities")? {
            let old: CanonEntity = serde_json::from_value(value.clone())
                .map_err(|error| invalid(format!("an entity cannot be read: {error}")))?;
            let created = self
                .create_entity(CreateCanonEntity {
                    project_id: pid,
                    kind: old.kind,
                    name: old.name,
                    aliases: old.aliases,
                    notes: old.notes,
                    introduced_at: old.introduced_at,
                })
                .await?;
            ids.insert(old.id.to_string(), created.id.to_string());
        }

        // Documents, parents before children.
        let mut pending: Vec<&Value> = array(bundle, "documents")?.iter().collect();
        let known: std::collections::HashSet<String> = pending
            .iter()
            .filter_map(|d| d.get("id").and_then(Value::as_str).map(str::to_owned))
            .collect();
        let mut rounds = 0;
        while !pending.is_empty() {
            rounds += 1;
            if rounds > 64 {
                return Err(invalid("the project file's documents are nested in a loop"));
            }
            let mut later = Vec::new();
            for value in pending {
                let parent = value.get("parent_id").and_then(Value::as_str);
                if let Some(parent) = parent {
                    if known.contains(parent) && !ids.contains_key(parent) {
                        later.push(value);
                        continue;
                    }
                }
                let old: Document = {
                    let mut copy = value.clone();
                    remap(&mut copy, &ids);
                    // Ids inside content are block ids; they stay, and so do the
                    // entity references remapped above.
                    serde_json::from_value(copy)
                        .map_err(|error| invalid(format!("a document cannot be read: {error}")))?
                };
                let parent_id = parent
                    .and_then(|p| ids.get(p))
                    .and_then(|p| Uuid::parse_str(p).ok())
                    .map(DocumentId);
                let created = self
                    .create_document(CreateDocument {
                        project_id: pid,
                        parent_id,
                        kind: old.kind,
                        title: old.title,
                        content_json: old.content_json,
                        position: old.position,
                        scene: Some(old.scene),
                    })
                    .await?;
                if old.exclude_from_compile {
                    self.update_document(
                        created.id,
                        crate::UpdateDocument {
                            exclude_from_compile: Some(true),
                            ..crate::UpdateDocument::default()
                        },
                    )
                    .await?;
                }
                if let Some(old_id) = value.get("id").and_then(Value::as_str) {
                    ids.insert(old_id.to_owned(), created.id.to_string());
                }
            }
            pending = later;
        }

        for value in array(bundle, "facts")? {
            let input: CreateFact = make(value.clone(), pid, &ids)?;
            self.create_fact(input).await?;
        }
        for value in array(bundle, "events")? {
            let input: CreateTimelineEvent = make(value.clone(), pid, &ids)?;
            self.create_event(input).await?;
        }
        for value in array(bundle, "world_entries")? {
            let input: CreateWorldEntry = make(value.clone(), pid, &ids)?;
            self.create_world_entry(input).await?;
        }
        for value in array(bundle, "routes")? {
            if let Some(route) = value.get("route").cloned() {
                let route = serde_json::from_value(route)
                    .map_err(|error| invalid(format!("a route cannot be read: {error}")))?;
                self.create_route(pid, CreateRoute { route }).await?;
            }
        }
        for value in array(bundle, "invariants")? {
            if let Some(invariant) = value.get("invariant") {
                let input: CreateInvariant = serde_json::from_value(invariant.clone())
                    .map_err(|error| invalid(format!("a rule cannot be read: {error}")))?;
                self.create_invariant(pid, input).await?;
            }
        }
        for value in array(bundle, "comments")? {
            let mut copy = value.clone();
            remap(&mut copy, &ids);
            let Some(document_id) = copy
                .get("document_id")
                .and_then(Value::as_str)
                .and_then(|id| Uuid::parse_str(id).ok())
                .map(DocumentId)
            else {
                continue;
            };
            let Ok(created) = self
                .create_comment(CreateComment {
                    document_id,
                    block_id: copy
                        .get("block_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    from_offset: copy.get("from_offset").and_then(Value::as_i64).unwrap_or(0),
                    to_offset: copy.get("to_offset").and_then(Value::as_i64).unwrap_or(0),
                    body: copy
                        .get("body")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned(),
                })
                .await
            else {
                continue;
            };
            if copy.get("resolved").and_then(Value::as_bool) == Some(true) {
                self.update_comment(
                    created.id,
                    UpdateComment {
                        body: None,
                        resolved: Some(true),
                    },
                )
                .await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remapping_replaces_whole_strings_only() {
        let ids: HashMap<String, String> = [("old-1".to_owned(), "new-1".to_owned())].into();
        let mut value = json!({"a": "old-1", "b": ["old-1", "old-10"], "c": {"d": "xold-1"}});
        remap(&mut value, &ids);
        assert_eq!(value["a"], "new-1");
        assert_eq!(value["b"][0], "new-1");
        assert_eq!(value["b"][1], "old-10");
        assert_eq!(value["c"]["d"], "xold-1");
    }
}
