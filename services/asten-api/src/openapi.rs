//! The OpenAPI 3.1 document, generated at compile time from the
//! `#[utoipa::path]` annotation on every handler and the `ToSchema` derives on
//! the contract crates, and the public route that serves it.
//!
//! `node scripts/openapi-export.mjs` writes the document to `docs/openapi.json`
//! and the route table into `docs/API.md`. The tests below hold the document to
//! the router (every route annotated, every annotation routed), to the
//! security model, and to real serialised values of the shapes that needed a
//! hand-written schema.

use std::sync::LazyLock;

use axum::{Router, http::header, response::IntoResponse, routing::get};
use serde_json::json;
use utoipa::openapi::extensions::Extensions;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::ApiState;

/// Everything under `/api/v1`, with the same paths the routers register.
#[derive(OpenApi)]
#[openapi(
    components(schemas(asten_domain::DocumentSummary)),
    paths(
        openapi_json,
        crate::desk_info,
        crate::list_projects,
        crate::get_project,
        crate::create_project,
        crate::list_documents,
        crate::get_document,
        crate::create_document,
        crate::update_document,
        crate::delete_document,
        crate::list_facts,
        crate::create_fact,
        crate::update_fact,
        crate::delete_fact,
        crate::list_diagnostics,
        crate::list_world_entries,
        crate::create_world_entry,
        crate::update_world_entry,
        crate::world_pulse,
        crate::auth::status,
        crate::auth::pair,
        crate::auth::login,
        crate::auth::setup_owner,
        crate::auth::me,
        crate::auth::logout,
        crate::auth::start_pairing,
        crate::auth::list_tokens,
        crate::auth::revoke_token,
        crate::auth::change_passphrase,
        crate::auth::stream_ticket,
        crate::story::stream,
        crate::story::get_settings,
        crate::story::update_settings,
        crate::story::diagnostics,
        crate::story::fact_card,
        crate::story::mentions,
        crate::story::where_settled,
        crate::story::scene_summary,
        crate::story::cast_search,
        crate::story::apply_repair,
        crate::story::bisect,
        crate::story::catalog,
        crate::story::rename,
        crate::story::list_invariants,
        crate::story::create_invariant,
        crate::story::update_invariant,
        crate::story::delete_invariant,
        crate::story::list_routes,
        crate::story::create_route,
        crate::story::delete_route,
        crate::story::list_baselines,
        crate::story::create_baseline,
        crate::story::activate_baseline,
        crate::story::clear_baseline,
        crate::story::delete_baseline,
        crate::history::history,
        crate::history::verify,
        crate::history::at_seq,
        crate::history::blame,
        crate::history::list_drafts,
        crate::history::create_draft,
        crate::history::update_draft,
        crate::history::delete_draft,
        crate::history::merge_preview,
        crate::history::merge_draft,
        crate::extra::list_entities,
        crate::extra::create_entity,
        crate::extra::update_entity,
        crate::extra::delete_entity,
        crate::extra::merge_entity,
        crate::extra::list_proposals,
        crate::extra::list_events,
        crate::extra::create_event,
        crate::extra::update_event,
        crate::extra::delete_event,
        crate::extra::list_comments,
        crate::extra::create_comment,
        crate::extra::update_comment,
        crate::extra::list_ops,
        crate::extra::list_snapshots,
        crate::extra::create_snapshot,
        crate::extra::restore_snapshot,
        crate::extra::list_suppressions,
        crate::extra::create_suppression,
        crate::extra::delete_suppression,
        crate::extra::list_packs,
        crate::extra::update_pack,
        crate::extra::list_changes,
        crate::extra::create_change,
        crate::extra::resolve_change,
        crate::extra::compile_project,
        crate::extra::import_document,
    )
)]
struct V1;

