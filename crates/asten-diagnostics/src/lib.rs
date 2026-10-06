use asten_domain::{
    CanonEntity, Diagnostic, DiagnosticId, Document, DocumentKind, Fact, FactModality, FactStatus,
    MethodPack, Project, Severity, SourceAnchor, SourceRange, Suppression, TimelineEvent,
    document_plain_text, normalize_key, normalize_value, story_times_out_of_order,
};

pub const ATTRIBUTE_CONTRADICTION: &str = "CANON-E013";
pub const LIFECYCLE_AFTER_DEATH: &str = "CANON-E101";
pub const CLOCK_ORDER: &str = "CANON-E201";

const COLOR_WORDS: &[&str] = &[
    "amber", "auburn", "black", "blond", "blonde", "blue", "brown", "ginger", "gray", "green",
    "grey", "hazel", "pink", "red", "silver", "white",
];

pub struct AnalysisContext<'a> {
    pub project: &'a Project,
    pub documents: &'a [Document],
    pub facts: &'a [Fact],
    pub entities: &'a [CanonEntity],
    pub events: &'a [TimelineEvent],
    pub suppressions: &'a [Suppression],
    pub packs: &'a [MethodPack],
}

impl<'a> AnalysisContext<'a> {
    #[must_use]
    pub fn core(project: &'a Project, documents: &'a [Document], facts: &'a [Fact]) -> Self {
        Self {
            project,
            documents,
            facts,
            entities: &[],
            events: &[],
            suppressions: &[],
            packs: &[],
        }
    }

    #[must_use]
    pub fn pack_enabled(&self, key: &str) -> bool {
        self.packs
            .iter()
            .find(|pack| pack.pack_key == key)
            .map(|pack| pack.enabled)
            .unwrap_or(key == "scribe")
    }
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
        engine.register(LifecycleContradiction);
        engine.register(ClockContradiction);
        engine.register(DictionHints);
        engine
    }

    pub fn register(&mut self, rule: impl Rule + 'static) {
        self.rules.push(Box::new(rule));
    }

    #[must_use]
    pub fn analyze(&self, context: &AnalysisContext<'_>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for rule in &self.rules {
            if !rule_allowed(context, rule.code()) {
                continue;
            }
            rule.evaluate(context, &mut diagnostics);
        }
        diagnostics.retain(|item| !is_suppressed(context, item));
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
                (0, value.len() as u32),
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
                    (start as u32, (end + 2) as u32),
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
    span: (u32, u32),
) -> Fact {
    Fact {
        id: asten_domain::FactId::new(),
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
            start: span.0,
            end: span.1,
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
            let severity =
                if left.status == FactStatus::Confirmed && right.status == FactStatus::Confirmed {
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
                    Some(
                        "Confirmed or asserted values disagree over overlapping story time. \
                         Update canon, revise the passage, record a change event, or suppress later."
                            .to_owned(),
                    ),
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
    matches!(
        predicate,
        "hair_color" | "eye_color" | "haircolour" | "eyecolour"
    )
}

fn rule_allowed(context: &AnalysisContext<'_>, code: &str) -> bool {
    match code {
        ATTRIBUTE_CONTRADICTION => true,
        LIFECYCLE_AFTER_DEATH => context.pack_enabled("lifecycle"),
        CLOCK_ORDER => context.pack_enabled("clocks"),
        "CRAFT-H001" | "CRAFT-H002" => context.pack_enabled("diction"),
        _ => true,
    }
}

fn is_suppressed(context: &AnalysisContext<'_>, diagnostic: &Diagnostic) -> bool {
    let entity = diagnostic.message.split(" · ").next().map(normalize_key);
    context.suppressions.iter().any(|suppression| {
        suppression.hides(
            &diagnostic.code,
            diagnostic.source.document_id,
            entity.as_deref(),
        )
    })
}

pub struct LifecycleContradiction;

impl Rule for LifecycleContradiction {
    fn code(&self) -> &'static str {
        LIFECYCLE_AFTER_DEATH
    }

    fn evaluate(&self, context: &AnalysisContext<'_>, output: &mut Vec<Diagnostic>) {
        for fact in context.facts {
            if fact.status != FactStatus::Confirmed
                || fact.modality == FactModality::Contested
                || !is_death_fact(fact)
            {
                continue;
            }
            let death_order = context
                .events
                .iter()
                .filter(|event| event_matches_entity(event, &fact.entity_key, context))
                .filter_map(|event| event.reading_order)
                .min();
            let Some(death_order) = death_order else {
                continue;
            };
            for event in context.events {
                let Some(order) = event.reading_order else {
                    continue;
                };
                if order <= death_order || !event_matches_entity(event, &fact.entity_key, context) {
                    continue;
                }
                if looks_like_flashback(&event.notes) {
                    continue;
                }
                let source = event
                    .document_id
                    .or_else(|| context.documents.first().map(|document| document.id));
                let Some(document_id) = source else {
                    continue;
                };
                output.push(Diagnostic {
                    id: DiagnosticId::new(),
                    code: LIFECYCLE_AFTER_DEATH.to_owned(),
                    severity: Severity::Error,
                    message: format!(
                        "{} acts after a settled death ({})",
                        fact.entity_key, event.title
                    ),
                    source: SourceRange {
                        document_id,
                        start: 0,
                        end: 0,
                    },
                    details: Some(
                        "A later reading-order beat still belongs to someone you marked dead. \
                         Mark the beat a flashback, move the death, or suppress the rule."
                            .into(),
                    ),
                });
            }
        }
    }
}

fn is_death_fact(fact: &Fact) -> bool {
    matches!(fact.predicate.as_str(), "lifecycle" | "status" | "alive")
        && matches!(
            normalize_value(&fact.value).as_str(),
            "dead" | "deceased" | "died" | "false" | "no"
        )
}

fn event_matches_entity(
    event: &TimelineEvent,
    entity_key: &str,
    context: &AnalysisContext<'_>,
) -> bool {
    if let Some(entity_id) = event.entity_id {
        return context
            .entities
            .iter()
            .any(|entity| entity.id == entity_id && normalize_key(&entity.name) == entity_key);
    }
    normalize_key(&event.title).contains(entity_key)
        || normalize_key(&event.notes).contains(entity_key)
}

fn looks_like_flashback(notes: &str) -> bool {
    let lower = notes.to_ascii_lowercase();
    lower.contains("flashback") || lower.contains("memory") || lower.contains("earlier")
}

pub struct ClockContradiction;

impl Rule for ClockContradiction {
    fn code(&self) -> &'static str {
        CLOCK_ORDER
    }

    fn evaluate(&self, context: &AnalysisContext<'_>, output: &mut Vec<Diagnostic>) {
        let mut dated: Vec<&TimelineEvent> = context
            .events
            .iter()
            .filter(|event| {
                event
                    .story_time
                    .as_ref()
                    .is_some_and(|value| !value.trim().is_empty())
            })
            .filter(|event| event.reading_order.is_some())
            .collect();
        dated.sort_by_key(|event| event.reading_order);
        for pair in dated.windows(2) {
            let earlier = pair[0];
            let later = pair[1];
            let (Some(left), Some(right)) = (&earlier.story_time, &later.story_time) else {
                continue;
            };
            if !story_times_out_of_order(left, right) {
                continue;
            }
            let document_id = later
                .document_id
                .or(earlier.document_id)
                .or_else(|| context.documents.first().map(|document| document.id));
            let Some(document_id) = document_id else {
                continue;
            };
            output.push(Diagnostic {
                id: DiagnosticId::new(),
                code: CLOCK_ORDER.to_owned(),
                severity: Severity::Error,
                message: format!(
                    "reading order places “{}” before “{}”, but story time runs the other way",
                    earlier.title, later.title
                ),
                source: SourceRange {
                    document_id,
                    start: 0,
                    end: 0,
                },
                details: Some(
                    "Story time and reading order are different clocks. Reorder the binder, \
                     retcon a date, or mark one beat a flashback."
                        .into(),
                ),
            });
        }
    }
}

