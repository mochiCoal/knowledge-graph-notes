PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS courses (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS knowledge_points (
    id TEXT PRIMARY KEY,
    course_id TEXT NOT NULL,
    parent_id TEXT,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    FOREIGN KEY (course_id)
        REFERENCES courses(id)
        ON DELETE CASCADE,

    FOREIGN KEY (parent_id)
        REFERENCES knowledge_points(id)
        ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_knowledge_course
ON knowledge_points(course_id);

CREATE INDEX IF NOT EXISTS idx_knowledge_parent
ON knowledge_points(parent_id);

CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    knowledge_point_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    FOREIGN KEY (knowledge_point_id)
        REFERENCES knowledge_points(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_notes_knowledge
ON notes(knowledge_point_id);

CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS note_tags (
    note_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,

    PRIMARY KEY (note_id, tag_id),

    FOREIGN KEY (note_id)
        REFERENCES notes(id)
        ON DELETE CASCADE,

    FOREIGN KEY (tag_id)
        REFERENCES tags(id)
        ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS knowledge_relations (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,

    CHECK (source_id <> target_id),

    UNIQUE (source_id, target_id, relation_type),

    FOREIGN KEY (source_id)
        REFERENCES knowledge_points(id)
        ON DELETE CASCADE,

    FOREIGN KEY (target_id)
        REFERENCES knowledge_points(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_relation_source
ON knowledge_relations(source_id);

CREATE INDEX IF NOT EXISTS idx_relation_target
ON knowledge_relations(target_id);