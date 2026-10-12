use std::sync::Arc;

use asten_application::{
    Application, CreateCanonEntity, CreateDocument, CreateDraft, CreateFact, CreateProject,
    CreateTimelineEvent, CreateWorldEntry, MergeRequest, UpdateDocument, UpdateDraft,
    UpdateWorldEntry,
};
use asten_domain::{DocumentKind, EntityKind, ProjectKind, ScenePlacement, WorldKind, WorldStatus};
use asten_ir::{Anchor, EventKind};
use asten_storage::SqliteRepository;
use serde_json::json;
use uuid::Uuid;

fn paragraph(id: &str, text: &str) -> serde_json::Value {
    json!({"type":"paragraph","attrs":{"id":id},"content":[{"type":"text","text":text}]})
}

fn doc(blocks: Vec<serde_json::Value>) -> serde_json::Value {
    json!({"type":"doc","content":blocks})
}

async fn open() -> (Application, std::path::PathBuf) {
    let path = std::env::temp_dir().join(format!("asten-test-{}.db", Uuid::now_v7()));
    let repository = SqliteRepository::open(&path).await.unwrap();
    (
        Application::with_auth(Arc::new(repository), asten_application::AuthMode::Pairing),
        path,
    )
}

fn cleanup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("db-shm"));
    let _ = std::fs::remove_file(path.with_extension("db-wal"));
}