/// The whole API: `/health` at the root and everything else under `/api/v1`.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Asten API",
        description = "The one transport of Asten, a local-first writing desk. The browser desk, the desktop shell's webview, the Word task pane and the Docs sidebar all speak this API, usually over loopback; nothing here reaches any other server.\n\nPair once with a six-digit code (`POST /api/v1/auth/pair`) and send the token as `Authorization: Bearer <token>`; a browser served by the API itself may rely on the `asten_session` cookie instead. The change stream takes a one-minute single-use ticket in the query string because an EventSource cannot set headers. Token kinds carry authority: `desk` and owner `session` tokens may manage pairing, tokens and the passphrase; `guest` tokens may read and write the story and revoke only themselves. `ASTEN_AUTH` selects the mode (`pairing`, `owner`, `off`); the `x-asten` extension below lists them.\n\nEvery error is JSON of the form `{ \"error\": { \"code\": \"404\", \"message\": \"…\" } }`. See `docs/API.md` for the curl walk-through and `docs/SECURITY.md` for the threat model."
    ),
    servers((url = "http://127.0.0.1:4317", description = "The local API at its default bind")),
    paths(crate::health),
    nest((path = "/api/v1", api = V1)),
    tags(
        (name = "desk", description = "What this API is and where it runs"),
        (name = "auth", description = "Pairing codes, tokens, the owner passphrase and stream tickets"),
        (name = "projects", description = "Projects and their engine settings"),
        (name = "documents", description = "The binder: documents, comments and tracked changes"),
        (name = "canon", description = "Facts, entities, timeline events, invariants and travel routes"),
        (name = "story", description = "The story language server: diagnostics, fact cards, mentions, repairs, suppressions, baselines and the change stream"),
        (name = "history", description = "The hash-chained event log, verification, blame, snapshots and drafts"),
        (name = "world", description = "The worldbuilding board and its pulse"),
        (name = "packs", description = "Method packs of predicates, cues and rules"),
        (name = "compile", description = "Compile the manuscript out, import documents in"),
    ),
    modifiers(&Security)
)]
pub struct ApiDoc;

/// Adds the bearer scheme every protected route names, and the `x-asten`
/// extension that lists the authentication modes, token kinds and credentials.
struct Security;

impl Modify for Security {
    fn modify(&self, document: &mut utoipa::openapi::OpenApi) {
        let components = document.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "asten_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("ast_<base64url>")
                    .description(Some(
                        "A token from `POST /api/v1/auth/pair` or `POST /api/v1/auth/login`, \
                         shown once. Browsers served by the API may present the `asten_session` \
                         cookie instead; the stream route also accepts a one-minute ticket as \
                         `?ticket=`.",
                    ))
                    .build(),
            ),
        );
        // Keys are in alphabetical order so the document serialises the same
        // whether or not serde_json preserves insertion order.
        document.info.extensions = Some(Extensions::from_iter([(
            "x-asten",
            json!({
                "auth_modes": [
                    {
                        "mode": "off",
                        "summary": "Everyone is a desk. Loopback only; the server refuses to start with it on another bind."
                    },
                    {
                        "mode": "owner",
                        "summary": "Paired clients, plus a browser that knows the owner passphrase (`POST /auth/login`)."
                    },
                    {
                        "mode": "pairing",
                        "summary": "Clients that exchanged a one-time six-digit code. The default, and what the desktop shell uses."
                    }
                ],
                "credentials": {
                    "cookie": "asten_session",
                    "header": "Authorization: Bearer <token>",
                    "stream_ticket": {
                        "query_parameter": "ticket",
                        "route": "POST /api/v1/projects/{project_id}/stream-ticket",
                        "single_use": true,
                        "ttl_seconds": 60
                    }
                },
                "default_mode": "pairing",
                "environment_variable": "ASTEN_AUTH",
                "public_routes": [
                    "GET /health",
                    "GET /api/v1/openapi.json",
                    "GET /api/v1/auth/status",
                    "POST /api/v1/auth/pair",
                    "POST /api/v1/auth/login",
                    "POST /api/v1/auth/owner/setup"
                ],
                "token_kinds": [
                    { "kind": "desk", "may_administer": true },
                    { "kind": "guest", "may_administer": false },
                    { "kind": "session", "may_administer": true }
                ]
            }),
        )]));
    }
}

