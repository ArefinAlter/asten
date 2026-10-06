PRAGMA foreign_keys = ON;

CREATE TABLE world_entries (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (
        kind IN (
            'place',
            'culture',
            'system',
            'creature',
            'item',
            'question',
            'fragment'
        )
    ),
    title TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    prompt TEXT CHECK (
        prompt IS NULL OR prompt IN ('why', 'how', 'what_result')
    ),
    status TEXT NOT NULL CHECK (status IN ('spark', 'sketched', 'settled')),
    board_x REAL NOT NULL DEFAULT 12,
    board_y REAL NOT NULL DEFAULT 12,
    map_x REAL,
    map_y REAL,
    hue TEXT NOT NULL DEFAULT 'parchment',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
) STRICT;

CREATE INDEX world_entries_project
    ON world_entries(project_id, kind, status);
