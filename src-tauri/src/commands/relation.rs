use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use rusqlite::{params, Connection, OptionalExtension, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, get_connection};
use crate::models::relation::{
    KnowledgeRelation, RelationDetail, RelationSuggestion, RelationType,
};

/// 关联建议生成的默认类型（无向的「相关知识」）
const DEFAULT_SUGGESTION_TYPE: &str = "related";
/// 低于该综合相似度的候选不进入建议列表
const MIN_SUGGESTION_SCORE: f64 = 0.02;
/// 单次最多返回的关联建议数量
const MAX_SUGGESTIONS: i64 = 20;
/// 每个知识点参与相似度计算的笔记文本上限（字符数）
const NOTE_TEXT_LIMIT: usize = 3000;

/// 关联查询统一使用的 SELECT 片段，字段顺序与 [`map_relation`] 一致。
const RELATION_SELECT: &str = "
SELECT id, source_id, target_id, relation_type, description, created_at
FROM knowledge_relations
";

/// 关联详情查询片段，末尾不带 WHERE，供不同过滤条件拼接。
const RELATION_DETAIL_SELECT: &str = "
SELECT r.id, r.source_id, r.target_id, r.relation_type, r.description, r.created_at,
       s.title AS source_title, t.title AS target_title,
       s.course_id AS source_course_id, t.course_id AS target_course_id,
       sc.name AS source_course_name, tc.name AS target_course_name,
       COALESCE(rt.label, r.relation_type) AS relation_label,
       COALESCE(rt.color, '#337ea9') AS relation_color,
       COALESCE(rt.directed, 1) AS directed
FROM knowledge_relations r
JOIN knowledge_points s ON s.id = r.source_id
JOIN knowledge_points t ON t.id = r.target_id
JOIN courses sc ON sc.id = s.course_id
JOIN courses tc ON tc.id = t.course_id
LEFT JOIN relation_types rt ON rt.id = r.relation_type
";

fn map_relation(row: &Row<'_>) -> rusqlite::Result<KnowledgeRelation> {
    Ok(KnowledgeRelation {
        id: row.get(0)?,
        source_id: row.get(1)?,
        target_id: row.get(2)?,
        relation_type: row.get(3)?,
        description: row.get(4)?,
        created_at: row.get(5)?,
    })
}

fn map_relation_detail(row: &Row<'_>) -> rusqlite::Result<RelationDetail> {
    let relation = KnowledgeRelation {
        id: row.get(0)?,
        source_id: row.get(1)?,
        target_id: row.get(2)?,
        relation_type: row.get(3)?,
        description: row.get(4)?,
        created_at: row.get(5)?,
    };

    Ok(RelationDetail {
        relation,
        source_title: row.get(6)?,
        target_title: row.get(7)?,
        source_course_id: row.get(8)?,
        source_course_name: row.get(10)?,
        target_course_id: row.get(9)?,
        target_course_name: row.get(11)?,
        relation_label: row.get(12)?,
        relation_color: row.get(13)?,
        directed: row.get::<_, i64>(14)? != 0,
    })
}

fn map_relation_type(row: &Row<'_>) -> rusqlite::Result<RelationType> {
    Ok(RelationType {
        id: row.get(0)?,
        label: row.get(1)?,
        color: row.get(2)?,
        directed: row.get::<_, i64>(3)? != 0,
        description: row.get(4)?,
        is_builtin: row.get::<_, i64>(5)? != 0,
        sort_order: row.get(6)?,
        created_at: row.get(7)?,
        usage_count: row.get(8)?,
    })
}

/// 关联写入失败的统一处理：把数据库约束错误转换成可直接展示的提示。
fn relation_write_error(context: &str, error: rusqlite::Error) -> String {
    let message = error.to_string();

    if message.contains("UNIQUE constraint failed: knowledge_relations") {
        "该关联已存在，请勿重复创建".to_string()
    } else if message.contains("CHECK constraint failed") {
        "不能把知识点关联到自身".to_string()
    } else {
        db_error(context, message)
    }
}

fn find_relation(conn: &Connection, relation_id: &str) -> Result<Option<KnowledgeRelation>, String> {
    let sql = format!("{RELATION_SELECT} WHERE id = ?1");
    conn.query_row(&sql, [relation_id], map_relation)
        .optional()
        .map_err(|e| db_error("读取知识点关联", e))
}

fn require_relation(conn: &Connection, relation_id: &str) -> Result<KnowledgeRelation, String> {
    find_relation(conn, relation_id)?.ok_or_else(|| "关联不存在或已被删除，请刷新后重试".to_string())
}

fn load_relation_detail(conn: &Connection, relation_id: &str) -> Result<RelationDetail, String> {
    let sql = format!("{RELATION_DETAIL_SELECT} WHERE r.id = ?1");
    conn.query_row(&sql, [relation_id], map_relation_detail)
        .optional()
        .map_err(|e| db_error("读取关联详情", e))?
        .ok_or_else(|| "关联不存在或已被删除，请刷新后重试".to_string())
}

fn ensure_point_exists(conn: &Connection, point_id: &str) -> Result<(), String> {
    conn.query_row(
        "SELECT 1 FROM knowledge_points WHERE id = ?1",
        [point_id],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(|e| db_error("读取知识点", e))?
    .map(|_| ())
    .ok_or_else(|| "知识点不存在或已被删除，请刷新后重试".to_string())
}

/// 读取关联类型的有向标记，同时完成「类型是否存在」的校验
fn require_type_directed(conn: &Connection, type_id: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT directed FROM relation_types WHERE id = ?1",
        [type_id],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(|e| db_error("读取关联类型", e))?
    .map(|value| value != 0)
    .ok_or_else(|| "关联类型不存在，请重新选择".to_string())
}

fn normalize_description(description: &str) -> Result<String, String> {
    let description = description.trim();

    if description.chars().count() > 500 {
        return Err("关联描述不能超过 500 个字符".into());
    }

    Ok(description.to_string())
}

fn normalize_type_label(label: &str) -> Result<String, String> {
    let label = label.trim();

    if label.is_empty() {
        return Err("关联类型名称不能为空".into());
    }
    if label.chars().count() > 20 {
        return Err("关联类型名称不能超过 20 个字符".into());
    }

    Ok(label.to_string())
}

fn normalize_type_color(color: &str) -> Result<String, String> {
    let color = color.trim();

    if color.is_empty() {
        return Ok("#337ea9".to_string());
    }

    let valid = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|ch| ch.is_ascii_hexdigit());

    if !valid {
        return Err("关联类型颜色需为 #RRGGBB 格式".into());
    }

    Ok(color.to_ascii_lowercase())
}

fn normalize_type_description(description: &str) -> Result<String, String> {
    let description = description.trim();

    if description.chars().count() > 200 {
        return Err("关联类型说明不能超过 200 个字符".into());
    }

    Ok(description.to_string())
}

