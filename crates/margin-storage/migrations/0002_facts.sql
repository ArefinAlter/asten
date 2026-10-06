PRAGMA foreign_keys = ON;

CREATE TABLE facts (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    entity_key TEXT NOT NULL,
    predicate TEXT NOT NULL,
    value TEXT NOT NULL,
    modality TEXT NOT NULL CHECK (
        modality IN ('established', 'inferred', 'contested', 'author_override')
    ),
    status TEXT NOT NULL CHECK (status IN ('proposed', 'confirmed', 'deprecated')),
    valid_from TEXT,
    valid_to TEXT,
    source_document_id TEXT REFERENCES documents(id) ON DELETE SET NULL,
    source_start INTEGER,
    source_end INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX facts_project_entity
    ON facts(project_id, entity_key, predicate);
