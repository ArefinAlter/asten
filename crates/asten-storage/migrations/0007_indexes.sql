-- Indexes for the per-document and per-project lists the desk and the engine
-- read on every change. Each one backs a `WHERE document_id = ?` or
-- `WHERE project_id = ?` with the `ORDER BY created_at` the query uses.
--
-- Already covered and therefore not repeated here: block_events(document_id,
-- seq) and document_ops(document_id, seq) by their primary/unique keys,
-- block_snapshots(document_id, seq) by its unique key, auth_tokens(hash) by
-- its unique constraint, documents(project_id, parent_id, position),
-- facts(project_id, entity_key, predicate), entities(project_id, kind, name),
-- timeline_events(project_id, reading_order), world_entries(project_id, ...).

CREATE INDEX IF NOT EXISTS comments_document ON comments(document_id, created_at);
CREATE INDEX IF NOT EXISTS drafts_document ON drafts(document_id, created_at);
CREATE INDEX IF NOT EXISTS document_snapshots_document ON document_snapshots(document_id, created_at);
CREATE INDEX IF NOT EXISTS tracked_changes_document ON tracked_changes(document_id, created_at);
CREATE INDEX IF NOT EXISTS suppressions_project ON suppressions(project_id, created_at);
CREATE INDEX IF NOT EXISTS invariants_project ON invariants(project_id, created_at);
CREATE INDEX IF NOT EXISTS routes_project ON routes(project_id, created_at);
CREATE INDEX IF NOT EXISTS baselines_project ON baselines(project_id, created_at);