static DOCUMENT: LazyLock<String> = LazyLock::new(|| {
    ApiDoc::openapi()
        .to_pretty_json()
        .expect("the OpenAPI document serialises")
});

/// The document as the JSON text the route serves and the export writes.
#[must_use]
pub fn document_json() -> &'static str {
    &DOCUMENT
}

#[utoipa::path(
    get,
    path = "/openapi.json",
    tag = "desk",
    summary = "This document",
    description = "The OpenAPI 3.1 description of the API, generated from the code. Public: it describes the API and holds no data.",
    responses(
        (status = 200, description = "The document", body = serde_json::Value)
    )
)]
async fn openapi_json() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/json")],
        DOCUMENT.as_str(),
    )
}

/// The document needs no token.
pub fn public_router() -> Router<ApiState> {
    Router::new().route("/openapi.json", get(openapi_json))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Arc;

    use asten_ir::{
        Anchor, Atom, Context, EntityId, Event, EventId, EventKind, EventKindPattern, Fluent,
        Invariant, InvariantPattern, Mention, Observation, ObservationId, PredicateId, Proposition,
        Provenance, SettledAt, Status, StoryIr, TemporalConstraint, TimeBounds, TimePoint, TimeRef,
        TraceClock, Value as IrValue,
    };
    use asten_ls::{DiagnosticsDelta, StreamEvent};
    use asten_time::calendar::{Calendar, FictionalCalendar, LeapRule, Month};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{Map, Value, json};
    use utoipa::ToSchema;

    use super::*;

    const SOURCES: &[&str] = &[
        include_str!("lib.rs"),
        include_str!("auth.rs"),
        include_str!("story.rs"),
        include_str!("history.rs"),
        include_str!("extra.rs"),
        include_str!("openapi.rs"),
    ];

    const METHODS: [&str; 5] = ["get", "post", "put", "delete", "patch"];

    type Routes = BTreeMap<String, BTreeSet<String>>;

    /// `/health` is registered on the outer router; everything else is nested
    /// under `/api/v1`.
    fn full_path(path: &str) -> String {
        if path == "/health" {
            path.to_owned()
        } else {
            format!("/api/v1{path}")
        }
    }

    /// Every `Router::route` call in the handler modules, parsed from source:
    /// the path literal, then the method constructors in the chain that follows.
    fn routes_in_source() -> Routes {
        // Spelled in two halves so this function does not match itself.
        let needle = concat!(".rou", "te(");
        let mut routes = Routes::new();
        for source in SOURCES {
            let mut rest = *source;
            while let Some(index) = rest.find(needle) {
                let call = &rest[index + needle.len()..];
                let open = call
                    .find('"')
                    .expect("a route starts with its path literal");
                let close = open + 1 + call[open + 1..].find('"').expect("the literal closes");
                let path = &call[open + 1..close];
                let mut depth = 1usize;
                let mut end = call.len();
                for (offset, character) in call.char_indices() {
                    match character {
                        '(' => depth += 1,
                        ')' => {
                            depth -= 1;
                            if depth == 0 {
                                end = offset;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let chain = &call[close + 1..end];
                for method in METHODS {
                    let marker = format!("{method}(");
                    let mut search = chain;
                    while let Some(found) = search.find(&marker) {
                        let preceded_by_identifier = found > 0 && {
                            let before = search.as_bytes()[found - 1];
                            before.is_ascii_alphanumeric() || before == b'_'
                        };
                        if !preceded_by_identifier {
                            routes
                                .entry(full_path(path))
                                .or_default()
                                .insert(method.to_owned());
                        }
                        search = &search[found + marker.len()..];
                    }
                }
                rest = &call[end..];
            }
        }
        routes
    }

    fn document() -> Value {
        serde_json::to_value(ApiDoc::openapi()).expect("the document serialises")
    }

    /// `(method, path, operation)` for every operation in the document.
    fn operations(document: &Value) -> Vec<(String, String, &Value)> {
        let mut operations = Vec::new();
        for (path, item) in document["paths"].as_object().expect("paths") {
            for (method, operation) in item.as_object().expect("a path item") {
                if METHODS.contains(&method.as_str()) {
                    operations.push((method.clone(), path.clone(), operation));
                }
            }
        }
        operations
    }

    fn routes_in_document(document: &Value) -> Routes {
        let mut routes = Routes::new();
        for (method, path, _) in operations(document) {
            routes.entry(path).or_default().insert(method);
        }
        routes
    }

    fn difference(left: &Routes, right: &Routes) -> Vec<String> {
        left.iter()
            .flat_map(|(path, methods)| {
                methods
                    .iter()
                    .filter(|method| !right.get(path).is_some_and(|set| set.contains(*method)))
                    .map(move |method| format!("{} {path}", method.to_uppercase()))
            })
            .collect()
    }

    #[test]
    fn every_route_is_documented_and_every_documented_route_exists() {
        let source = routes_in_source();
        let documented = routes_in_document(&document());
        let undocumented = difference(&source, &documented);
        let unrouted = difference(&documented, &source);
        assert!(
            undocumented.is_empty(),
            "routes without a #[utoipa::path] annotation: {undocumented:#?}"
        );
        assert!(
            unrouted.is_empty(),
            "annotations without a route: {unrouted:#?}"
        );
        let count: usize = source.values().map(BTreeSet::len).sum();
        assert!(
            count > 80,
            "expected the whole API, found {count} operations"
        );
    }

    #[test]
    fn every_operation_has_a_tag_and_a_summary_and_distinct_parameters() {
        let document = document();
        let tags: BTreeSet<&str> = document["tags"]
            .as_array()
            .expect("tags")
            .iter()
            .map(|tag| tag["name"].as_str().expect("a tag name"))
            .collect();
        for (method, path, operation) in operations(&document) {
            let where_ = format!("{} {path}", method.to_uppercase());
            let tag = operation["tags"][0].as_str();
            assert!(
                tag.is_some_and(|tag| tags.contains(tag)),
                "{where_} needs one of the declared tags, has {tag:?}"
            );
            assert!(
                operation["summary"].as_str().is_some_and(|s| !s.is_empty()),
                "{where_} needs a summary"
            );
            let mut names = BTreeSet::new();
            for parameter in operation["parameters"].as_array().into_iter().flatten() {
                let name = parameter["name"].as_str().expect("a parameter name");
                assert!(
                    names.insert(name),
                    "{where_} declares the parameter {name} twice"
                );
            }
            let placeholders = path.split('/').filter_map(|segment| {
                segment
                    .strip_prefix('{')
                    .and_then(|segment| segment.strip_suffix('}'))
            });
            for placeholder in placeholders {
                assert!(
                    names.contains(placeholder),
                    "{where_} does not describe its path parameter {placeholder}"
                );
            }
        }
        let fact_card = &document["paths"]["/api/v1/projects/{project_id}/story/fact-card"]["get"];
        let query: BTreeSet<&str> = fact_card["parameters"]
            .as_array()
            .expect("parameters")
            .iter()
            .filter(|p| p["in"] == "query")
            .map(|p| p["name"].as_str().expect("a name"))
            .collect();
        assert_eq!(query, BTreeSet::from(["entity", "document"]));
    }

    #[test]
    fn protected_routes_name_the_bearer_scheme_and_public_ones_do_not() {
        let document = document();
        assert_eq!(
            document["components"]["securitySchemes"]["asten_token"]["scheme"],
            "bearer"
        );
        let public = BTreeSet::from([
            "GET /health",
            "GET /api/v1/openapi.json",
            "GET /api/v1/auth/status",
            "POST /api/v1/auth/pair",
            "POST /api/v1/auth/login",
        ]);
        let optional = "POST /api/v1/auth/owner/setup";
        for (method, path, operation) in operations(&document) {
            let where_ = format!("{} {path}", method.to_uppercase());
            let requirements: Vec<&Map<String, Value>> = operation["security"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|requirement| requirement.as_object().expect("a requirement"))
                .collect();
            let names_bearer = requirements
                .iter()
                .any(|requirement| requirement.contains_key("asten_token"));
            let allows_anonymous = requirements
                .iter()
                .any(|requirement| requirement.is_empty());
            if public.contains(where_.as_str()) {
                assert!(
                    requirements.is_empty(),
                    "{where_} is public and must declare no security"
                );
            } else if where_ == optional {
                assert!(
                    names_bearer && allows_anonymous,
                    "{where_} is open only until a passphrase exists"
                );
            } else {
                assert!(
                    names_bearer,
                    "{where_} is protected and must name asten_token"
                );
                assert!(
                    !allows_anonymous,
                    "{where_} must not allow anonymous callers"
                );
                assert!(
                    operation["responses"]["401"].is_object(),
                    "{where_} must document 401"
                );
            }
        }
        let modes: Vec<&str> = document["info"]["x-asten"]["auth_modes"]
            .as_array()
            .expect("auth modes")
            .iter()
            .map(|mode| mode["mode"].as_str().expect("a mode"))
            .collect();
        assert_eq!(modes, ["off", "owner", "pairing"]);
        let stream = &document["paths"]["/api/v1/projects/{project_id}/stream"]["get"];
        let description = stream["description"]
            .as_str()
            .expect("the stream is described");
        assert!(description.contains("asten_session") && description.contains("ticket"));
        assert!(stream["responses"]["200"]["content"]["text/event-stream"].is_object());
    }

    // ───────────── schemas against serialised values ─────────────

    /// A small JSON Schema checker: `$ref`, `allOf`, `oneOf`/`anyOf`, `enum`,
    /// `type`, `required`, `properties`, `additionalProperties`, `items` and
    /// `prefixItems`. Enough to prove a serialised value fits its schema.
    fn check(
        value: &Value,
        schema: &Value,
        components: &Map<String, Value>,
        at: &str,
    ) -> Result<(), String> {
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            let name = reference
                .strip_prefix("#/components/schemas/")
                .ok_or_else(|| format!("{at}: unexpected reference {reference}"))?;
            let target = components
                .get(name)
                .ok_or_else(|| format!("{at}: unresolved schema {name}"))?;
            return check(value, target, components, at);
        }
        if let Some(all) = schema.get("allOf").and_then(Value::as_array) {
            for (index, part) in all.iter().enumerate() {
                check(value, part, components, &format!("{at}/allOf[{index}]"))?;
            }
        }
        for key in ["oneOf", "anyOf"] {
            if let Some(options) = schema.get(key).and_then(Value::as_array) {
                let failures: Vec<String> = options
                    .iter()
                    .enumerate()
                    .filter_map(|(index, option)| {
                        check(value, option, components, &format!("{at}/{key}[{index}]")).err()
                    })
                    .collect();
                if failures.len() == options.len() {
                    return Err(format!(
                        "{at}: no alternative accepts {value}:\n    {}",
                        failures.join("\n    ")
                    ));
                }
            }
        }
        if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
            if !allowed.contains(value) {
                return Err(format!("{at}: {value} is not one of {allowed:?}"));
            }
        }
        let types: Vec<&str> = match schema.get("type") {
            Some(Value::String(one)) => vec![one.as_str()],
            Some(Value::Array(many)) => many.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        if !types.is_empty() {
            let kind = match value {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(number) if number.is_i64() || number.is_u64() => "integer",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
            };
            let fits = types
                .iter()
                .any(|wanted| *wanted == kind || (*wanted == "number" && kind == "integer"));
            if !fits {
                return Err(format!(
                    "{at}: {value} is {kind}, the schema wants {types:?}"
                ));
            }
        }
        if let Some(object) = value.as_object() {
            for name in schema
                .get("required")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                if !object.contains_key(name) {
                    return Err(format!("{at}: the required property {name} is missing"));
                }
            }
            if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
                for (name, property) in object {
                    if let Some(property_schema) = properties.get(name) {
                        check(
                            property,
                            property_schema,
                            components,
                            &format!("{at}.{name}"),
                        )?;
                    }
                }
            }
            if let Some(additional) = schema.get("additionalProperties").filter(|a| a.is_object()) {
                for (name, property) in object {
                    check(property, additional, components, &format!("{at}.{name}"))?;
                }
            }
        }
        if let Some(items) = value.as_array() {
            if let Some(prefix) = schema.get("prefixItems").and_then(Value::as_array) {
                for (index, (item, item_schema)) in items.iter().zip(prefix).enumerate() {
                    check(item, item_schema, components, &format!("{at}[{index}]"))?;
                }
            } else if let Some(item_schema) = schema.get("items").filter(|i| i.is_object()) {
                for (index, item) in items.iter().enumerate() {
                    check(item, item_schema, components, &format!("{at}[{index}]"))?;
                }
            }
        }
        Ok(())
    }

    /// The document's components plus the schemas `T` depends on, for types
    /// no handler reaches.
    fn components_with<T: ToSchema>(document: &Value) -> Map<String, Value> {
        let mut components = document["components"]["schemas"]
            .as_object()
            .expect("component schemas")
            .clone();
        let mut collected = Vec::new();
        T::schemas(&mut collected);
        collected.push((T::name().into_owned(), T::schema()));
        for (name, schema) in collected {
            components.insert(name, serde_json::to_value(schema).expect("a schema"));
        }
        components
    }

    fn anchor() -> Anchor {
        Anchor::new("doc-1", "block-1", 0, 4)
    }

    fn fluent() -> Fluent {
        Fluent {
            predicate: PredicateId::new("hair_color"),
            subject: EntityId::new("mira"),
            value: IrValue::Text {
                norm: "black".into(),
                label: "black".into(),
            },
        }
    }

    fn assert_fits<T: serde::Serialize>(
        failures: &mut Vec<String>,
        components: &Map<String, Value>,
        schema: &str,
        label: &str,
        value: &T,
    ) {
        let value = serde_json::to_value(value).expect("the example serialises");
        let reference = json!({ "$ref": format!("#/components/schemas/{schema}") });
        if let Err(failure) = check(&value, &reference, components, label) {
            failures.push(failure);
        }
    }

    #[test]
    fn hand_written_schemas_accept_real_serialised_values() {
        let document = document();
        let components = document["components"]["schemas"]
            .as_object()
            .expect("component schemas")
            .clone();
        let mut failures = Vec::new();

        let events = [
            StreamEvent::Revision {
                revision: "r1".into(),
            },
            StreamEvent::Diagnostics(DiagnosticsDelta {
                from_revision: "r0".into(),
                revision: "r1".into(),
                diff: asten_diagnostics::MarksDiff {
                    added: Vec::new(),
                    removed: vec!["CANON-E013:1".into()],
                    changed: Vec::new(),
                },
            }),
            StreamEvent::Document {
                document: "doc-1".into(),
                seq: 7,
            },
            StreamEvent::Canon {
                what: "facts".into(),
            },
            StreamEvent::Heartbeat,
        ];
        for (index, event) in events.iter().enumerate() {
            assert_fits(
                &mut failures,
                &components,
                "StreamEvent",
                &format!("StreamEvent[{index}]"),
                event,
            );
        }

        let calendars = [
            Calendar::DayCount,
            Calendar::Gregorian,
            Calendar::Fictional(FictionalCalendar {
                months: vec![Month {
                    name: "Thaw".into(),
                    days: 30,
                }],
                leap: Some(LeapRule {
                    every: 4,
                    extra_days: 1,
                    month: 0,
                }),
                epoch_year: 1,
                eras: vec!["Before the Flood".into()],
                seasons: Vec::new(),
            }),
        ];
        for (index, calendar) in calendars.iter().enumerate() {
            assert_fits(
                &mut failures,
                &components,
                "Calendar",
                &format!("Calendar[{index}]"),
                calendar,
            );
        }

        let with_atom = components_with::<Atom>(&document);
        let atoms = [
            Atom::Mention(Mention {
                anchor: anchor(),
                entity: EntityId::new("mira"),
                surface: "Mira".into(),
                confidence: 0.9,
                reading_pos: 1,
                context: Context::default(),
            }),
            Atom::Observation(Observation {
                id: ObservationId::new("obs-1"),
                fluent: fluent(),
                context: Context::Says { speaker: None },
                at: TimeRef::Unknown,
                anchor: anchor(),
                confidence: 0.8,
                negated: false,
                provenance: Provenance::Text { anchor: anchor() },
                reading_pos: 2,
            }),
            Atom::Constraint(TemporalConstraint {
                from: TimePoint::Origin,
                to: TimePoint::Named {
                    label: "the siege".into(),
                },
                lo: Some(3),
                hi: None,
                label: "three days after".into(),
                provenance: Provenance::default(),
                status: Status::Settled,
                confidence: 1.0,
                anchor: None,
            }),
            Atom::UnknownName {
                anchor: anchor(),
                surface: "Zed".into(),
                reading_pos: 3,
            },
            Atom::ContextSpan {
                anchor: anchor(),
                context: Context::Dream { key: "k".into() },
            },
            Atom::Event(Event {
                id: EventId::new("ev-1"),
                kind: EventKind::Dies,
                label: String::new(),
                subject: Some(EntityId::new("mira")),
                participants: Vec::new(),
                at: TimeRef::Unknown,
                reading_pos: None,
                context: Context::default(),
                status: Status::Settled,
                confidence: 1.0,
                provenance: Provenance::default(),
                anchor: None,
                document: None,
                initiates: Vec::new(),
                terminates: Vec::new(),
            }),
        ];
        for (index, atom) in atoms.iter().enumerate() {
            assert_fits(
                &mut failures,
                &with_atom,
                "Atom",
                &format!("Atom[{index}]"),
                atom,
            );
        }

        assert!(
            failures.is_empty(),
            "schemas rejected values:\n  {}",
            failures.join("\n  ")
        );
    }

    #[test]
    fn derived_schemas_accept_real_serialised_values() {
        let document = document();
        let components = document["components"]["schemas"]
            .as_object()
            .expect("component schemas")
            .clone();
        let mut failures = Vec::new();

        // Transparent ids, a date-time, an enum and the nested clock.
        let settings = asten_domain::ProjectSettings::for_project(asten_domain::ProjectId::new());
        assert_fits(
            &mut failures,
            &components,
            "ProjectSettings",
            "ProjectSettings",
            &settings,
        );

        // A free-form JSON value and a doubly optional field sent as null.
        let update = asten_application::UpdateDocument {
            title: Some("Arrival".into()),
            content_json: Some(json!({ "type": "doc", "content": [] })),
            parent_id: Some(None),
            expected_head_seq: Some(4),
            ..Default::default()
        };
        assert_fits(
            &mut failures,
            &components,
            "UpdateDocument",
            "UpdateDocument",
            &update,
        );

        // Fluent.value points at ir.Value, and a newtype variant beside unit ones.
        let invariant = Invariant {
            id: "inv-1".into(),
            label: "No one speaks after dying".into(),
            pattern: InvariantPattern::Response {
                trigger: Proposition::Holds { fluent: fluent() },
                response: Proposition::Happens {
                    subject: None,
                    event: EventKindPattern::Other("whispers".into()),
                },
            },
            clock: TraceClock::StoryTime,
            enabled: true,
        };
        assert_fits(
            &mut failures,
            &components,
            "Invariant",
            "Invariant",
            &invariant,
        );

        // The auth mode enum from another crate.
        let desk = asten_application::DeskInfo {
            version: "0.0.1-alpha".into(),
            host: "test".into(),
            embedded: false,
            auth_mode: asten_application::AuthMode::Pairing,
            database_path: None,
            logs_path: Some("C:/logs".into()),
            backups_path: Some("C:/backups".into()),
        };
        assert_fits(&mut failures, &components, "DeskInfo", "DeskInfo", &desk);

        // Tagged struct variants with a free-form payload, and a unit variant.
        let repairs = [
            asten_diagnostics::Repair::keep(),
            asten_diagnostics::Repair::new(
                asten_diagnostics::RepairKind::MarkContext {
                    document: "doc-1".into(),
                    context: Context::Says { speaker: None },
                },
                "Mark as dialogue",
                2.0,
            ),
        ];
        for (index, repair) in repairs.iter().enumerate() {
            assert_fits(
                &mut failures,
                &components,
                "Repair",
                &format!("Repair[{index}]"),
                repair,
            );
        }

        // Tick fields named as i64, a tuple, maps, and the IR fact.
        let with_ir = components_with::<StoryIr>(&document);
        let mut ir = StoryIr::default();
        ir.unknown_names.push((anchor(), "Zed".into(), 3));
        ir.facts.push(asten_ir::Fact {
            id: asten_ir::FactId::new("fact-1"),
            fluent: fluent(),
            context: Context::World,
            valid: TimeBounds {
                lo: Some(3),
                hi: None,
            },
            status: Status::Settled,
            provenance: Provenance::Author {
                action: "typed".into(),
            },
            confidence: 1.0,
            anchor: Some(anchor()),
            settled_at: Some(SettledAt {
                actor: "author".into(),
                at: "2026-01-01T00:00:00Z".into(),
            }),
        });
        assert_fits(&mut failures, &with_ir, "StoryIr", "StoryIr", &ir);
        assert!(
            with_ir.contains_key("ir.Fact") && with_ir.contains_key("ir.Value"),
            "the IR types carry their crate prefix"
        );
        assert!(
            !components.contains_key("i64"),
            "Tick fields must not register an i64 component"
        );

        assert!(
            failures.is_empty(),
            "schemas rejected values:\n  {}",
            failures.join("\n  ")
        );
    }

    // ───────────── the export and the route ─────────────

    /// With `ASTEN_OPENAPI_OUT` set, writes the document there (what
    /// `scripts/openapi-export.mjs` does); otherwise holds `docs/openapi.json`
    /// to the generated document so a route or schema change cannot leave the
    /// committed copy behind.
    #[test]
    fn the_committed_document_is_current() {
        let generated = format!("{}\n", document_json());
        if let Ok(target) = std::env::var("ASTEN_OPENAPI_OUT") {
            std::fs::write(&target, &generated).expect("the document is written");
            return;
        }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/openapi.json");
        let committed = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            committed == generated,
            "docs/openapi.json is out of date; run `node scripts/openapi-export.mjs`"
        );
    }

    #[tokio::test]
    async fn the_document_is_served_without_a_token() {
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let repository = asten_storage::SqliteRepository::in_memory()
            .await
            .expect("an in-memory store");
        let repository: Arc<dyn asten_application::Repository> = Arc::new(repository);
        let application = asten_application::Application::with_auth(
            repository,
            asten_application::AuthMode::Pairing,
        );
        let app = crate::router(application, crate::ApiConfig::default());

        let response = app
            .clone()
            .oneshot(
                Request::get("/api/v1/openapi.json")
                    .body(Body::empty())
                    .expect("a request"),
            )
            .await
            .expect("a response");
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        assert!(
            content_type.starts_with("application/json"),
            "{content_type}"
        );
        let body = response
            .into_body()
            .collect()
            .await
            .expect("a body")
            .to_bytes();
        let document: Value = serde_json::from_slice(&body).expect("JSON");
        assert!(
            document["openapi"]
                .as_str()
                .is_some_and(|v| v.starts_with("3.1"))
        );
        assert!(document["paths"]["/api/v1/projects"]["get"].is_object());

        // The rest of the API still asks for a token.
        let response = app
            .oneshot(
                Request::get("/api/v1/projects")
                    .body(Body::empty())
                    .expect("a request"),
            )
            .await
            .expect("a response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