pub struct DictionHints;

impl Rule for DictionHints {
    fn code(&self) -> &'static str {
        "CRAFT-H001"
    }

    fn evaluate(&self, context: &AnalysisContext<'_>, output: &mut Vec<Diagnostic>) {
        if !context.pack_enabled("diction") {
            return;
        }
        let Some(pack) = context.packs.iter().find(|pack| pack.pack_key == "diction") else {
            return;
        };
        let Some(hints) = pack
            .manifest
            .get("hints")
            .and_then(|value| value.as_array())
        else {
            return;
        };
        for document in context.documents {
            if document.kind == DocumentKind::Wiki {
                continue;
            }
            let text = document_plain_text(&document.content_json);
            for hint in hints {
                let Some(word) = hint.get("word").and_then(|value| value.as_str()) else {
                    continue;
                };
                let Some(found) = find_word(&text, word) else {
                    continue;
                };
                output.push(Diagnostic {
                    id: DiagnosticId::new(),
                    code: hint
                        .get("code")
                        .and_then(|value| value.as_str())
                        .unwrap_or("CRAFT-H001")
                        .to_owned(),
                    severity: Severity::Hint,
                    message: hint
                        .get("message")
                        .and_then(|value| value.as_str())
                        .unwrap_or("Optional diction note.")
                        .to_owned(),
                    source: SourceRange {
                        document_id: document.id,
                        start: found as u32,
                        end: (found + word.len()) as u32,
                    },
                    details: Some(
                        "Craft hint from an enabled method pack. Not a canon error.".into(),
                    ),
                });
            }
        }
    }
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
    use asten_domain::{
        Document, DocumentId, DocumentKind, Fact, FactId, FactModality, FactStatus, Project,
        ProjectId, ProjectKind, Severity,
    };
    use chrono::Utc;
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
            exclude_from_compile: false,
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
            fact(
                &project,
                "auburn",
                FactStatus::Confirmed,
                FactModality::Established,
            ),
            fact(
                &project,
                "black",
                FactStatus::Confirmed,
                FactModality::Established,
            ),
        ];
        let diagnostics = engine.analyze(&AnalysisContext::core(&project, &[scene], &facts));
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
            fact(
                &project,
                "auburn",
                FactStatus::Confirmed,
                FactModality::Established,
            ),
            fact(
                &project,
                "black",
                FactStatus::Confirmed,
                FactModality::Contested,
            ),
        ];
        let diagnostics = engine.analyze(&AnalysisContext::core(&project, &[scene], &facts));
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
        let diagnostics = engine.analyze(&AnalysisContext::core(&project, &[scene], &facts));
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
        let diagnostics = engine.analyze(&AnalysisContext::core(&project, &[wiki, scene], &[]));
        assert!(diagnostics.iter().any(|item| item.code == "CANON-E013"));
    }
}