/// 校验两端知识点与关联类型，并阻止自关联与重复关联。
///
/// 无向类型（related / contrast）额外拦截反向重复，例如已有 A→B 的
/// 「相关知识」时，不再允许创建 B→A 的同类型关联。
fn validate_relation_targets(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    type_id: &str,
    exclude_relation_id: Option<&str>,
) -> Result<(String, String), String> {
    let source_id = source_id.trim();
    let target_id = target_id.trim();

    if source_id == target_id {
        return Err("不能把知识点关联到自身".into());
    }

    ensure_point_exists(conn, source_id)?;
    ensure_point_exists(conn, target_id)?;

    let directed = require_type_directed(conn, type_id)?;
    let exclude_id = exclude_relation_id.unwrap_or("");

    let duplicated: i64 = if directed {
        conn.query_row(
            "SELECT COUNT(*) FROM knowledge_relations
             WHERE source_id = ?1 AND target_id = ?2 AND relation_type = ?3 AND id <> ?4",
            params![source_id, target_id, type_id, exclude_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("检查重复关联", e))?
    } else {
        conn.query_row(
            "SELECT COUNT(*) FROM knowledge_relations
             WHERE relation_type = ?1
               AND ((source_id = ?2 AND target_id = ?3) OR (source_id = ?3 AND target_id = ?2))
               AND id <> ?4",
            params![type_id, source_id, target_id, exclude_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("检查重复关联", e))?
    };

    if duplicated > 0 {
        return Err("这两个知识点之间已存在该类型的关联，请勿重复创建".into());
    }

    Ok((source_id.to_string(), target_id.to_string()))
}

fn insert_relation(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    type_id: &str,
    description: &str,
    created_at: &str,
) -> Result<KnowledgeRelation, String> {
    let relation = KnowledgeRelation {
        id: Uuid::new_v4().to_string(),
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        relation_type: type_id.to_string(),
        description: description.to_string(),
        created_at: created_at.to_string(),
    };

    conn.execute(
        "INSERT INTO knowledge_relations
            (id, source_id, target_id, relation_type, description, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            relation.id,
            relation.source_id,
            relation.target_id,
            relation.relation_type,
            relation.description,
            relation.created_at
        ],
    )
    .map_err(|e| relation_write_error("创建知识点关联", e))?;

    Ok(relation)
}

/// 知识点对按字典序规范化，与 relation_suggestions 的 `source_id < target_id` 约束一致
fn canonical_pair(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_string(), right.to_string())
    } else {
        (right.to_string(), left.to_string())
    }
}

/// 手动创建关联后清理同一知识点对的建议，避免「已有正式关联仍被推荐」
fn clear_suggestion_pair(conn: &Connection, left: &str, right: &str) -> Result<(), String> {
    let (source_id, target_id) = canonical_pair(left, right);
    conn.execute(
        "DELETE FROM relation_suggestions WHERE source_id = ?1 AND target_id = ?2",
        params![source_id, target_id],
    )
    .map_err(|e| db_error("清理关联建议", e))?;

    Ok(())
}

/* ------------------------------------------------------------------ */
/* 关联类型管理                                                        */
/* ------------------------------------------------------------------ */

const TYPE_SELECT: &str = "
SELECT t.id, t.label, t.color, t.directed, t.description, t.is_builtin, t.sort_order,
       t.created_at,
       (SELECT COUNT(*) FROM knowledge_relations r WHERE r.relation_type = t.id)
FROM relation_types t
";

fn list_types(conn: &Connection) -> Result<Vec<RelationType>, String> {
    let sql = format!("{TYPE_SELECT} ORDER BY t.sort_order ASC, t.label ASC");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取关联类型", e))?;

    stmt.query_map([], map_relation_type)
        .map_err(|e| db_error("读取关联类型", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取关联类型", e))
}

fn load_type(conn: &Connection, type_id: &str) -> Result<RelationType, String> {
    let sql = format!("{TYPE_SELECT} WHERE t.id = ?1");
    conn.query_row(&sql, [type_id], map_relation_type)
        .optional()
        .map_err(|e| db_error("读取关联类型", e))?
        .ok_or_else(|| "关联类型不存在或已被删除".to_string())
}

fn ensure_type_label_available(
    conn: &Connection,
    label: &str,
    exclude_type_id: Option<&str>,
) -> Result<(), String> {
    let duplicated: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM relation_types WHERE label = ?1 AND id <> ?2",
            params![label, exclude_type_id.unwrap_or("")],
            |row| row.get(0),
        )
        .map_err(|e| db_error("检查关联类型名称", e))?;

    if duplicated > 0 {
        return Err("已存在同名关联类型，请换一个名称".into());
    }

    Ok(())
}

fn create_type(
    conn: &Connection,
    label: &str,
    color: &str,
    directed: bool,
    description: &str,
) -> Result<RelationType, String> {
    let label = normalize_type_label(label)?;
    let color = normalize_type_color(color)?;
    let description = normalize_type_description(description)?;
    ensure_type_label_available(conn, &label, None)?;

    let sort_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) + 10 FROM relation_types",
            [],
            |row| row.get(0),
        )
        .map_err(|e| db_error("读取关联类型排序", e))?;

    let type_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO relation_types
            (id, label, color, directed, description, is_builtin, sort_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
        params![
            type_id,
            label,
            color,
            directed as i64,
            description,
            sort_order,
            created_at
        ],
    )
    .map_err(|e| relation_write_error("创建关联类型", e))?;

    load_type(conn, &type_id)
}

fn update_type(
    conn: &Connection,
    type_id: &str,
    label: &str,
    color: &str,
    directed: bool,
    description: &str,
) -> Result<RelationType, String> {
    let existing = load_type(conn, type_id)?;
    let label = normalize_type_label(label)?;
    let color = normalize_type_color(color)?;
    let description = normalize_type_description(description)?;
    ensure_type_label_available(conn, &label, Some(&existing.id))?;

    conn.execute(
        "UPDATE relation_types
            SET label = ?1, color = ?2, directed = ?3, description = ?4
          WHERE id = ?5",
        params![label, color, directed as i64, description, type_id],
    )
    .map_err(|e| relation_write_error("修改关联类型", e))?;

    load_type(conn, type_id)
}

fn delete_type(conn: &Connection, type_id: &str) -> Result<(), String> {
    let existing = load_type(conn, type_id)?;

    if existing.is_builtin {
        return Err("内置关联类型不能删除，可以修改名称或颜色".into());
    }
    if existing.usage_count > 0 {
        return Err(format!(
            "该类型已被 {} 条关联使用，请先修改这些关联的类型再删除",
            existing.usage_count
        ));
    }

    conn.execute("DELETE FROM relation_types WHERE id = ?1", [type_id])
        .map_err(|e| db_error("删除关联类型", e))?;

    Ok(())
}

/* ------------------------------------------------------------------ */
/* 关联关系查询与增删改                                                */
/* ------------------------------------------------------------------ */

