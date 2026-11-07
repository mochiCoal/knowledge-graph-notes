-- 阶段六：知识点关联类型字典与关联建议
--
-- 说明：
-- 1. 001_initial.sql 中的 knowledge_relations 已通过 CHECK / UNIQUE 约束
--    阻止自关联与重复关联，本迁移只追加类型字典与建议持久化所需的表；
-- 2. 关联类型用字典表维护，图谱阶段可直接读取颜色与是否有向；
-- 3. 关联建议以「无序知识点对」唯一保存，拒绝后不会反复推荐。

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS relation_types (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL DEFAULT '#337ea9',
    -- 1 表示有向（前置、包含、延伸），0 表示无向（相关、对比）
    directed INTEGER NOT NULL DEFAULT 1 CHECK (directed IN (0, 1)),
    description TEXT NOT NULL DEFAULT '',
    is_builtin INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0, 1)),
    sort_order INTEGER NOT NULL DEFAULT 100,
    created_at TEXT NOT NULL
);

INSERT OR IGNORE INTO relation_types
    (id, label, color, directed, description, is_builtin, sort_order, created_at)
VALUES
    ('prerequisite', '前置知识', '#4f7cac', 1, '学习目标知识点之前需要先掌握源知识点', 1, 10, '2026-01-01T00:00:00Z'),
    ('related',      '相关知识', '#2f8f5b', 0, '两个知识点内容相关，可以相互参照', 1, 20, '2026-01-01T00:00:00Z'),
    ('contains',     '包含关系', '#8a6bbf', 1, '源知识点在内容上包含目标知识点', 1, 30, '2026-01-01T00:00:00Z'),
    ('contrast',     '对比辨析', '#c07a2c', 0, '两个知识点容易混淆，需要对比辨析', 1, 40, '2026-01-01T00:00:00Z'),
    ('extends',      '延伸拓展', '#337ea9', 1, '源知识点可以进一步延伸到目标知识点', 1, 50, '2026-01-01T00:00:00Z');

CREATE TABLE IF NOT EXISTS relation_suggestions (
    id TEXT PRIMARY KEY,
    -- source_id / target_id 按字典序保存，保证同一知识点对只有一条建议
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    relation_type TEXT NOT NULL,
    -- 综合相似度，0 ~ 1
    score REAL NOT NULL DEFAULT 0,
    -- JSON 字符串数组，存放可直接展示的推荐理由
    reasons TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'rejected')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,

    CHECK (source_id <> target_id),
    CHECK (source_id < target_id),
    UNIQUE (source_id, target_id),

    FOREIGN KEY (source_id)
        REFERENCES knowledge_points(id)
        ON DELETE CASCADE,

    FOREIGN KEY (target_id)
        REFERENCES knowledge_points(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_suggestion_source
ON relation_suggestions(source_id);

CREATE INDEX IF NOT EXISTS idx_suggestion_target
ON relation_suggestions(target_id);

CREATE INDEX IF NOT EXISTS idx_suggestion_status
ON relation_suggestions(status);
