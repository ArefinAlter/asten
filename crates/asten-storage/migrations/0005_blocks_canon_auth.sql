PRAGMA foreign_keys = ON;

-- Documents: scene placement, the materialised block state, and the head of the
-- hash-chained event log.
ALTER TABLE documents ADD COLUMN scene_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE documents ADD COLUMN state_json TEXT NOT NULL DEFAULT '';
ALTER TABLE documents ADD COLUMN head_seq INTEGER NOT NULL DEFAULT 0;
ALTER TABLE documents ADD COLUMN chain_hash TEXT NOT NULL DEFAULT '0000000000000000000000000000000000000000000000000000000000000000';

-- Facts: a wider status set and the engine's context, confidence, anchor and provenance.
CREATE TABLE facts_v2 (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    entity_key TEXT NOT NULL,
    predicate TEXT NOT NULL,
    value TEXT NOT NULL,
    modality TEXT NOT NULL CHECK (
        modality IN ('established', 'inferred', 'contested', 'author_override')
    ),
    status TEXT NOT NULL CHECK (status IN ('proposed', 'confirmed', 'deprecated', 'rejected')),
    valid_from TEXT,
    valid_to TEXT,
    source_document_id TEXT REFERENCES documents(id) ON DELETE SET NULL,
    source_start INTEGER,
    source_end INTEGER,
    context_json TEXT NOT NULL DEFAULT '{"kind":"world"}',
    confidence REAL NOT NULL DEFAULT 1.0,
    anchor_json TEXT,
    provenance_json TEXT NOT NULL DEFAULT '{"kind":"author","action":""}',
    settled_by TEXT,
    settled_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

INSERT INTO facts_v2 (
    id, project_id, entity_key, predicate, value, modality, status, valid_from, valid_to,
    source_document_id, source_start, source_end, created_at, updated_at
)
SELECT
    id, project_id, entity_key, predicate, value, modality, status, valid_from, valid_to,
    source_document_id, source_start, source_end, created_at, updated_at
FROM facts;

DROP TABLE facts;
ALTER TABLE facts_v2 RENAME TO facts;
CREATE INDEX facts_project_entity ON facts(project_id, entity_key, predicate);

-- Entities: where they are introduced.
ALTER TABLE entities ADD COLUMN introduced_at INTEGER;

-- Timeline events: typed kinds, participants, context, status, placement.
ALTER TABLE timeline_events ADD COLUMN kind_json TEXT NOT NULL DEFAULT '{"kind":"acts"}';
ALTER TABLE timeline_events ADD COLUMN participants_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE timeline_events ADD COLUMN context_json TEXT NOT NULL DEFAULT '{"kind":"narration"}';
ALTER TABLE timeline_events ADD COLUMN status TEXT NOT NULL DEFAULT 'confirmed';
ALTER TABLE timeline_events ADD COLUMN confidence REAL NOT NULL DEFAULT 1.0;
ALTER TABLE timeline_events ADD COLUMN anchor_json TEXT;
ALTER TABLE timeline_events ADD COLUMN tick INTEGER;
ALTER TABLE timeline_events ADD COLUMN tick_lo INTEGER;
ALTER TABLE timeline_events ADD COLUMN tick_hi INTEGER;
ALTER TABLE timeline_events ADD COLUMN provenance_json TEXT NOT NULL DEFAULT '{"kind":"author","action":""}';

-- Suppressions that know their scope and evidence.
ALTER TABLE suppressions ADD COLUMN scope TEXT NOT NULL DEFAULT 'document';
ALTER TABLE suppressions ADD COLUMN diagnostic_id TEXT;
ALTER TABLE suppressions ADD COLUMN evidence_fingerprint TEXT;
ALTER TABLE suppressions ADD COLUMN stale INTEGER NOT NULL DEFAULT 0;

-- Named snapshots commit to an event head.
ALTER TABLE document_snapshots ADD COLUMN seq INTEGER NOT NULL DEFAULT 0;
ALTER TABLE document_snapshots ADD COLUMN chain_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE document_snapshots ADD COLUMN merkle_root TEXT NOT NULL DEFAULT '';

-- The hash-chained block event log. Append-only by application policy.
CREATE TABLE block_events (
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    seq INTEGER NOT NULL,
    event_id TEXT NOT NULL,
    actor TEXT NOT NULL,
    created_at TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    prev_chain_hash TEXT NOT NULL,
    chain_hash TEXT NOT NULL,
    PRIMARY KEY (document_id, seq),
    UNIQUE (document_id, chain_hash)
) STRICT;

CREATE TABLE block_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    seq INTEGER NOT NULL,
    chain_hash TEXT NOT NULL,
    merkle_root TEXT NOT NULL,
    state_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE (document_id, seq)
) STRICT;