fn list_all_relations(conn: &Connection) -> Result<Vec<KnowledgeRelation>, String> {
    let sql = format!("{RELATION_SELECT} ORDER BY created_at ASC");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取知识点关联", e))?;

    stmt.query_map([], map_relation)
        .map_err(|e| db_error("读取知识点关联", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取知识点关联", e))
}

fn list_details(conn: &Connection) -> Result<Vec<RelationDetail>, String> {
    let sql = format!("{RELATION_DETAIL_SELECT} ORDER BY r.created_at DESC");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取关联列表", e))?;

    stmt.query_map([], map_relation_detail)
        .map_err(|e| db_error("读取关联列表", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取关联列表", e))
}

fn list_details_for_point(
    conn: &Connection,
    point_id: &str,
) -> Result<Vec<RelationDetail>, String> {
    ensure_point_exists(conn, point_id)?;

    let sql = format!(
        "{RELATION_DETAIL_SELECT}
         WHERE r.source_id = ?1 OR r.target_id = ?1
         ORDER BY r.created_at DESC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取知识点关联", e))?;

    stmt.query_map([point_id], map_relation_detail)
        .map_err(|e| db_error("读取知识点关联", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取知识点关联", e))
}

fn list_details_by_course(
    conn: &Connection,
    course_id: &str,
) -> Result<Vec<RelationDetail>, String> {
    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM courses WHERE id = ?1",
            [course_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("读取课程", e))?;

    if exists == 0 {
        return Err("课程不存在，请刷新后重试".to_string());
    }

    let sql = format!(
        "{RELATION_DETAIL_SELECT}
         WHERE s.course_id = ?1 OR t.course_id = ?1
         ORDER BY r.created_at DESC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取课程关联", e))?;

    stmt.query_map([course_id], map_relation_detail)
        .map_err(|e| db_error("读取课程关联", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取课程关联", e))
}

fn create_relation_record(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    type_id: &str,
    description: &str,
) -> Result<RelationDetail, String> {
    let description = normalize_description(description)?;
    let now = chrono::Utc::now().to_rfc3339();

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| db_error("创建知识点关联", e))?;

    let (source_id, target_id) =
        validate_relation_targets(&tx, source_id, target_id, type_id, None)?;
    let relation = insert_relation(&tx, &source_id, &target_id, type_id, &description, &now)?;

    clear_suggestion_pair(&tx, &source_id, &target_id)?;

    tx.commit()
        .map_err(|e| db_error("创建知识点关联", e))?;

    load_relation_detail(conn, &relation.id)
}

fn update_relation_record(
    conn: &Connection,
    relation_id: &str,
    type_id: &str,
    description: &str,
) -> Result<RelationDetail, String> {
    let existing = require_relation(conn, relation_id)?;
    let description = normalize_description(description)?;

    validate_relation_targets(
        conn,
        &existing.source_id,
        &existing.target_id,
        type_id,
        Some(relation_id),
    )?;

    conn.execute(
        "UPDATE knowledge_relations SET relation_type = ?1, description = ?2 WHERE id = ?3",
        params![type_id, description, relation_id],
    )
    .map_err(|e| relation_write_error("修改知识点关联", e))?;

    load_relation_detail(conn, relation_id)
}

fn delete_relation_record(conn: &Connection, relation_id: &str) -> Result<(), String> {
    let existing = require_relation(conn, relation_id)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| db_error("删除知识点关联", e))?;

    tx.execute("DELETE FROM knowledge_relations WHERE id = ?1", [relation_id])
        .map_err(|e| db_error("删除知识点关联", e))?;

    // 删除正式关联后允许再次被推荐
    clear_suggestion_pair(&tx, &existing.source_id, &existing.target_id)?;

    tx.commit()
        .map_err(|e| db_error("删除知识点关联", e))?;

    Ok(())
}

/* ------------------------------------------------------------------ */
/* 关联建议                                                            */
/* ------------------------------------------------------------------ */

fn is_cjk(ch: char) -> bool {
    matches!(ch as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF)
}

/// 中文按二元字组切分、英文按单词切分，去掉停用词与单字噪声。
///
/// 这里不引入分词模型，保证完全离线、可解释且无需额外依赖。
fn tokenize(text: &str) -> HashMap<String, f64> {
    let mut tokens: Vec<String> = Vec::new();
    let chars: Vec<char> = text.to_lowercase().chars().collect();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];

        if ch.is_ascii_alphanumeric() {
            let start = index;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric()
                    || chars[index] == '-'
                    || chars[index] == '_')
            {
                index += 1;
            }

            let word: String = chars[start..index].iter().collect();
            if word.len() >= 2 && !EN_STOPWORDS.contains(&word.as_str()) {
                tokens.push(word);
            }
        } else if is_cjk(ch) {
            let start = index;
            while index < chars.len() && is_cjk(chars[index]) {
                index += 1;
            }

            let run = &chars[start..index];
            if run.len() >= 2 {
                for pair in run.windows(2) {
                    // 含常见虚词的二元组区分度低，直接跳过（例如「的线」「和指」）
                    if pair.iter().any(|ch| CN_STOP_CHARS.contains(ch)) {
                        continue;
                    }

                    let token: String = pair.iter().collect();
                    if !CN_STOPWORDS.contains(&token.as_str()) {
                        tokens.push(token);
                    }
                }
            }
        } else {
            index += 1;
        }
    }

    let mut frequencies: HashMap<String, f64> = HashMap::new();
    for token in tokens {
        *frequencies.entry(token).or_insert(0.0) += 1.0;
    }

    frequencies
}

struct FieldTerms {
    title: HashMap<String, f64>,
    description: HashMap<String, f64>,
    notes: HashMap<String, f64>,
    all: HashMap<String, f64>,
}

impl FieldTerms {
    fn new(title: &str, description: &str, notes: &str) -> Self {
        let title = tokenize(title);
        let description = tokenize(description);
        let notes = tokenize(notes);

        let mut all: HashMap<String, f64> = HashMap::new();
        for term_map in [&title, &description, &notes] {
            for (term, weight) in term_map {
                *all.entry(term.clone()).or_insert(0.0) += *weight;
            }
        }

        FieldTerms {
            title,
            description,
            notes,
            all,
        }
    }
}

fn cosine(left: &HashMap<String, f64>, right: &HashMap<String, f64>) -> f64 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut left_norm = 0.0;

    for (term, weight) in left {
        left_norm += weight * weight;
        if let Some(other) = right.get(term) {
            dot += weight * other;
        }
    }

    let right_norm: f64 = right.values().map(|weight| weight * weight).sum();

    if left_norm == 0.0 || right_norm == 0.0 {
        return 0.0;
    }

    dot / (left_norm.sqrt() * right_norm.sqrt())
}

fn jaccard(left: &HashMap<String, f64>, right: &HashMap<String, f64>) -> f64 {
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }

    let intersection = left
        .keys()
        .filter(|term| right.contains_key(*term))
        .count();
    let union = left.len() + right.len() - intersection;

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// 取共同词，按两端词频之和降序，保证同分时理由顺序稳定
fn shared_terms(
    left: &HashMap<String, f64>,
    right: &HashMap<String, f64>,
    limit: usize,
) -> Vec<String> {
    let mut shared: Vec<(String, f64)> = left
        .iter()
        .filter_map(|(term, weight)| {
            right
                .get(term)
                .map(|other| (term.clone(), weight + other))
        })
        .collect();

    shared.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });

    shared
        .into_iter()
        .take(limit)
        .map(|(term, _)| term)
        .collect()
}

fn round_score(score: f64) -> f64 {
    (score * 10000.0).round() / 10000.0
}

/// 计算候选与目标知识点的综合相似度并生成可读理由。
///
/// 权重参考开题报告：名称 0.45、描述 0.25、笔记 0.20、词面 Jaccard 0.10。
fn score_candidate(
    target: &FieldTerms,
    candidate: &FieldTerms,
) -> (f64, Vec<String>) {
    let title_similarity = cosine(&target.title, &candidate.title);
    let description_similarity = cosine(&target.description, &candidate.description);
    let notes_similarity = cosine(&target.notes, &candidate.notes);
    let term_overlap = jaccard(&target.all, &candidate.all);

    let score = 0.45 * title_similarity
        + 0.25 * description_similarity
        + 0.20 * notes_similarity
        + 0.10 * term_overlap;

    let mut reasons: Vec<String> = Vec::new();

    let title_shared = shared_terms(&target.title, &candidate.title, 3);
    if !title_shared.is_empty() {
        reasons.push(format!("名称共同词：{}", title_shared.join("、")));
    }

    let description_shared = shared_terms(&target.description, &candidate.description, 3);
    if !description_shared.is_empty() {
        reasons.push(format!("描述共同词：{}", description_shared.join("、")));
    }

    let notes_shared = shared_terms(&target.notes, &candidate.notes, 3);
    if !notes_shared.is_empty() {
        reasons.push(format!("笔记共同提及：{}", notes_shared.join("、")));
    }

    if reasons.is_empty() {
        let primary = if title_similarity >= description_similarity
            && title_similarity >= notes_similarity
        {
            "名称"
        } else if description_similarity >= notes_similarity {
            "描述"
        } else {
            "笔记内容"
        };
        reasons.push(format!("{primary}等文本存在综合重合"));
    }

    (round_score(score), reasons)
}

struct PointBrief {
    id: String,
    course_id: String,
    title: String,
    description: String,
}

fn load_point_brief(conn: &Connection, point_id: &str) -> Result<PointBrief, String> {
    conn.query_row(
        "SELECT id, course_id, title, description FROM knowledge_points WHERE id = ?1",
        [point_id],
        |row| {
            Ok(PointBrief {
                id: row.get(0)?,
                course_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
            })
        },
    )
    .optional()
    .map_err(|e| db_error("读取知识点", e))?
    .ok_or_else(|| "知识点不存在或已被删除，请刷新后重试".to_string())
}

/// 同一课程内、除目标知识点外的其余知识点（第一阶段候选范围）
fn load_course_candidates(
    conn: &Connection,
    course_id: &str,
    exclude_id: &str,
) -> Result<Vec<PointBrief>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, course_id, title, description
             FROM knowledge_points
             WHERE course_id = ?1 AND id <> ?2
             ORDER BY created_at ASC",
        )
        .map_err(|e| db_error("读取候选知识点", e))?;

    stmt.query_map(params![course_id, exclude_id], |row| {
        Ok(PointBrief {
            id: row.get(0)?,
            course_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
        })
    })
    .map_err(|e| db_error("读取候选知识点", e))?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|e| db_error("读取候选知识点", e))
}

