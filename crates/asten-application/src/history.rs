//! Document history: committing content as hash-chained block events, keeping
//! anchors attached through edits, snapshots, verification, blame, and drafts
//! merged three-way.

use std::collections::BTreeMap;

pub use asten_blocks::{BlameSpan, EventRecord};
use asten_blocks::{
    Conflict, DocState, EventPayload, Mapped, apply, blocks_of, diff_docs_with_state,
    doc_from_state, map_span, merkle_root, next_record_after, patch, reattach, state_at, three_way,
    verify,
};
use asten_diagnostics::Diagnostic;
use asten_domain::{BlockSnapshot, Document, DocumentId, Draft, DraftId, SnapshotId};
use asten_ir::{Anchor, AnchorRevision};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{Application, ApplicationError, Result};

const SNAPSHOT_EVERY: i64 = 500;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateDraft {
    pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateDraft {
    pub name: Option<String>,
    pub content_json: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MergeRequest {
    /// `"{node_id}:{field}"` → `"ours"` | `"theirs"`.
    #[serde(default)]
    pub resolutions: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MergePreview {
    pub draft_id: DraftId,
    pub merged_content: serde_json::Value,
    pub conflicts: Vec<Conflict>,
    pub ops: usize,
    /// Marks that would exist after the merge and do not exist now.
    pub new_marks: Vec<Diagnostic>,
    /// Marks that exist now and would be gone after the merge.
    pub resolved_marks: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VerifyReport {
    pub document_id: DocumentId,
    pub events: usize,
    pub head_seq: i64,
    pub chain_hash: String,
    pub merkle_root: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Application {
    /// Diff the new content against the materialised state, append the events,
    /// store the new state, snapshot when due, and keep anchors attached.
    pub(crate) async fn commit_content(
        &self,
        mut document: Document,
        content: serde_json::Value,
        actor: &str,
    ) -> Result<Document> {
        let old_state = self
            .repository
            .document_state(document.id)
            .await?
            .unwrap_or_default();
        let ops = diff_docs_with_state(&old_state, &content);
        if ops.is_empty() {
            if document.revision == 0 {
                // A brand-new empty document still needs its row filled in.
                document.content_json = content;
                document.revision = 1;
                document.updated_at = Utc::now();
                self.repository.update_document(&document).await?;
            }
            return Ok(document);
        }
        let now = Utc::now();
        let at = now.to_rfc3339();
        let previous_head = document.head_seq;
        let mut head_seq = document.head_seq;
        let mut chain = if document.chain_hash.is_empty() {
            asten_blocks::ZERO_HASH_HEX.to_owned()
        } else {
            document.chain_hash.clone()
        };
        let mut records = Vec::with_capacity(ops.len());
        for op in ops {
            let record = next_record_after(head_seq, &chain, op, actor, &at);
            head_seq = record.seq;
            chain = record.chain_hash.clone();
            records.push(record);
        }
        let mut new_state = old_state.clone();
        for record in &records {
            apply(&mut new_state, &record.payload)
                .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        }
        document.content_json = doc_from_state(&new_state);
        document.head_seq = head_seq;
        document.chain_hash = chain.clone();
        document.revision += 1;
        document.updated_at = now;
        self.repository
            .commit_document(&document, &new_state, &records)
            .await?;
        if head_seq / SNAPSHOT_EVERY > previous_head / SNAPSHOT_EVERY {
            let snapshot = BlockSnapshot {
                id: SnapshotId::new(),
                document_id: document.id,
                seq: head_seq,
                chain_hash: chain,
                merkle_root: merkle_root(&new_state),
                state_json: serde_json::to_value(&new_state).unwrap_or_default(),
                created_at: now,
            };
            self.repository.insert_block_snapshot(&snapshot).await?;
        }
        self.remap_anchors(&document, &new_state, &records).await?;
        Ok(document)
    }

    /// Keep facts, events and comments anchored through the edit.
    async fn remap_anchors(
        &self,
        document: &Document,
        new_state: &DocState,
        records: &[EventRecord],
    ) -> Result<()> {
        let doc_id = asten_ir::DocId::new(document.id.to_string());
        let facts = self.repository.list_facts(document.project_id).await?;
        for mut fact in facts {
            let Some(anchor) = fact.anchor.clone() else {
                continue;
            };
            if anchor.document != doc_id {
                continue;
            }
            if let Some(mapped) = map_anchor(&anchor, records, new_state, document.head_seq) {
                fact.anchor = Some(mapped);
                fact.updated_at = Utc::now();
                self.repository.update_fact(&fact).await?;
            }
        }
        let events = self.repository.list_events(document.project_id).await?;
        for mut event in events {
            let Some(anchor) = event.anchor.clone() else {
                continue;
            };
            if anchor.document != doc_id {
                continue;
            }
            if let Some(mapped) = map_anchor(&anchor, records, new_state, document.head_seq) {
                event.anchor = Some(mapped);
                event.updated_at = Utc::now();
                self.repository.update_event(&event).await?;
            }
        }
        let comments = self.repository.list_comments(document.id).await?;
        for mut comment in comments {
            let Some(block) = comment.block_id.clone() else {
                continue;
            };
            let Some(ops) = root_ops_for(records, &block) else {
                continue;
            };
            let start = u32::try_from(comment.from_offset.max(0)).unwrap_or(0);
            let end = u32::try_from(comment.to_offset.max(0)).unwrap_or(0);
            if let Mapped::Moved { start: s, end: e } =
                map_span(ops, start, end.max(start.saturating_add(1)))
            {
                if i64::from(s) != comment.from_offset || i64::from(e) != comment.to_offset {
                    comment.from_offset = i64::from(s);
                    comment.to_offset = i64::from(e);
                    comment.updated_at = Utc::now();
                    self.repository.update_comment(&comment).await?;
                }
            }
        }
        Ok(())
    }

    /// The document's full hash-chained history.
    pub async fn document_history(&self, id: DocumentId) -> Result<Vec<EventRecord>> {
        self.get_document(id).await?;
        self.repository.list_block_events(id, 0).await
    }

    /// Verify the chain and the state against the stored head.
    pub async fn verify_document(&self, id: DocumentId) -> Result<VerifyReport> {
        let document = self.get_document(id).await?;
        let records = self.repository.list_block_events(id, 0).await?;
        let state = self
            .repository
            .document_state(id)
            .await?
            .unwrap_or_default();
        let mut report = VerifyReport {
            document_id: id,
            events: records.len(),
            head_seq: document.head_seq,
            chain_hash: document.chain_hash.clone(),
            merkle_root: merkle_root(&state),
            ok: true,
            error: None,
        };
        if let Err(error) = verify(&records) {
            report.ok = false;
            report.error = Some(error.to_string());
            return Ok(report);
        }
        if let Some(last) = records.last() {
            if last.seq != document.head_seq || last.chain_hash != document.chain_hash {
                report.ok = false;
                report.error = Some("the stored head does not match the last event".into());
                return Ok(report);
            }
        }
        // Replaying the log must reproduce the stored state.
        match asten_blocks::fold(records.iter().map(|record| &record.payload)) {
            Ok(replayed) if replayed.nodes == state.nodes => {}
            Ok(_) => {
                report.ok = false;
                report.error = Some("replaying the log does not reproduce the stored state".into());
            }
            Err(error) => {
                report.ok = false;
                report.error = Some(error.to_string());
            }
        }
        Ok(report)
    }

    /// The document as it was at an event sequence.
    pub async fn document_at(&self, id: DocumentId, seq: i64) -> Result<serde_json::Value> {
        self.get_document(id).await?;
        let state = self.state_at(id, seq).await?;
        Ok(doc_from_state(&state))
    }

    pub(crate) async fn state_at(&self, id: DocumentId, seq: i64) -> Result<DocState> {
        // Start from the latest snapshot at or before `seq`, then replay the tail.
        if let Some(snapshot) = self.repository.latest_block_snapshot(id, Some(seq)).await? {
            let mut state: DocState = serde_json::from_value(snapshot.state_json)
                .map_err(|error| ApplicationError::Storage(error.to_string()))?;
            let tail = self
                .repository
                .list_block_events(id, snapshot.seq + 1)
                .await?;
            for record in tail.iter().filter(|record| record.seq <= seq) {
                apply(&mut state, &record.payload)
                    .map_err(|error| ApplicationError::Storage(error.to_string()))?;
            }
            return Ok(state);
        }
        let records = self.repository.list_block_events(id, 0).await?;
        state_at(&records, seq).map_err(|error| ApplicationError::Storage(error.to_string()))
    }

    /// Who last wrote each range of each block.
    pub async fn document_blame(&self, id: DocumentId) -> Result<BTreeMap<String, Vec<BlameSpan>>> {
        self.get_document(id).await?;
        let records = self.repository.list_block_events(id, 0).await?;
        Ok(asten_blocks::blame(&records))
    }

    // ───────────────────────── drafts ─────────────────────────

    pub async fn list_drafts(&self, document_id: DocumentId) -> Result<Vec<Draft>> {
        self.get_document(document_id).await?;
        self.repository.list_drafts(document_id).await
    }

    pub async fn get_draft(&self, id: DraftId) -> Result<Draft> {
        self.repository
            .find_draft(id)
            .await?
            .ok_or(ApplicationError::NotFound { entity: "draft" })
    }

    pub async fn create_draft(&self, document_id: DocumentId, input: CreateDraft) -> Result<Draft> {
        let document = self.get_document(document_id).await?;
        let name = input.name.trim();
        if name.is_empty() {
            return Err(ApplicationError::Validation("a draft needs a name".into()));
        }
        let state = self
            .repository
            .document_state(document_id)
            .await?
            .unwrap_or_default();
        let now = Utc::now();
        let draft = Draft {
            id: DraftId::new(),
            document_id,
            name: name.to_owned(),
            base_seq: document.head_seq,
            base_state_json: serde_json::to_value(&state).unwrap_or_default(),
            content_json: document.content_json.clone(),
            created_at: now,
            updated_at: now,
            merged_at: None,
        };
        self.repository.insert_draft(&draft).await?;
        Ok(draft)
    }

    pub async fn update_draft(&self, id: DraftId, input: UpdateDraft) -> Result<Draft> {
        let mut draft = self.get_draft(id).await?;
        if draft.merged_at.is_some() {
            return Err(ApplicationError::Conflict(
                "this draft was already merged".into(),
            ));
        }
        if let Some(name) = input.name {
            let name = name.trim();
            if !name.is_empty() {
                draft.name = name.to_owned();
            }
        }
        if let Some(content) = input.content_json {
            draft.content_json = asten_domain::ensure_block_ids(content);
        }
        draft.updated_at = Utc::now();
        self.repository.update_draft(&draft).await?;
        Ok(draft)
    }

    pub async fn delete_draft(&self, id: DraftId) -> Result<()> {
        self.get_draft(id).await?;
        self.repository.delete_draft(id).await
    }

    /// Preview the three-way merge and the continuity marks it would introduce.
    pub async fn merge_preview(&self, id: DraftId, request: &MergeRequest) -> Result<MergePreview> {
        let draft = self.get_draft(id).await?;
        let document = self.get_document(draft.document_id).await?;
        let base: DocState =
            serde_json::from_value(draft.base_state_json.clone()).unwrap_or_default();
        let ours = self
            .repository
            .document_state(document.id)
            .await?
            .unwrap_or_default();
        let mut theirs = base.clone();
        for op in diff_docs_with_state(&base, &draft.content_json) {
            apply(&mut theirs, &op)
                .map_err(|error| ApplicationError::Storage(error.to_string()))?;
        }
        let result = three_way(&base, &ours, &theirs, &request.resolutions);
        let merged_content = doc_from_state(&result.state);
        // Semantic merge conflicts: marks after the merge that exist in neither side.
        let current = self.analyze_project(document.project_id).await?;
        let after = self
            .analyze_with_override(document.project_id, document.id, merged_content.clone())
            .await?;
        let current_ids: Vec<&str> = current
            .diagnostics
            .iter()
            .map(|mark| mark.id.as_str())
            .collect();
        let after_ids: Vec<&str> = after
            .diagnostics
            .iter()
            .map(|mark| mark.id.as_str())
            .collect();
        let new_marks = after
            .diagnostics
            .iter()
            .filter(|mark| !current_ids.contains(&mark.id.as_str()))
            .cloned()
            .collect();
        let resolved_marks = current
            .diagnostics
            .iter()
            .filter(|mark| !after_ids.contains(&mark.id.as_str()))
            .map(|mark| mark.id.clone())
            .collect();
        Ok(MergePreview {
            draft_id: id,
            merged_content,
            conflicts: result.conflicts,
            ops: result.ops.len(),
            new_marks,
            resolved_marks,
        })
    }

    /// Merge the draft into its document. Refuses while conflicts are unresolved.
    pub async fn merge_draft(&self, id: DraftId, request: &MergeRequest) -> Result<Document> {
        let preview = self.merge_preview(id, request).await?;
        if !preview.conflicts.is_empty() {
            return Err(ApplicationError::Conflict(format!(
                "{} conflict(s) need a resolution",
                preview.conflicts.len()
            )));
        }
        let mut draft = self.get_draft(id).await?;
        let document = self
            .update_document(
                draft.document_id,
                crate::UpdateDocument {
                    content_json: Some(preview.merged_content),
                    actor: Some(format!("merge:{}", draft.name)),
                    ..Default::default()
                },
            )
            .await?;
        draft.merged_at = Some(Utc::now());
        draft.updated_at = Utc::now();
        self.repository.update_draft(&draft).await?;
        Ok(document)
    }
}

/// Text ops that touched a block's root in this commit, if any.
fn root_ops_for<'a>(records: &'a [EventRecord], block: &str) -> Option<&'a [patch::TextOp]> {
    records.iter().find_map(|record| match &record.payload {
        EventPayload::RichTextPatched { node_id, patch } if node_id == block => {
            patch::root_text_ops(patch)
        }
        _ => None,
    })
}

/// Map an anchor through a commit. Returns the updated anchor when it changed.
fn map_anchor(
    anchor: &Anchor,
    records: &[EventRecord],
    new_state: &DocState,
    head_seq: i64,
) -> Option<Anchor> {
    let block = anchor.block.0.as_str();
    let touched = records
        .iter()
        .any(|record| record.payload.target_node_id() == Some(block));
    if !touched {
        return None;
    }
    let Some(node) = new_state.nodes.get(block) else {
        return Some(Anchor {
            stale: true,
            ..anchor.clone()
        });
    };
    if node.deleted {
        return Some(Anchor {
            stale: true,
            ..anchor.clone()
        });
    }
    let new_text = node.label().to_owned();
    let fingerprint = node
        .content()
        .map(asten_blocks::fingerprint)
        .unwrap_or_default();
    let mut mapped: Option<(u32, u32)> = None;
    if let Some(ops) = root_ops_for(records, block) {
        if let Mapped::Moved { start, end } = map_span(ops, anchor.start, anchor.end) {
            mapped = Some((start, end));
        }
    }
    let replaced = records.iter().any(|record| match &record.payload {
        EventPayload::RichTextPatched { node_id, patch } if node_id == block => {
            patch::replaces_root(patch)
        }
        EventPayload::FieldEdited { node_id, field, .. }
            if node_id == block && field == "content" =>
        {
            true
        }
        _ => false,
    });
    if mapped.is_none() || replaced {
        mapped = reattach(
            &anchor.quote,
            &anchor.context_before,
            &anchor.context_after,
            &new_text,
        );
    }
    match mapped {
        Some((start, end)) => {
            let mut next = Anchor::new(anchor.document.clone(), anchor.block.clone(), start, end)
                .with_text(&new_text);
            next.revision = Some(AnchorRevision {
                seq: head_seq,
                fingerprint,
            });
            next.stale = false;
            Some(next)
        }
        None => Some(Anchor {
            stale: true,
            ..anchor.clone()
        }),
    }
}

/// Plain text of every block in a document, by block id.
#[must_use]
pub fn block_texts(content: &serde_json::Value) -> BTreeMap<String, String> {
    blocks_of(content)
        .into_iter()
        .map(|block| (block.id, block.text))
        .collect()
}
