use margin_domain::{
    Diagnostic, DiagnosticId, Document, DocumentKind, Fact, FactModality, FactStatus, Project,
    Severity, SourceAnchor, SourceRange, document_plain_text, normalize_key, normalize_value,
};

pub const ATTRIBUTE_CONTRADICTION: &str = "CANON-E013";

const COLOR_WORDS: &[&str] = &[
    "amber", "auburn", "black", "blond", "blonde", "blue", "brown", "ginger", "gray", "green",
    "grey", "hazel", "pink", "red", "silver", "white",
];

pub struct AnalysisContext<'a> {
    pub project: &'a Project,
    pub documents: &'a [Document],
    pub facts: &'a [Fact],
}

pub trait Rule: Send + Sync {
    fn code(&self) -> &'static str;
    fn evaluate(&self, context: &AnalysisContext<'_>, output: &mut Vec<Diagnostic>);
}

#[derive(Default)]
pub struct DiagnosticEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl DiagnosticEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_builtin_rules() -> Self {
        let mut engine = Self::new();
        engine.register(AttributeContradiction);
        engine
    }

    pub fn register(&mut self, rule: impl Rule + 'static) {
        self.rules.push(Box::new(rule));
    }

    #[must_use]
    pub fn analyze(&self, context: &AnalysisContext<'_>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for rule in &self.rules {
            rule.evaluate(context, &mut diagnostics);
        }
        diagnostics
    }

    #[must_use]
    pub fn rule_codes(&self) -> Vec<&'static str> {
        self.rules.iter().map(|rule| rule.code()).collect()
    }
}

pub struct AttributeContradiction;

impl Rule for AttributeContradiction {
    fn code(&self) -> &'static str {
        ATTRIBUTE_CONTRADICTION
    }

    fn evaluate(&self, context: &AnalysisContext<'_>, output: &mut Vec<Diagnostic>) {
        let mut facts = context.facts.to_vec();
        facts.extend(derived_facts(context));
        emit_fact_conflicts(&facts, context.documents, output);
        emit_lexical_conflicts(&facts, context.documents, output);
    }
}

fn derived_facts(context: &AnalysisContext<'_>) -> Vec<Fact> {
    let mut derived = Vec::new();
    for document in context.documents {
        derived.extend(wiki_attribute_facts(context.project, document));
        derived.extend(inline_assertion_facts(context.project, document));
    }
    derived
}

fn wiki_attribute_facts(project: &Project, document: &Document) -> Vec<Fact> {
    if document.kind != DocumentKind::Wiki {
        return Vec::new();
    }
    let Some(entity) = document
        .content_json
        .get("entity")
        .and_then(|value| value.as_str())
    else {
        return Vec::new();
    };
    let Some(attributes) = document
        .content_json
        .get("attributes")
        .and_then(|value| value.as_object())
    else {
        return Vec::new();
    };
    attributes
        .iter()
        .filter_map(|(predicate, value)| {
            let value = value.as_str()?;
            Some(analysis_fact(
                project,
                document,
                entity,
                predicate,
                value,
                FactStatus::Confirmed,
                0,
                value.len() as u32,
            ))
        })
        .collect()
}

fn inline_assertion_facts(project: &Project, document: &Document) -> Vec<Fact> {
    let text = document_plain_text(&document.content_json);
    let mut facts = Vec::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while let Some(start_rel) = text[index..].find("[[") {
        let start = index + start_rel;
        let Some(end_rel) = text[start + 2..].find("]]") else {
            break;
        };
        let end = start + 2 + end_rel;
        let inner = &text[start + 2..end];
        if let Some((entity_pred, value)) = inner.split_once('=') {
            if let Some((entity, predicate)) = entity_pred.split_once('.') {
                facts.push(analysis_fact(
                    project,
                    document,
                    entity,
                    predicate,
                    value,
                    FactStatus::Proposed,
                    start as u32,
                    (end + 2) as u32,
                ));
            }
        }
        index = end + 2;
        if index >= bytes.len() {
            break;
        }
    }
    facts
}

fn analysis_fact(
    project: &Project,
    document: &Document,
    entity: &str,
    predicate: &str,
    value: &str,
    status: FactStatus,
    start: u32,
    end: u32,
) -> Fact {
    Fact {
        id: margin_domain::FactId::new(),
        project_id: project.id,
        entity_key: normalize_key(entity),
        predicate: normalize_key(predicate),
        value: value.trim().to_owned(),
        modality: FactModality::Established,
        status,
        valid_from: None,
        valid_to: None,
        source: Some(SourceAnchor {
            document_id: document.id,
            start,
            end,
        }),
        created_at: project.created_at,
        updated_at: project.updated_at,
    }
}