/// 读取一门课程下所有知识点的笔记文本（每个知识点最多截断到 3000 字）
fn load_note_texts(
    conn: &Connection,
    course_id: &str,
) -> Result<HashMap<String, String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT n.knowledge_point_id, n.title, n.content
             FROM notes n
             JOIN knowledge_points kp ON kp.id = n.knowledge_point_id
             WHERE kp.course_id = ?1
             ORDER BY n.updated_at DESC",
        )
        .map_err(|e| db_error("读取笔记内容", e))?;

    let rows = stmt
        .query_map([course_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| db_error("读取笔记内容", e))?;

    let mut texts: HashMap<String, String> = HashMap::new();

    for row in rows {
        let (point_id, title, content) = row.map_err(|e| db_error("读取笔记内容", e))?;
        let entry = texts.entry(point_id).or_default();
        let used = entry.chars().count();

        if used >= NOTE_TEXT_LIMIT {
            continue;
        }

        let mut text = String::new();
        text.push_str(&title);
        text.push('\n');
        text.push_str(&content);

        let remaining = NOTE_TEXT_LIMIT - used;
        entry.extend(text.chars().take(remaining));
    }

    Ok(texts)
}

fn related_point_ids(conn: &Connection, point_id: &str) -> Result<HashSet<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT source_id, target_id FROM knowledge_relations
             WHERE source_id = ?1 OR target_id = ?1",
        )
        .map_err(|e| db_error("读取已有关系", e))?;

    let rows = stmt
        .query_map([point_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| db_error("读取已有关系", e))?;

    let mut ids: HashSet<String> = HashSet::new();
    for row in rows {
        let (source_id, target_id) = row.map_err(|e| db_error("读取已有关系", e))?;
        ids.insert(if source_id == point_id {
            target_id
        } else {
            source_id
        });
    }

    Ok(ids)
}

/// 已接受 / 已拒绝的知识点对不再参与推荐
fn handled_suggestion_ids(
    conn: &Connection,
    point_id: &str,
) -> Result<HashSet<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT source_id, target_id FROM relation_suggestions
             WHERE status IN ('accepted', 'rejected')
               AND (source_id = ?1 OR target_id = ?1)",
        )
        .map_err(|e| db_error("读取已处理建议", e))?;

    let rows = stmt
        .query_map([point_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| db_error("读取已处理建议", e))?;

    let mut ids: HashSet<String> = HashSet::new();
    for row in rows {
        let (source_id, target_id) = row.map_err(|e| db_error("读取已处理建议", e))?;
        ids.insert(if source_id == point_id {
            target_id
        } else {
            source_id
        });
    }

    Ok(ids)
}

