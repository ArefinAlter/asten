PRAGMA foreign_keys = ON;

ALTER TABLE documents ADD COLUMN exclude_from_compile INTEGER NOT NULL DEFAULT 0;

CREATE TABLE entities (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (
        kind IN ('character', 'place', 'event', 'item', 'rule', 'source', 'other')
    ),
    name TEXT NOT NULL,
    aliases TEXT NOT NULL DEFAULT '[]',
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX entities_project_kind ON entities(project_id, kind, name);

CREATE TABLE timeline_events (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    entity_id TEXT REFERENCES entities(id) ON DELETE SET NULL,
    document_id TEXT REFERENCES documents(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    story_time TEXT,
    reading_order INTEGER,
    duration TEXT,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX timeline_events_project ON timeline_events(project_id, reading_order);

CREATE TABLE comments (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    block_id TEXT,
    from_offset INTEGER NOT NULL DEFAULT 0,
    to_offset INTEGER NOT NULL DEFAULT 0,
    body TEXT NOT NULL,
    resolved INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE document_ops (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    seq INTEGER NOT NULL,
    kind TEXT NOT NULL,
    op_json TEXT NOT NULL,
    actor TEXT NOT NULL DEFAULT 'author',
    created_at TEXT NOT NULL,
    UNIQUE (document_id, seq)
) STRICT;

CREATE TABLE document_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL,
    label TEXT,
    content_json TEXT NOT NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE suppressions (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    rule_code TEXT NOT NULL,
    document_id TEXT REFERENCES documents(id) ON DELETE CASCADE,
    entity_key TEXT,
    reason TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE method_packs (
    id TEXT PRIMARY KEY NOT NULL,
    pack_key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    enabled INTEGER NOT NULL DEFAULT 1,
    manifest_json TEXT NOT NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE TABLE tracked_changes (
    id TEXT PRIMARY KEY NOT NULL,
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('insert', 'delete')),
    block_id TEXT,
    text TEXT NOT NULL,
    actor TEXT NOT NULL DEFAULT 'author',
    accepted INTEGER,
    created_at TEXT NOT NULL
) STRICT;

INSERT INTO method_packs (id, pack_key, name, version, summary, enabled, manifest_json, created_at)
VALUES
(
    '0199b000-0000-7000-8000-000000000001',
    'scribe',
    'Scribe',
    '0.1.0',
    'Core canon compiler. Attribute contradiction is always on.',
    1,
    '{"rules":["CANON-E013"],"locked":true}',
    '2026-10-06T00:00:00+00:00'
),
(
    '0199b000-0000-7000-8000-000000000002',
    'lifecycle',
    'Lifecycle',
    '0.1.0',
    'Notices a settled death followed by a later clocked act.',
    1,
    '{"rules":["CANON-E101"]}',
    '2026-10-06T00:00:00+00:00'
),
(
    '0199b000-0000-7000-8000-000000000003',
    'clocks',
    'Clocks',
    '0.1.0',
    'Compares reading order with stated story time. Silent until both clocks exist.',
    1,
    '{"rules":["CANON-E201"]}',
    '2026-10-06T00:00:00+00:00'
),
(
    '0199b000-0000-7000-8000-000000000004',
    'diction',
    'Diction watch',
    '0.1.0',
    'Optional craft hints. Never a canon error. Off until you enable it.',
    0,
    '{"hints":[{"code":"CRAFT-H001","word":"very","message":"Filter word. Keep it if the voice wants it."},{"code":"CRAFT-H002","word":"really","message":"Filter word. Cut only if you mean to."}]}',
    '2026-10-06T00:00:00+00:00'
);