fn emit_fact_conflicts(facts: &[Fact], documents: &[Document], output: &mut Vec<Diagnostic>) {
    for (index, left) in facts.iter().enumerate() {
        if !is_comparable(left) {
            continue;
        }
        for right in facts.iter().skip(index + 1) {
            if !is_comparable(right)
                || left.entity_key != right.entity_key
                || left.predicate != right.predicate
                || !intervals_overlap(left, right)
                || normalize_value(&left.value) == normalize_value(&right.value)
            {
                continue;
            }
            let severity = if left.status == FactStatus::Confirmed
                && right.status == FactStatus::Confirmed
            {
                Severity::Error
            } else {
                Severity::Warning
            };
            if let Some(source) = range_for_facts(left, right, documents) {
                output.push(conflict_diagnostic(
                    severity,
                    &left.entity_key,
                    &left.predicate,
                    &left.value,
                    &right.value,
                    source,
                    Some(format!(
                        "Confirmed or asserted values disagree over overlapping story time. \
                         Update canon, revise the passage, record a change event, or suppress later."
                    )),
                ));
            }
        }
    }
}

fn emit_lexical_conflicts(facts: &[Fact], documents: &[Document], output: &mut Vec<Diagnostic>) {
    for fact in facts {
        if fact.status != FactStatus::Confirmed
            || fact.modality == FactModality::Contested
            || !is_color_predicate(&fact.predicate)
        {
            continue;
        }
        let canon = normalize_value(&fact.value);
        for document in documents {
            if document.kind == DocumentKind::Wiki {
                continue;
            }
            let text = document_plain_text(&document.content_json);
            let Some(mention) = find_word(&text, &fact.entity_key) else {
                continue;
            };
            let window = surrounding_window(&text, mention, fact.entity_key.len(), 80);
            for color in COLOR_WORDS {
                let observed = normalize_value(color);
                if observed == canon {
                    continue;
                }
                if find_word(&window, color).is_none() {
                    continue;
                }
                let start = mention as u32;
                let end = start + fact.entity_key.len() as u32;
                output.push(conflict_diagnostic(
                    Severity::Warning,
                    &fact.entity_key,
                    &fact.predicate,
                    &fact.value,
                    color,
                    SourceRange {
                        document_id: document.id,
                        start,
                        end,
                    },
                    Some(format!(
                        "The passage mentions {} near a color that is not the confirmed {}.",
                        fact.entity_key, fact.value
                    )),
                ));
                break;
            }
        }
    }
}

fn conflict_diagnostic(
    severity: Severity,
    entity: &str,
    predicate: &str,
    left: &str,
    right: &str,
    source: SourceRange,
    details: Option<String>,
) -> Diagnostic {
    Diagnostic {
        id: DiagnosticId::new(),
        code: ATTRIBUTE_CONTRADICTION.to_owned(),
        severity,
        message: format!("{entity} · {predicate} · {left} conflicts with {right}"),
        source,
        details,
    }
}

fn is_comparable(fact: &Fact) -> bool {
    fact.status != FactStatus::Deprecated && fact.modality != FactModality::Contested
}

fn is_color_predicate(predicate: &str) -> bool {
    matches!(predicate, "hair_color" | "eye_color" | "haircolour" | "eyecolour")
}

fn intervals_overlap(left: &Fact, right: &Fact) -> bool {
    !range_before(left.valid_to.as_deref(), right.valid_from.as_deref())
        && !range_before(right.valid_to.as_deref(), left.valid_from.as_deref())
}

fn range_before(end: Option<&str>, start: Option<&str>) -> bool {
    match (end, start) {
        (Some(end), Some(start)) => end < start,
        _ => false,
    }
}

fn range_for_facts(left: &Fact, right: &Fact, documents: &[Document]) -> Option<SourceRange> {
    left.source
        .clone()
        .or(right.source.clone())
        .map(SourceRange::from)
        .or_else(|| {
            documents.first().map(|document| SourceRange {
                document_id: document.id,
                start: 0,
                end: 0,
            })
        })
}

fn find_word(haystack: &str, needle: &str) -> Option<usize> {
    let hay = haystack.to_ascii_lowercase();
    let needle = needle.to_ascii_lowercase();
    if needle.is_empty() {
        return None;
    }
    let mut offset = 0;
    while let Some(found) = hay[offset..].find(&needle) {
        let index = offset + found;
        let before = index
            .checked_sub(1)
            .and_then(|pos| hay.as_bytes().get(pos))
            .copied();
        let after = hay.as_bytes().get(index + needle.len()).copied();
        if before.is_none_or(|byte| !byte.is_ascii_alphanumeric())
            && after.is_none_or(|byte| !byte.is_ascii_alphanumeric())
        {
            return Some(index);
        }
        offset = index + 1;
    }
    None
}