fn map_suggestion(row: &Row<'_>) -> rusqlite::Result<RelationSuggestion> {
    let reasons_text: String = row.get(8)?;
    let reasons: Vec<String> = serde_json::from_str(&reasons_text).unwrap_or_default();

    Ok(RelationSuggestion {
        id: row.get(0)?,
        source_id: row.get(1)?,
        target_id: row.get(2)?,
        source_title: row.get(3)?,
        target_title: row.get(4)?,
        relation_type: row.get(5)?,
        relation_type_label: row.get(6)?,
        score: row.get(7)?,
        reasons,
        status: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

const SUGGESTION_SELECT: &str = "
SELECT sg.id, sg.source_id, sg.target_id, s.title AS source_title,
       t.title AS target_title, sg.relation_type,
       COALESCE(rt.label, sg.relation_type) AS relation_type_label,
       sg.score, sg.reasons, sg.status, sg.created_at, sg.updated_at
FROM relation_suggestions sg
JOIN knowledge_points s ON s.id = sg.source_id
JOIN knowledge_points t ON t.id = sg.target_id
LEFT JOIN relation_types rt ON rt.id = sg.relation_type
";

fn load_pending_suggestions(
    conn: &Connection,
    point_id: &str,
    limit: i64,
) -> Result<Vec<RelationSuggestion>, String> {
    let sql = format!(
        "{SUGGESTION_SELECT}
         WHERE sg.status = 'pending' AND (sg.source_id = ?1 OR sg.target_id = ?1)
         ORDER BY sg.score DESC, sg.updated_at DESC
         LIMIT ?2"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取关联建议", e))?;

    stmt.query_map(params![point_id, limit], map_suggestion)
        .map_err(|e| db_error("读取关联建议", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取关联建议", e))
}

struct SuggestionRecord {
    id: String,
    source_id: String,
    target_id: String,
    relation_type: String,
    status: String,
}

fn load_suggestion(conn: &Connection, suggestion_id: &str) -> Result<SuggestionRecord, String> {
    conn.query_row(
        "SELECT id, source_id, target_id, relation_type, status
         FROM relation_suggestions WHERE id = ?1",
        [suggestion_id],
        |row| {
            Ok(SuggestionRecord {
                id: row.get(0)?,
                source_id: row.get(1)?,
                target_id: row.get(2)?,
                relation_type: row.get(3)?,
                status: row.get(4)?,
            })
        },
    )
    .optional()
    .map_err(|e| db_error("读取关联建议", e))?
    .ok_or_else(|| "关联建议不存在或已过期，请刷新后重试".to_string())
}

/// 生成（并刷新）某个知识点的关联建议。
///
/// 候选默认为同一课程内的其它知识点；已有正式关联、已接受或已拒绝的
/// 知识点对会被排除。建议以无序知识点对唯一保存，重复生成只更新分数与理由。
fn generate_suggestions(
    conn: &Connection,
    point_id: &str,
    limit: Option<i64>,
) -> Result<Vec<RelationSuggestion>, String> {
    let point = load_point_brief(conn, point_id)?;
    let limit = limit.unwrap_or(8).clamp(1, MAX_SUGGESTIONS);

    let candidates = load_course_candidates(conn, &point.course_id, &point.id)?;
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let note_texts = load_note_texts(conn, &point.course_id)?;
    let related = related_point_ids(conn, &point.id)?;
    let handled = handled_suggestion_ids(conn, &point.id)?;

    let target_notes = note_texts.get(&point.id).map(String::as_str).unwrap_or("");
    let target_terms = FieldTerms::new(&point.title, &point.description, target_notes);

    let mut scored: Vec<(PointBrief, f64, Vec<String>)> = Vec::new();

    for candidate in candidates {
        if related.contains(&candidate.id) || handled.contains(&candidate.id) {
            continue;
        }

        let candidate_notes = note_texts
            .get(&candidate.id)
            .map(String::as_str)
            .unwrap_or("");
        let candidate_terms =
            FieldTerms::new(&candidate.title, &candidate.description, candidate_notes);

        let (score, reasons) = score_candidate(&target_terms, &candidate_terms);

        if score >= MIN_SUGGESTION_SCORE {
            scored.push((candidate, score, reasons));
        }
    }

    scored.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0.id.cmp(&right.0.id))
    });
    scored.truncate(limit as usize);

    let now = chrono::Utc::now().to_rfc3339();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| db_error("生成关联建议", e))?;

    tx.execute(
        "DELETE FROM relation_suggestions
         WHERE status = 'pending' AND (source_id = ?1 OR target_id = ?1)",
        [&point.id],
    )
    .map_err(|e| db_error("刷新关联建议", e))?;

    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO relation_suggestions
                    (id, source_id, target_id, relation_type, score, reasons,
                     status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?7)",
            )
            .map_err(|e| db_error("保存关联建议", e))?;

        for (candidate, score, reasons) in &scored {
            let (source_id, target_id) = canonical_pair(&point.id, &candidate.id);
            let reasons = serde_json::to_string(reasons)
                .map_err(|e| format!("序列化关联建议理由失败：{e}"))?;

            stmt.execute(params![
                Uuid::new_v4().to_string(),
                source_id,
                target_id,
                DEFAULT_SUGGESTION_TYPE,
                score,
                reasons,
                now
            ])
            .map_err(|e| db_error("保存关联建议", e))?;
        }
    }

    tx.commit()
        .map_err(|e| db_error("生成关联建议", e))?;

    load_pending_suggestions(conn, &point.id, limit)
}

fn accept_suggestion(conn: &Connection, suggestion_id: &str) -> Result<RelationDetail, String> {
    let suggestion = load_suggestion(conn, suggestion_id)?;

    if suggestion.status != "pending" {
        return Err("该关联建议已处理，请刷新列表".to_string());
    }

    let (source_id, target_id) = validate_relation_targets(
        conn,
        &suggestion.source_id,
        &suggestion.target_id,
        &suggestion.relation_type,
        None,
    )?;

    let now = chrono::Utc::now().to_rfc3339();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| db_error("接受关联建议", e))?;

    let relation = insert_relation(
        &tx,
        &source_id,
        &target_id,
        &suggestion.relation_type,
        "",
        &now,
    )?;

    tx.execute(
        "UPDATE relation_suggestions SET status = 'accepted', updated_at = ?1 WHERE id = ?2",
        params![now, suggestion.id],
    )
    .map_err(|e| db_error("接受关联建议", e))?;

    tx.commit()
        .map_err(|e| db_error("接受关联建议", e))?;

    load_relation_detail(conn, &relation.id)
}

fn reject_suggestion(conn: &Connection, suggestion_id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().to_rfc3339();
    let updated = conn
        .execute(
            "UPDATE relation_suggestions
                SET status = 'rejected', updated_at = ?1
              WHERE id = ?2 AND status = 'pending'",
            params![now, suggestion_id],
        )
        .map_err(|e| db_error("拒绝关联建议", e))?;

    if updated == 0 {
        return Err("该关联建议已处理或不存在，请刷新列表".to_string());
    }

    Ok(())
}

/* ------------------------------------------------------------------ */
/* Tauri 命令                                                          */
/* ------------------------------------------------------------------ */

/// 查询全部知识点关联（笔记页右侧「相关知识点」使用）
#[tauri::command]
pub fn list_relations(app: AppHandle) -> Result<Vec<KnowledgeRelation>, String> {
    let conn = get_connection(&app)?;
    list_all_relations(&conn)
}

/// 查询全部关联详情（含两端标题、课程与类型信息）
#[tauri::command]
pub fn list_relation_details(app: AppHandle) -> Result<Vec<RelationDetail>, String> {
    let conn = get_connection(&app)?;
    list_details(&conn)
}