CREATE TABLE drafts (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    base_seq INTEGER NOT NULL,
    base_state_json TEXT NOT NULL,
    content_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    merged_at TEXT
) STRICT;

CREATE TABLE project_settings (
    project_id TEXT PRIMARY KEY NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    settings_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE invariants (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    invariant_json TEXT NOT NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE routes (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    route_json TEXT NOT NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE baselines (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    revision TEXT NOT NULL,
    diagnostic_ids_json TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
) STRICT;

-- Local authentication: hashed tokens, salted pairing codes, one owner passphrase.
CREATE TABLE auth_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    hash TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK (kind IN ('desk', 'guest', 'session')),
    label TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT,
    revoked_at TEXT,
    last_used_at TEXT
) STRICT;

CREATE TABLE pairing_codes (
    id TEXT PRIMARY KEY NOT NULL,
    code_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    label TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    consumed_at TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE owner_credential (
    id INTEGER PRIMARY KEY NOT NULL CHECK (id = 1),
    phc TEXT NOT NULL,
    created_at TEXT NOT NULL
) STRICT;

-- Packs for the new analyses.
INSERT OR IGNORE INTO method_packs (id, pack_key, name, version, summary, enabled, manifest_json, created_at)
VALUES
(
    '0199b000-0000-7000-8000-000000000005',
    'relations',
    'Relations',
    '0.1.0',
    'Genealogy cycles, exclusive relations, and parent ages.',
    1,
    '{"rules":["CANON-E501","CANON-E502","CANON-E503"]}',
    '2026-10-12T00:00:00+00:00'
),
(
    '0199b000-0000-7000-8000-000000000006',
    'knowledge',
    'Knowledge',
    '0.1.0',
    'Who knows what, and when they learned it.',
    1,
    '{"rules":["CANON-E401"]}',
    '2026-10-12T00:00:00+00:00'
),
(
    '0199b000-0000-7000-8000-000000000007',
    'atlas',
    'Atlas',
    '0.1.0',
    'Travel that fits the time, over the routes you declare.',
    1,
    '{"rules":["CANON-E301","CANON-E302"]}',
    '2026-10-12T00:00:00+00:00'
),
(
    '0199b000-0000-7000-8000-000000000008',
    'invariants',
    'Invariants',
    '0.1.0',
    'Your own rules over the story, and the setup/payoff ledger.',
    1,
    '{"rules":["CANON-E601","PLOT-I001","PLOT-I002"]}',
    '2026-10-12T00:00:00+00:00'
);

UPDATE method_packs SET summary = 'Core canon compiler: attributes, names, and the two clocks between settled facts.', manifest_json = '{"rules":["CANON-E013","CANON-E014","CANON-E015","NAME-W001","NAME-W002","NAME-W003","NAME-W004"],"locked":true}' WHERE pack_key = 'scribe';
UPDATE method_packs SET summary = 'Lifecycles in story time: acting after death, appearing before birth, using what was destroyed.', manifest_json = '{"rules":["CANON-E101","CANON-E102","CANON-E103"]}' WHERE pack_key = 'lifecycle';
UPDATE method_packs SET summary = 'Story time against reading order: continuous runs, impossible dates, effects before causes. Flashbacks are not errors.', manifest_json = '{"rules":["CANON-E201","CANON-E202","CANON-E203"]}' WHERE pack_key = 'clocks';