fn surrounding_window(text: &str, start: usize, len: usize, radius: usize) -> String {
    let from = start.saturating_sub(radius);
    let to = (start + len).saturating_add(radius).min(text.len());
    text[from..to].to_owned()
}

#[cfg(test)]
mod tests {
    use super::{AnalysisContext, DiagnosticEngine};
    use chrono::Utc;
    use margin_domain::{
        Document, DocumentId, DocumentKind, Fact, FactId, FactModality, FactStatus, Project,
        ProjectId, ProjectKind, Severity,
    };
    use serde_json::json;

    fn project() -> Project {
        let now = Utc::now();
        Project {
            id: ProjectId::new(),
            name: "Test".into(),
            kind: ProjectKind::Novel,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn document(project: &Project, kind: DocumentKind, content: serde_json::Value) -> Document {
        let now = Utc::now();
        Document {
            id: DocumentId::new(),
            project_id: project.id,
            parent_id: None,
            kind,
            title: "Scene".into(),
            content_json: content,
            position: 0,
            revision: 1,
            created_at: now,
            updated_at: now,
        }
    }

    fn fact(project: &Project, value: &str, status: FactStatus, modality: FactModality) -> Fact {
        let now = Utc::now();
        Fact {
            id: FactId::new(),
            project_id: project.id,
            entity_key: "mira".into(),
            predicate: "hair_color".into(),
            value: value.into(),
            modality,
            status,
            valid_from: None,
            valid_to: None,
            source: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn empty_engine_has_no_rules() {
        assert!(DiagnosticEngine::new().rule_codes().is_empty());
    }

    #[test]
    fn confirmed_attribute_clash_is_an_error() {
        let project = project();
        let scene = document(&project, DocumentKind::Scene, json!({ "text": "Later." }));
        let engine = DiagnosticEngine::with_builtin_rules();
        let facts = vec![
            fact(&project, "auburn", FactStatus::Confirmed, FactModality::Established),
            fact(&project, "black", FactStatus::Confirmed, FactModality::Established),
        ];
        let diagnostics = engine.analyze(&AnalysisContext {
            project: &project,
            documents: &[scene],
            facts: &facts,
        });
        assert!(
            diagnostics
                .iter()
                .any(|item| item.code == "CANON-E013" && item.severity == Severity::Error)
        );
    }

    #[test]
    fn contested_facts_do_not_emit() {
        let project = project();
        let scene = document(&project, DocumentKind::Scene, json!({ "text": "Later." }));
        let engine = DiagnosticEngine::with_builtin_rules();
        let facts = vec![
            fact(&project, "auburn", FactStatus::Confirmed, FactModality::Established),
            fact(&project, "black", FactStatus::Confirmed, FactModality::Contested),
        ];
        let diagnostics = engine.analyze(&AnalysisContext {
            project: &project,
            documents: &[scene],
            facts: &facts,
        });
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn prose_color_clash_is_a_warning() {
        let project = project();
        let scene = document(
            &project,
            DocumentKind::Scene,
            json!({ "text": "Mira pushed her black hair out of her eyes." }),
        );
        let engine = DiagnosticEngine::with_builtin_rules();
        let facts = vec![fact(
            &project,
            "auburn",
            FactStatus::Confirmed,
            FactModality::Established,
        )];
        let diagnostics = engine.analyze(&AnalysisContext {
            project: &project,
            documents: &[scene],
            facts: &facts,
        });
        assert!(
            diagnostics
                .iter()
                .any(|item| item.code == "CANON-E013" && item.severity == Severity::Warning)
        );
    }

    #[test]
    fn wiki_and_inline_annotation_clash() {
        let project = project();
        let wiki = document(
            &project,
            DocumentKind::Wiki,
            json!({ "entity": "Mira", "attributes": { "hair_color": "auburn" } }),
        );
        let scene = document(
            &project,
            DocumentKind::Scene,
            json!({ "text": "[[mira.hair_color=black]]" }),
        );
        let engine = DiagnosticEngine::with_builtin_rules();
        let diagnostics = engine.analyze(&AnalysisContext {
            project: &project,
            documents: &[wiki, scene],
            facts: &[],
        });
        assert!(diagnostics.iter().any(|item| item.code == "CANON-E013"));
    }
}