/// 查询与某个知识点相关的全部关联
#[tauri::command]
pub fn list_relation_details_for_point(
    app: AppHandle,
    knowledge_point_id: String,
) -> Result<Vec<RelationDetail>, String> {
    let conn = get_connection(&app)?;
    list_details_for_point(&conn, knowledge_point_id.trim())
}

/// 查询某门课程涉及的关联（任一端属于该课程）
#[tauri::command]
pub fn list_relation_details_by_course(
    app: AppHandle,
    course_id: String,
) -> Result<Vec<RelationDetail>, String> {
    let conn = get_connection(&app)?;
    list_details_by_course(&conn, course_id.trim())
}

/// 手动创建知识点关联
#[tauri::command]
pub fn create_relation(
    app: AppHandle,
    source_id: String,
    target_id: String,
    relation_type: String,
    description: String,
) -> Result<RelationDetail, String> {
    let conn = get_connection(&app)?;
    create_relation_record(
        &conn,
        source_id.trim(),
        target_id.trim(),
        relation_type.trim(),
        &description,
    )
}

/// 修改关联类型与描述（两端知识点保持不变）
#[tauri::command]
pub fn update_relation(
    app: AppHandle,
    relation_id: String,
    relation_type: String,
    description: String,
) -> Result<RelationDetail, String> {
    let conn = get_connection(&app)?;
    update_relation_record(
        &conn,
        relation_id.trim(),
        relation_type.trim(),
        &description,
    )
}

/// 删除关联关系
#[tauri::command]
pub fn delete_relation(app: AppHandle, relation_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    delete_relation_record(&conn, relation_id.trim())
}

/// 查询关联类型字典（含使用次数）
#[tauri::command]
pub fn list_relation_types(app: AppHandle) -> Result<Vec<RelationType>, String> {
    let conn = get_connection(&app)?;
    list_types(&conn)
}

/// 新增自定义关联类型
#[tauri::command]
pub fn create_relation_type(
    app: AppHandle,
    label: String,
    color: String,
    directed: bool,
    description: String,
) -> Result<RelationType, String> {
    let conn = get_connection(&app)?;
    create_type(&conn, &label, &color, directed, &description)
}

/// 修改关联类型（内置类型同样可以改名与换色）
#[tauri::command]
pub fn update_relation_type(
    app: AppHandle,
    type_id: String,
    label: String,
    color: String,
    directed: bool,
    description: String,
) -> Result<RelationType, String> {
    let conn = get_connection(&app)?;
    update_type(&conn, type_id.trim(), &label, &color, directed, &description)
}

/// 删除未被使用的自定义关联类型
#[tauri::command]
pub fn delete_relation_type(app: AppHandle, type_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    delete_type(&conn, type_id.trim())
}

/// 生成（并刷新）某个知识点的关联建议
#[tauri::command]
pub fn generate_relation_suggestions(
    app: AppHandle,
    knowledge_point_id: String,
    limit: Option<i64>,
) -> Result<Vec<RelationSuggestion>, String> {
    let conn = get_connection(&app)?;
    generate_suggestions(&conn, knowledge_point_id.trim(), limit)
}

/// 读取已保存的待处理关联建议（不重新计算）
#[tauri::command]
pub fn list_relation_suggestions(
    app: AppHandle,
    knowledge_point_id: String,
    limit: Option<i64>,
) -> Result<Vec<RelationSuggestion>, String> {
    let conn = get_connection(&app)?;
    let knowledge_point_id = knowledge_point_id.trim();
    ensure_point_exists(&conn, knowledge_point_id)?;

    let limit = limit.unwrap_or(8).clamp(1, MAX_SUGGESTIONS);
    load_pending_suggestions(&conn, knowledge_point_id, limit)
}

/// 接受关联建议：写入正式关联并把建议标记为已接受
#[tauri::command]
pub fn accept_relation_suggestion(
    app: AppHandle,
    suggestion_id: String,
) -> Result<RelationDetail, String> {
    let conn = get_connection(&app)?;
    accept_suggestion(&conn, suggestion_id.trim())
}

/// 拒绝关联建议：记录状态，之后不再重复推荐
#[tauri::command]
pub fn reject_relation_suggestion(app: AppHandle, suggestion_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    reject_suggestion(&conn, suggestion_id.trim())
}

/// 含这些虚词的二元组会被丢弃，避免「的线」这类无意义共同词进入推荐理由
const CN_STOP_CHARS: &[char] = &[
    '的', '了', '和', '与', '及', '或', '在', '是', '为', '这', '那', '也', '都', '就', '而',
    '其', '之', '以', '把', '被', '让', '使', '从', '到', '于', '并', '但', '很', '更', '最',
    '等',
];

/// 中文二元字组停用词：过滤没有区分度的常见组合
const CN_STOPWORDS: &[&str] = &[
    "一个", "一些", "一种", "我们", "这个", "这些", "那些", "可以", "以及", "其中", "因此",
    "所以", "如果", "但是", "并且", "或者", "例如", "通过", "使用", "用于", "进行", "需要",
    "包括", "主要", "重要", "不同", "相同", "相关", "知识", "识点", "学习", "内容", "问题",
    "方法", "时候", "以及", "的是", "不是", "没有", "就是", "还是", "因为", "由于", "然后",
    "同时", "对于", "关于", "根据", "实现", "提供", "支持", "表示", "保存", "包含", "属于",
];

