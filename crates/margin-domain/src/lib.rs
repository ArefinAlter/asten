use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! entity_id {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

entity_id!(ProjectId);
entity_id!(DocumentId);
entity_id!(DiagnosticId);
entity_id!(FactId);
entity_id!(WorldEntryId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    Novel,
    Screenplay,
    NonFiction,
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub kind: ProjectKind,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentKind {
    Manuscript,
    Chapter,
    Scene,
    Wiki,
    Note,
    Source,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Document {
    pub id: DocumentId,
    pub project_id: ProjectId,
    pub parent_id: Option<DocumentId>,
    pub kind: DocumentKind,
    pub title: String,
    pub content_json: serde_json::Value,
    pub position: i64,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceRange {
    pub document_id: DocumentId,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Diagnostic {
    pub id: DiagnosticId,
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub source: SourceRange,
    pub details: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FactModality {
    #[default]
    Established,
    Inferred,
    Contested,
    AuthorOverride,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FactStatus {
    Proposed,
    #[default]
    Confirmed,
    Deprecated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceAnchor {
    pub document_id: DocumentId,
    pub start: u32,
    pub end: u32,
}

impl From<SourceAnchor> for SourceRange {
    fn from(anchor: SourceAnchor) -> Self {
        Self {
            document_id: anchor.document_id,
            start: anchor.start,
            end: anchor.end,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Fact {
    pub id: FactId,
    pub project_id: ProjectId,
    pub entity_key: String,
    pub predicate: String,
    pub value: String,
    pub modality: FactModality,
    pub status: FactStatus,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub source: Option<SourceAnchor>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldKind {
    Place,
    Culture,
    System,
    Creature,
    Item,
    Question,
    Fragment,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldStatus {
    #[default]
    Spark,
    Sketched,
    Settled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdeaPrompt {
    Why,
    How,
    WhatResult,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WorldEntry {
    pub id: WorldEntryId,
    pub project_id: ProjectId,
    pub kind: WorldKind,
    pub title: String,
    pub summary: String,
    pub prompt: Option<IdeaPrompt>,
    pub status: WorldStatus,
    pub board_x: f64,
    pub board_y: f64,
    pub map_x: Option<f64>,
    pub map_y: Option<f64>,
    pub hue: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WorldPulse {
    pub sparks: u32,
    pub sketched: u32,
    pub settled: u32,
    pub questions: u32,
    pub places: u32,
    pub systems: u32,
    pub scenes: u32,
    pub underwater: u32,
    pub iceberg: f32,
    pub discoveries: Vec<String>,
}

#[must_use]
pub fn default_world_hue(kind: &WorldKind) -> &'static str {
    match kind {
        WorldKind::Place => "ochre",
        WorldKind::Culture => "dusk",
        WorldKind::System => "moss",
        WorldKind::Creature => "clay",
        WorldKind::Item => "fog",
        WorldKind::Question => "ink",
        WorldKind::Fragment => "parchment",
    }
}

#[must_use]
pub fn entity_key_from_title(title: &str) -> String {
    normalize_key(title)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
}

#[must_use]
pub fn compute_world_pulse(entries: &[WorldEntry], documents: &[Document]) -> WorldPulse {
    let mut pulse = WorldPulse {
        sparks: 0,
        sketched: 0,
        settled: 0,
        questions: 0,
        places: 0,
        systems: 0,
        scenes: 0,
        underwater: 0,
        iceberg: 0.0,
        discoveries: Vec::new(),
    };
    let mut has_map_pin = false;
    let mut asked_why = false;
    let mut settled_rule = false;
    for entry in entries {
        match entry.status {
            WorldStatus::Spark => pulse.sparks += 1,
            WorldStatus::Sketched => pulse.sketched += 1,
            WorldStatus::Settled => pulse.settled += 1,
        }
        match entry.kind {
            WorldKind::Question => pulse.questions += 1,
            WorldKind::Place => pulse.places += 1,
            WorldKind::System => pulse.systems += 1,
            _ => {}
        }
        if entry.map_x.is_some() && entry.map_y.is_some() {
            has_map_pin = true;
        }
        if entry.kind == WorldKind::Question && entry.prompt == Some(IdeaPrompt::Why) {
            asked_why = true;
        }
        if entry.kind == WorldKind::System && entry.status == WorldStatus::Settled {
            settled_rule = true;
        }
        if entry.kind != WorldKind::Question
            && matches!(entry.status, WorldStatus::Sketched | WorldStatus::Settled)
        {
            pulse.underwater += 1;
        }
    }
    pulse.scenes = documents
        .iter()
        .filter(|document| {
            matches!(
                document.kind,
                DocumentKind::Scene | DocumentKind::Chapter | DocumentKind::Manuscript
            )
        })
        .count() as u32;
    let depth = pulse.underwater + pulse.scenes;
    pulse.iceberg = if depth == 0 {
        0.0
    } else {
        pulse.underwater as f32 / depth as f32
    };
    if !entries.is_empty() {
        pulse.discoveries.push("first_spark".into());
    }
    if asked_why {
        pulse.discoveries.push("asked_why".into());
    }
    if pulse.places > 0 {
        pulse.discoveries.push("first_place".into());
    }
    if has_map_pin {
        pulse.discoveries.push("ugly_triangles".into());
    }
    if settled_rule {
        pulse.discoveries.push("first_rule".into());
    }
    if pulse.scenes > 0 {
        pulse.discoveries.push("scene_anyway".into());
    }
    pulse
}

#[must_use]
pub fn normalize_key(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[must_use]
pub fn normalize_value(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "blonde" => "blond".to_owned(),
        "grey" => "gray".to_owned(),
        other => other.to_owned(),
    }
}

#[must_use]
pub fn document_plain_text(content: &serde_json::Value) -> String {
    let mut text = String::new();
    collect_plain_text(content, &mut text);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_plain_text(value: &serde_json::Value, output: &mut String) {
    match value {
        serde_json::Value::String(text) => append_text(output, text),
        serde_json::Value::Array(items) => {
            for item in items {
                collect_plain_text(item, output);
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(text) = map.get("text").and_then(serde_json::Value::as_str) {
                append_text(output, text);
            }
            if let Some(content) = map.get("content") {
                collect_plain_text(content, output);
            }
        }
        _ => {}
    }
}

fn append_text(output: &mut String, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    if !output.is_empty() {
        output.push(' ');
    }
    output.push_str(trimmed);
}

#[cfg(test)]
mod tests {
    use super::document_plain_text;
    use serde_json::json;

    #[test]
    fn reads_plain_text_and_prosemirror_nodes() {
        assert_eq!(
            document_plain_text(&json!({ "text": "Mira waited." })),
            "Mira waited."
        );
        assert_eq!(
            document_plain_text(&json!({
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": "Black hair." }]
                }]
            })),
            "Black hair."
        );
    }

    #[test]
    fn world_pulse_treats_notes_as_iceberg_and_scenes_as_the_tip() {
        use super::{
            Document, DocumentId, DocumentKind, IdeaPrompt, ProjectId, WorldEntry, WorldEntryId,
            WorldKind, WorldStatus, compute_world_pulse, entity_key_from_title,
        };
        use chrono::Utc;

        let now = Utc::now();
        let project_id = ProjectId::new();
        let entries = vec![
            WorldEntry {
                id: WorldEntryId::new(),
                project_id,
                kind: WorldKind::Place,
                title: "Dusk Harbor".into(),
                summary: String::new(),
                prompt: None,
                status: WorldStatus::Sketched,
                board_x: 10.0,
                board_y: 12.0,
                map_x: Some(40.0),
                map_y: Some(55.0),
                hue: "ochre".into(),
                created_at: now,
                updated_at: now,
            },
            WorldEntry {
                id: WorldEntryId::new(),
                project_id,
                kind: WorldKind::Question,
                title: "Why do the tides speak?".into(),
                summary: String::new(),
                prompt: Some(IdeaPrompt::Why),
                status: WorldStatus::Spark,
                board_x: 20.0,
                board_y: 18.0,
                map_x: None,
                map_y: None,
                hue: "ink".into(),
                created_at: now,
                updated_at: now,
            },
        ];
        let documents = vec![Document {
            id: DocumentId::new(),
            project_id,
            parent_id: None,
            kind: DocumentKind::Scene,
            title: "Arrival".into(),
            content_json: serde_json::json!({"text": "She heard the harbor."}),
            position: 0,
            revision: 1,
            created_at: now,
            updated_at: now,
        }];
        let pulse = compute_world_pulse(&entries, &documents);
        assert_eq!(pulse.places, 1);
        assert_eq!(pulse.questions, 1);
        assert_eq!(pulse.underwater, 1);
        assert_eq!(pulse.scenes, 1);
        assert!(pulse.discoveries.contains(&"asked_why".into()));
        assert!(pulse.discoveries.contains(&"ugly_triangles".into()));
        assert!(pulse.discoveries.contains(&"scene_anyway".into()));
        assert_eq!(entity_key_from_title("Dusk Harbor"), "dusk_harbor");
    }
}