#[tokio::test]
async fn documents_keep_a_hash_chained_history_and_anchors_follow_edits() {
    let (application, path) = open().await;
    let project = application
        .create_project(CreateProject {
            name: "A test novel".into(),
            kind: ProjectKind::Novel,
            description: None,
        })
        .await
        .unwrap();
    let chapter = application
        .create_document(CreateDocument {
            project_id: project.id,
            parent_id: None,
            kind: DocumentKind::Chapter,
            title: "Chapter 2".into(),
            content_json: doc(vec![paragraph(
                "b1",
                "Mira tied back her auburn hair and waited for Oren.",
            )]),
            position: 0,
            scene: Some(ScenePlacement {
                story_time: Some("day 3".into()),
                ..ScenePlacement::default()
            }),
        })
        .await
        .unwrap();
    assert_eq!(chapter.head_seq, 1, "genesis event for the first block");
    let history = application.document_history(chapter.id).await.unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].prev_chain_hash, asten_blocks::ZERO_HASH_HEX);

    // A fact anchored on "auburn" (scalars 19..25).
    let fact = application
        .create_fact(CreateFact {
            project_id: project.id,
            entity_key: "Mira".into(),
            predicate: "hair_color".into(),
            value: "auburn".into(),
            modality: Default::default(),
            status: Default::default(),
            valid_from: Some("day 3".into()),
            valid_to: None,
            source: None,
            context: None,
            anchor: Some(
                Anchor::new(chapter.id.to_string(), "b1", 19, 25)
                    .with_text("Mira tied back her auburn hair and waited for Oren."),
            ),
            confidence: None,
            provenance: None,
        })
        .await
        .unwrap();
    assert_eq!(fact.anchor.as_ref().unwrap().quote, "auburn");

    // Edit before the anchor: it moves; the chain grows and verifies.
    let edited = application
        .update_document(
            chapter.id,
            UpdateDocument {
                content_json: Some(doc(vec![paragraph(
                    "b1",
                    "Young Mira tied back her auburn hair and waited for Oren.",
                )])),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(edited.head_seq, 2);
    let fact = application.get_fact(fact.id).await.unwrap();
    let anchor = fact.anchor.unwrap();
    assert_eq!((anchor.start, anchor.end), (25, 31));
    assert_eq!(anchor.quote, "auburn");
    assert!(!anchor.stale);
    let report = application.verify_document(chapter.id).await.unwrap();
    assert!(report.ok, "{report:?}");

    // Replace the whole paragraph: the anchor re-attaches by its quote.
    application
        .update_document(
            chapter.id,
            UpdateDocument {
                content_json: Some(doc(vec![paragraph(
                    "b1",
                    "The auburn braid was Mira's one vanity.",
                )])),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let fact = application.get_fact(fact.id).await.unwrap();
    let anchor = fact.anchor.unwrap();
    assert_eq!(anchor.quote, "auburn");
    assert_eq!(anchor.start, 4);

    // History can be materialised at any sequence.
    let at_one = application.document_at(chapter.id, 1).await.unwrap();
    assert!(asten_blocks::plain_text(&at_one).starts_with("Mira tied back"));
    let blame = application.document_blame(chapter.id).await.unwrap();
    assert!(blame.contains_key("b1"));
    cleanup(&path);
}

#[tokio::test]
async fn the_engine_marks_a_contradiction_and_repairs_clear_it() {
    let (application, path) = open().await;
    let project = application
        .create_project(CreateProject {
            name: "Marks".into(),
            kind: ProjectKind::Novel,
            description: None,
        })
        .await
        .unwrap();
    application
        .create_entity(CreateCanonEntity {
            project_id: project.id,
            kind: EntityKind::Character,
            name: "Mira".into(),
            aliases: Vec::new(),
            notes: String::new(),
            introduced_at: None,
        })
        .await
        .unwrap();
    let ch2 = application
        .create_document(CreateDocument {
            project_id: project.id,
            parent_id: None,
            kind: DocumentKind::Chapter,
            title: "Chapter 2".into(),
            content_json: doc(vec![paragraph("b1", "Mira tied back her auburn hair.")]),
            position: 0,
            scene: Some(ScenePlacement {
                story_time: Some("day 3".into()),
                ..ScenePlacement::default()
            }),
        })
        .await
        .unwrap();
    let ch19 = application
        .create_document(CreateDocument {
            project_id: project.id,
            parent_id: None,
            kind: DocumentKind::Chapter,
            title: "Chapter 19".into(),
            content_json: doc(vec![paragraph(
                "b9",
                "Mira pushed her black hair out of her eyes.",
            )]),
            position: 1,
            scene: Some(ScenePlacement {
                story_time: Some("day 22".into()),
                ..ScenePlacement::default()
            }),
        })
        .await
        .unwrap();
    application
        .create_fact(CreateFact {
            project_id: project.id,
            entity_key: "Mira".into(),
            predicate: "hair_color".into(),
            value: "auburn".into(),
            modality: Default::default(),
            status: Default::default(),
            valid_from: Some("day 3".into()),
            valid_to: None,
            source: None,
            context: None,
            anchor: Some(
                Anchor::new(ch2.id.to_string(), "b1", 19, 25)
                    .with_text("Mira tied back her auburn hair."),
            ),
            confidence: None,
            provenance: None,
        })
        .await
        .unwrap();
    let snapshot = application.analyze_project(project.id).await.unwrap();
    let mark = snapshot
        .diagnostics
        .iter()
        .find(|mark| mark.code == "CANON-E013")
        .expect("an attribute mark");
    assert_eq!(
        mark.primary.as_ref().unwrap().document.0,
        ch19.id.to_string()
    );
    assert_eq!(mark.primary.as_ref().unwrap().quote, "black");
    assert!(mark.message.contains("Chapter 2"));

    // Hover: the fact card for Mira at chapter 19.
    let card = application
        .fact_card(project.id, "mira", Some(ch19.id))
        .await
        .unwrap();
    assert_eq!(card.facts.len(), 1);
    assert_eq!(card.mention_count, 2);

    // Repair: add the change event; the mark goes away.
    let add_event = mark
        .repairs
        .iter()
        .position(|repair| repair.label == "Add an event")
        .unwrap();
    let response = application
        .apply_repair(
            project.id,
            asten_ls::ApplyRepairRequest {
                diagnostic_id: mark.id.clone(),
                repair_index: add_event,
                reason: String::new(),
                scope: None,
            },
        )
        .await
        .unwrap();
    assert!(response.message.contains("change event"));
    let after = application.analyze_project(project.id).await.unwrap();
    assert!(
        after
            .diagnostics
            .iter()
            .all(|mark| mark.code != "CANON-E013"),
        "{:?}",
        after.diagnostics
    );

    // A flashback: Oren dies on day 14; a scene placed on day 5 mentions him; no mark.
    application
        .create_entity(CreateCanonEntity {
            project_id: project.id,
            kind: EntityKind::Character,
            name: "Captain Oren".into(),
            aliases: vec!["Oren".into()],
            notes: String::new(),
            introduced_at: None,
        })
        .await
        .unwrap();
    let oren = application
        .list_entities(project.id)
        .await
        .unwrap()
        .into_iter()
        .find(|entity| entity.name == "Captain Oren")
        .unwrap();
    application
        .create_timeline_event_for_test(project.id, oren.id, "Oren dies", "day 14")
        .await;
    application
        .create_document(CreateDocument {
            project_id: project.id,
            parent_id: None,
            kind: DocumentKind::Scene,
            title: "Memory of the harbor".into(),
            content_json: doc(vec![paragraph(
                "m1",
                "Oren poured the tea and said nothing.",
            )]),
            position: 2,
            scene: Some(ScenePlacement {
                story_time: Some("day 5".into()),
                ..ScenePlacement::default()
            }),
        })
        .await
        .unwrap();
    let after = application.analyze_project(project.id).await.unwrap();
    assert!(
        after
            .diagnostics
            .iter()
            .all(|mark| mark.code != "CANON-E101"),
        "{:?}",
        after.diagnostics
    );
    // The same scene on day 20 is a must violation.
    let memory = application
        .list_documents(project.id)
        .await
        .unwrap()
        .into_iter()
        .find(|document| document.title == "Memory of the harbor")
        .unwrap();
    application
        .update_document(
            memory.id,
            UpdateDocument {
                scene: Some(ScenePlacement {
                    story_time: Some("day 20".into()),
                    ..ScenePlacement::default()
                }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let after = application.analyze_project(project.id).await.unwrap();
    let e101 = after
        .diagnostics
        .iter()
        .find(|mark| mark.code == "CANON-E101")
        .expect("lifecycle mark");
    assert_eq!(e101.tier, asten_diagnostics::Tier::Must);
    cleanup(&path);
}

#[tokio::test]
async fn drafts_merge_three_way_and_world_entries_mint_facts() {
    let (application, path) = open().await;
    let project = application
        .create_project(CreateProject {
            name: "Drafts".into(),
            kind: ProjectKind::Novel,
            description: None,
        })
        .await
        .unwrap();
    let document = application
        .create_document(CreateDocument {
            project_id: project.id,
            parent_id: None,
            kind: DocumentKind::Scene,
            title: "Harbor".into(),
            content_json: doc(vec![
                paragraph("a", "Mira pushed her hair back and waited."),
                paragraph("b", "The tide came in."),
            ]),
            position: 0,
            scene: None,
        })
        .await
        .unwrap();
    let draft = application
        .create_draft(
            document.id,
            CreateDraft {
                name: "Try a stronger ending".into(),
            },
        )
        .await
        .unwrap();
    // Main edits block a; the draft edits block b.
    application
        .update_document(
            document.id,
            UpdateDocument {
                content_json: Some(doc(vec![
                    paragraph("a", "Mira pushed her auburn hair back and waited."),
                    paragraph("b", "The tide came in."),
                ])),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    application
        .update_draft(
            draft.id,
            UpdateDraft {
                name: None,
                content_json: Some(doc(vec![
                    paragraph("a", "Mira pushed her hair back and waited."),
                    paragraph("b", "The tide came in, black and quick."),
                ])),
            },
        )
        .await
        .unwrap();
    let preview = application
        .merge_preview(draft.id, &MergeRequest::default())
        .await
        .unwrap();
    assert!(preview.conflicts.is_empty(), "{:?}", preview.conflicts);
    assert_eq!(
        asten_blocks::plain_text(&preview.merged_content),
        "Mira pushed her auburn hair back and waited.\nThe tide came in, black and quick."
    );
    let merged = application
        .merge_draft(draft.id, &MergeRequest::default())
        .await
        .unwrap();
    assert!(asten_blocks::plain_text(&merged.content_json).contains("black and quick"));
    assert!(
        application
            .get_draft(draft.id)
            .await
            .unwrap()
            .merged_at
            .is_some()
    );

    let place = application
        .create_world_entry(CreateWorldEntry {
            project_id: project.id,
            kind: WorldKind::Place,
            title: "Dusk Harbor".into(),
            summary: "Appears only after the bells.".into(),
            prompt: None,
            status: Some(WorldStatus::Spark),
            board_x: None,
            board_y: None,
            map_x: None,
            map_y: None,
            hue: None,
        })
        .await
        .unwrap();
    application
        .update_world_entry(
            place.id,
            UpdateWorldEntry {
                status: Some(WorldStatus::Settled),
                map_x: Some(Some(42.0)),
                map_y: Some(Some(61.0)),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let pulse = application.world_pulse(project.id).await.unwrap();
    assert_eq!(pulse.places, 1);
    let facts = application.list_facts(project.id).await.unwrap();
    assert!(
        facts
            .iter()
            .any(|fact| fact.entity_key == "dusk_harbor" && fact.predicate == "place")
    );
    let compiled = application
        .compile_project(
            project.id,
            asten_application::CompileRequest {
                format: asten_application::CompileFormat::Markdown,
                document_ids: Vec::new(),
            },
        )
        .await
        .unwrap();
    assert!(compiled.body.contains("Harbor"));
    cleanup(&path);
}

#[tokio::test]
async fn pairing_issues_tokens_and_rejects_bad_codes() {
    let (application, path) = open().await;
    let status = application.auth_status().await.unwrap();
    assert!(status.requires_auth);
    assert_eq!(status.paired_clients, 0);
    let start = application
        .start_pairing("Word on this laptop", asten_domain::TokenKind::Guest)
        .await
        .unwrap();
    assert_eq!(start.code.len(), 6);
    assert!(
        application
            .complete_pairing("000000", "", "client-a")
            .await
            .is_err()
            || start.code == "000000"
    );
    let paired = application
        .complete_pairing(&start.code, "", "client-a")
        .await
        .unwrap();
    assert!(paired.token.starts_with("ast_"));
    assert_eq!(paired.label, "Word on this laptop");
    // The code is single use.
    assert!(
        application
            .complete_pairing(&start.code, "", "client-a")
            .await
            .is_err()
    );
    let client = application
        .authenticate(Some(&paired.token))
        .await
        .unwrap()
        .expect("authenticated");
    assert_eq!(client.kind, asten_domain::TokenKind::Guest);
    assert!(
        application
            .authenticate(Some("ast_nope"))
            .await
            .unwrap()
            .is_none()
    );
    application.revoke_token(paired.token_id).await.unwrap();
    assert!(
        application
            .authenticate(Some(&paired.token))
            .await
            .unwrap()
            .is_none()
    );

    application
        .setup_owner("a very long passphrase", "client-b")
        .await
        .unwrap();
    // Once means once: a second setup is a conflict, not a silent overwrite.
    assert!(matches!(
        application
            .setup_owner("another long passphrase", "client-b")
            .await,
        Err(asten_application::ApplicationError::Conflict(_))
    ));
    // Desk codes grant desk tokens; guests cannot administer.
    let desk_code = application
        .start_pairing("Second desk", asten_domain::TokenKind::Desk)
        .await
        .unwrap();
    let desk = application
        .complete_pairing(&desk_code.code, "", "client-c")
        .await
        .unwrap();
    assert_eq!(desk.kind, asten_domain::TokenKind::Desk);
    assert!(desk.kind.can_administer());
    assert!(!asten_domain::TokenKind::Guest.can_administer());
    // Stream tickets are single use and bound to their project.
    let project = application
        .create_project(asten_application::CreateProject {
            name: "Tickets".into(),
            kind: asten_domain::ProjectKind::Novel,
            description: None,
        })
        .await
        .unwrap();
    let who = application
        .authenticate(Some(&desk.token))
        .await
        .unwrap()
        .unwrap();
    let ticket = application
        .issue_stream_ticket(project.id, &who)
        .await
        .unwrap();
    assert!(
        application
            .redeem_stream_ticket(&ticket.ticket, asten_domain::ProjectId::new())
            .is_none()
    );
    let ticket = application
        .issue_stream_ticket(project.id, &who)
        .await
        .unwrap();
    assert!(
        application
            .redeem_stream_ticket(&ticket.ticket, project.id)
            .is_some()
    );
    assert!(
        application
            .redeem_stream_ticket(&ticket.ticket, project.id)
            .is_none()
    );
    assert!(
        application
            .login_owner("wrong passphrase", "client-b")
            .await
            .is_err()
    );
    let session = application
        .login_owner("a very long passphrase", "client-b")
        .await
        .unwrap();
    assert_eq!(session.kind, asten_domain::TokenKind::Session);
    assert!(session.expires_at.is_some());
    cleanup(&path);
}

trait TestHelpers {
    async fn create_timeline_event_for_test(
        &self,
        project: asten_domain::ProjectId,
        entity: asten_domain::EntityId,
        title: &str,
        when: &str,
    );
}

impl TestHelpers for Application {
    async fn create_timeline_event_for_test(
        &self,
        project: asten_domain::ProjectId,
        entity: asten_domain::EntityId,
        title: &str,
        when: &str,
    ) {
        self.create_event(CreateTimelineEvent {
            project_id: project,
            title: title.into(),
            entity_id: Some(entity),
            document_id: None,
            story_time: Some(when.into()),
            reading_order: None,
            duration: None,
            notes: String::new(),
            kind: Some(EventKind::Dies),
            participants: Vec::new(),
            context: None,
            status: None,
            tick: None,
            tick_lo: None,
            tick_hi: None,
            anchor: None,
        })
        .await
        .unwrap();
    }
}
