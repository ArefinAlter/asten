PRAGMA foreign_keys = ON;

CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('novel', 'screenplay', 'non_fiction', 'other')),
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE documents (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    parent_id TEXT REFERENCES documents(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (
        kind IN ('manuscript', 'chapter', 'scene', 'wiki', 'note', 'source')
    ),
    title TEXT NOT NULL,
    content_json TEXT NOT NULL DEFAULT '{}',
    position INTEGER NOT NULL DEFAULT 0,
    revision INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX documents_project_position
    ON documents(project_id, parent_id, position);

CREATE TABLE schema_metadata (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
) STRICT;

INSERT INTO schema_metadata (key, value)
VALUES ('content_format', 'prosemirror-json-v1');