/// 英文停用词
const EN_STOPWORDS: &[&str] = &[
    "the", "and", "for", "with", "that", "this", "from", "are", "was", "were", "not", "you",
    "can", "has", "have", "had", "its", "into", "such", "when", "then", "than", "they",
    "their", "will", "would", "should", "about", "which", "there", "these", "those", "also",
    "more", "most", "some", "any", "all", "one", "two", "use", "used", "using", "note",
    "notes", "with", "without", "between", "other", "each", "both", "only", "same",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        let conn = Connection::open_in_memory().expect("打开内存数据库失败");
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
            .expect("初始化 001 迁移失败");
        conn.execute_batch(include_str!("../../migrations/002_relations.sql"))
            .expect("初始化 002 迁移失败");
        conn
    }

    fn insert_course(conn: &Connection, id: &str, name: &str) {
        conn.execute(
            "INSERT INTO courses (id, name, description, created_at, updated_at)
             VALUES (?1, ?2, '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            params![id, name],
        )
        .expect("插入课程失败");
    }

    fn insert_point(
        conn: &Connection,
        id: &str,
        course_id: &str,
        title: &str,
        description: &str,
    ) {
        conn.execute(
            "INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
             VALUES (?1, ?2, NULL, ?3, ?4, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            params![id, course_id, title, description],
        )
        .expect("插入知识点失败");
    }

    fn insert_note(conn: &Connection, id: &str, point_id: &str, title: &str, content: &str) {
        conn.execute(
            "INSERT INTO notes
                (id, knowledge_point_id, title, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            params![id, point_id, title, content],
        )
        .expect("插入笔记失败");
    }

    fn seed_relations(conn: &Connection) {
        insert_course(conn, "c1", "数据结构");
        insert_course(conn, "c2", "操作系统");

        insert_point(conn, "k1", "c1", "双向链表", "每个节点保存前驱和后继指针");
        insert_point(conn, "k2", "c1", "单链表", "节点只保存后继指针");
        insert_point(conn, "k3", "c1", "进程调度", "操作系统按优先级分配 CPU 时间片");
        insert_point(conn, "k6", "c1", "循环链表", "尾节点的后继指针指回头节点");
        insert_point(conn, "k4", "c2", "线性表", "顺序存储与链式存储");

        insert_note(
            conn,
            "n1",
            "k1",
            "双向链表",
            "插入删除时需要调整前后指针，注意边界条件",
        );
        insert_note(
            conn,
            "n2",
            "k2",
            "单链表",
            "插入节点时需要修改前驱指针，删除同理",
        );
        insert_note(
            conn,
            "n3",
            "k3",
            "进程调度",
            "时间片轮转与上下文切换，按优先级分配 CPU 时间",
        );
        insert_note(
            conn,
            "n4",
            "k6",
            "循环链表",
            "遍历时需要判断指针是否回到头节点",
        );
    }

    #[test]
    fn creates_relation_with_description_and_detail() {
        let conn = test_connection();
        seed_relations(&conn);

        let detail = create_relation_record(
            &conn,
            "k1",
            "k2",
            "prerequisite",
            "  先掌握单链表再学习双向链表  ",
        )
        .expect("创建关联失败");

        assert_eq!(detail.relation.source_id, "k1");
        assert_eq!(detail.relation.target_id, "k2");
        assert_eq!(detail.relation.relation_type, "prerequisite");
        assert_eq!(detail.relation.description, "先掌握单链表再学习双向链表");
        assert_eq!(detail.source_title, "双向链表");
        assert_eq!(detail.target_title, "单链表");
        assert_eq!(detail.source_course_name, "数据结构");
        assert_eq!(detail.relation_label, "前置知识");
        assert_eq!(detail.relation_color, "#4f7cac");
        assert!(detail.directed);

        let all = list_all_relations(&conn).expect("读取关联失败");
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn rejects_self_duplicate_and_missing_references() {
        let conn = test_connection();
        seed_relations(&conn);

        assert!(create_relation_record(&conn, "k1", "k1", "related", "").is_err());
        assert!(create_relation_record(&conn, "k1", "missing", "related", "").is_err());
        assert!(create_relation_record(&conn, "missing", "k1", "related", "").is_err());
        assert!(create_relation_record(&conn, "k1", "k2", "unknown", "").is_err());

        create_relation_record(&conn, "k1", "k2", "related", "").expect("创建关联失败");
        assert!(
            create_relation_record(&conn, "k1", "k2", "related", "").is_err(),
            "同向重复关联应被拒绝"
        );
        assert!(
            create_relation_record(&conn, "k2", "k1", "related", "").is_err(),
            "无向类型的反向关联同样应被拒绝"
        );

        // 有向类型允许反向关系（互为前置）
        create_relation_record(&conn, "k1", "k2", "prerequisite", "").expect("创建前置关系失败");
        create_relation_record(&conn, "k2", "k1", "prerequisite", "").expect("创建反向前置失败");
    }

    #[test]
    fn updates_and_deletes_relation() {
        let conn = test_connection();
        seed_relations(&conn);

        let created = create_relation_record(&conn, "k1", "k2", "related", "旧描述")
            .expect("创建关联失败");

        let updated = update_relation_record(&conn, &created.relation.id, "contrast", " 对比两种链表 ")
            .expect("修改关联失败");
        assert_eq!(updated.relation.relation_type, "contrast");
        assert_eq!(updated.relation.description, "对比两种链表");
        assert_eq!(updated.relation_label, "对比辨析");
        assert!(!updated.directed);

        // 改成已有类型组合时仍要拦截重复
        create_relation_record(&conn, "k2", "k3", "related", "").expect("创建关联失败");
        create_relation_record(&conn, "k1", "k2", "prerequisite", "").expect("创建前置关系失败");
        assert!(update_relation_record(&conn, &created.relation.id, "prerequisite", "").is_err());

        delete_relation_record(&conn, &created.relation.id).expect("删除关联失败");
        let remaining = list_details(&conn).expect("读取关联失败");
        assert_eq!(remaining.len(), 2);
        assert!(!remaining.iter().any(|item| item.relation.id == created.relation.id));
        assert!(delete_relation_record(&conn, &created.relation.id).is_err());
    }

    #[test]
    fn filters_relation_details() {
        let conn = test_connection();
        seed_relations(&conn);

        create_relation_record(&conn, "k1", "k2", "related", "").expect("创建关联失败");
        create_relation_record(&conn, "k1", "k4", "related", "").expect("创建跨课程关联失败");

        assert_eq!(
            list_details_for_point(&conn, "k1").expect("读取关联失败").len(),
            2
        );
        assert_eq!(
            list_details_for_point(&conn, "k3").expect("读取关联失败").len(),
            0
        );
        assert!(list_details_for_point(&conn, "missing").is_err());

        let c1 = list_details_by_course(&conn, "c1").expect("读取课程关联失败");
        assert_eq!(c1.len(), 2, "任一端在课程 c1 的关联都应返回");

        let c2 = list_details_by_course(&conn, "c2").expect("读取课程关联失败");
        assert_eq!(c2.len(), 1);
        assert!(list_details_by_course(&conn, "missing").is_err());
    }

    #[test]
    fn manages_relation_types() {
        let conn = test_connection();
        seed_relations(&conn);

        let builtins = list_types(&conn).expect("读取关联类型失败");
        assert_eq!(builtins.len(), 5);
        assert!(builtins.iter().all(|item| item.is_builtin));
        assert!(builtins.iter().any(|item| item.id == "related"));

        let custom = create_type(&conn, "  易混淆  ", "#ABCDEF", false, "复习时重点对比")
            .expect("创建关联类型失败");
        assert_eq!(custom.label, "易混淆");
        assert_eq!(custom.color, "#abcdef");
        assert!(!custom.directed);
        assert_eq!(custom.usage_count, 0);

        assert!(create_type(&conn, "易混淆", "#123456", false, "").is_err());
        assert!(create_type(&conn, "", "#123456", false, "").is_err());
        assert!(create_type(&conn, "颜色错误", "red", false, "").is_err());

        create_relation_record(&conn, "k1", "k2", &custom.id, "").expect("创建关联失败");
        let used = load_type(&conn, &custom.id).expect("读取关联类型失败");
        assert_eq!(used.usage_count, 1);
        assert!(delete_type(&conn, &custom.id).is_err(), "类型被使用时不能删除");

        let renamed = update_type(&conn, &custom.id, "易混淆概念", "#123456", true, "")
            .expect("修改关联类型失败");
        assert_eq!(renamed.label, "易混淆概念");
        assert!(renamed.directed);

        assert!(delete_type(&conn, "related").is_err(), "内置类型不能删除");

        let relations = list_all_relations(&conn).expect("读取关联失败");
        delete_relation_record(&conn, &relations[0].id).expect("删除关联失败");
        delete_type(&conn, &custom.id).expect("删除未使用类型失败");
        assert_eq!(list_types(&conn).expect("读取关联类型失败").len(), 5);
    }

    #[test]
    fn generates_suggestions_from_title_description_and_notes() {
        let conn = test_connection();
        seed_relations(&conn);

        let suggestions =
            generate_suggestions(&conn, "k1", Some(5)).expect("生成关联建议失败");

        let linked = suggestions
            .iter()
            .find(|item| item.source_id == "k2" || item.target_id == "k2")
            .expect("名称与笔记相近的单链表应被推荐");

        assert!(linked.score >= MIN_SUGGESTION_SCORE);
        assert!(linked.score <= 1.0);
        assert_eq!(linked.relation_type, "related");
        assert_eq!(linked.relation_type_label, "相关知识");
        assert!(
            linked
                .reasons
                .iter()
                .any(|reason| reason.contains("链表") || reason.contains("指针")),
            "推荐理由应包含共同关键词，实际为 {:?}",
            linked.reasons
        );

        assert!(
            !suggestions
                .iter()
                .any(|item| item.source_id == "k3" || item.target_id == "k3"),
            "无关的进程调度不应进入建议列表"
        );

        // 重复生成不会产生重复记录
        let again = generate_suggestions(&conn, "k1", Some(5)).expect("再次生成关联建议失败");
        assert_eq!(again.len(), suggestions.len());
    }

    #[test]
    fn accepted_and_rejected_suggestions_are_remembered() {
        let conn = test_connection();
        seed_relations(&conn);

        let suggestions =
            generate_suggestions(&conn, "k1", Some(10)).expect("生成关联建议失败");
        let accepted = suggestions
            .iter()
            .find(|item| item.source_id == "k2" || item.target_id == "k2")
            .expect("缺少单链表建议")
            .clone();

        let detail = accept_suggestion(&conn, &accepted.id).expect("接受关联建议失败");
        assert_eq!(detail.relation.relation_type, "related");

        let after_accept =
            generate_suggestions(&conn, "k1", Some(10)).expect("重新生成关联建议失败");
        assert!(
            !after_accept.iter().any(|item| item.id == accepted.id),
            "已接受的建议不应再次出现"
        );
        assert!(accept_suggestion(&conn, &accepted.id).is_err(), "重复接受应失败");

        let rejected = after_accept
            .first()
            .expect("接受后仍应有其它建议（如 k3 或课程内其它知识点）")
            .clone();
        reject_suggestion(&conn, &rejected.id).expect("拒绝关联建议失败");

        let after_reject =
            generate_suggestions(&conn, "k1", Some(10)).expect("重新生成关联建议失败");
        assert!(
            !after_reject.iter().any(|item| item.id == rejected.id),
            "已拒绝的建议不应再次出现"
        );
        assert!(reject_suggestion(&conn, &rejected.id).is_err(), "重复拒绝应失败");
    }

    #[test]
    fn deleting_relation_allows_suggestion_again() {
        let conn = test_connection();
        seed_relations(&conn);

        let suggestions =
            generate_suggestions(&conn, "k1", Some(10)).expect("生成关联建议失败");
        let first = suggestions.first().expect("缺少建议").clone();

        let relation = accept_suggestion(&conn, &first.id).expect("接受关联建议失败");
        delete_relation_record(&conn, &relation.relation.id).expect("删除关联失败");

        let regenerated =
            generate_suggestions(&conn, "k1", Some(10)).expect("重新生成关联建议失败");
        assert!(
            regenerated
                .iter()
                .any(|item| item.source_id == first.source_id && item.target_id == first.target_id),
            "删除正式关联后应允许再次推荐"
        );
    }

    #[test]
    fn manual_relation_removes_existing_suggestion() {
        let conn = test_connection();
        seed_relations(&conn);

        let suggestions =
            generate_suggestions(&conn, "k1", Some(10)).expect("生成关联建议失败");
        let target = suggestions
            .iter()
            .find(|item| item.source_id == "k2" || item.target_id == "k2")
            .expect("缺少单链表建议")
            .clone();

        create_relation_record(&conn, "k1", "k2", "related", "手动关联")
            .expect("手动创建关联失败");

        let remaining = load_pending_suggestions(&conn, "k1", 10).expect("读取建议失败");
        assert!(
            !remaining.iter().any(|item| item.id == target.id),
            "已有正式关联后应清理对应建议"
        );
    }

    #[test]
    fn suggestions_stay_within_course_and_skip_related_points() {
        let conn = test_connection();
        seed_relations(&conn);

        // k4 与 k1 属于不同课程，即使标题相近也不应被推荐
        insert_point(&conn, "k5", "c1", "队列", "先进先出");
        let suggestions =
            generate_suggestions(&conn, "k5", Some(10)).expect("生成关联建议失败");
        assert!(!suggestions.iter().any(|item| {
            item.source_id == "k4" || item.target_id == "k4" || item.source_id == "k3"
        }));

        create_relation_record(&conn, "k5", "k3", "related", "").expect("创建关联失败");
        let refreshed =
            generate_suggestions(&conn, "k5", Some(10)).expect("生成关联建议失败");
        assert!(
            !refreshed
                .iter()
                .any(|item| item.source_id == "k3" || item.target_id == "k3"),
            "已有正式关联的知识点不应再被推荐"
        );
    }

    #[test]
    fn suggestion_round_trips_reasons_as_json() {
        let conn = test_connection();
        seed_relations(&conn);

        let suggestions =
            generate_suggestions(&conn, "k1", Some(3)).expect("生成关联建议失败");
        assert!(!suggestions.is_empty());

        let raw: String = conn
            .query_row(
                "SELECT reasons FROM relation_suggestions WHERE id = ?1",
                [&suggestions[0].id],
                |row| row.get(0),
            )
            .expect("读取建议理由失败");
        let parsed: Vec<String> = serde_json::from_str(&raw).expect("理由不是合法 JSON");
        assert!(!parsed.is_empty());
        assert_eq!(parsed, suggestions[0].reasons);
    }

    #[test]
    fn tokenizer_filters_stopwords_and_splits_bigrams() {
        let terms = tokenize("我们可以使用双向链表");
        assert!(terms.contains_key("双向"));
        assert!(terms.contains_key("链表"));
        assert!(!terms.contains_key("我们"), "停用词「我们」应被过滤");
        assert!(!terms.contains_key("可以"), "停用词「可以」应被过滤");
        assert!(!terms.contains_key("使用"), "停用词「使用」应被过滤");

        let english = tokenize("Binary Search Tree");
        assert!(english.contains_key("binary"));
        assert!(english.contains_key("search"));
        assert!(english.contains_key("tree"));

        assert!(tokenize("的 是 在").is_empty());
    }

    #[test]
    fn suggestion_requires_existing_point() {
        let conn = test_connection();
        seed_relations(&conn);

        assert!(generate_suggestions(&conn, "missing", None).is_err());
    }
}
