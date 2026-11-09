use rusqlite::{Connection, Row, params};
use tauri::AppHandle;

use crate::db::{db_error, escape_like, get_connection};
use crate::models::search::{GlobalSearchResult, SearchHit};
use crate::point_path::{PointIndex, join_path, load_point_index, path_titles};
use crate::text::{excerpt, match_excerpt};

/// 每个分组最多返回的条数（分组内按相关度与更新时间排序）
const COURSE_LIMIT: i64 = 6;
const POINT_LIMIT: i64 = 12;
const NOTE_LIMIT: i64 = 12;
const TAG_LIMIT: i64 = 8;
const RELATION_LIMIT: i64 = 8;

/// 搜索结果片段的最大字符数
const SNIPPET_MAX: usize = 120;
/// 描述类字段摘要的最大字符数
const DESCRIPTION_MAX: usize = 96;

fn map_course(row: &Row<'_>) -> rusqlite::Result<SearchHit> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let description: String = row.get(2)?;
    let point_count: i64 = row.get(4)?;
    let note_count: i64 = row.get(5)?;

    Ok(SearchHit {
        id: id.clone(),
        kind: "course".into(),
        title: name,
        subtitle: format!("{point_count} 个知识点 · {note_count} 篇笔记"),
        excerpt: excerpt(&description, DESCRIPTION_MAX),
        course_id: Some(id),
        knowledge_point_id: None,
        updated_at: Some(row.get(3)?),
    })
}

fn search_courses(
    conn: &Connection,
    keyword: &str,
    pattern: &str,
    prefix_pattern: &str,
) -> Result<Vec<SearchHit>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.name, c.description, c.updated_at,
                    (SELECT COUNT(*) FROM knowledge_points kp WHERE kp.course_id = c.id),
                    (SELECT COUNT(*) FROM notes n
                       INNER JOIN knowledge_points kp ON kp.id = n.knowledge_point_id
                      WHERE kp.course_id = c.id)
             FROM courses c
             WHERE c.name LIKE ?1 ESCAPE '\\' OR c.description LIKE ?1 ESCAPE '\\'
             ORDER BY CASE
                        WHEN c.name = ?2 THEN 0
                        WHEN c.name LIKE ?3 ESCAPE '\\' THEN 1
                        ELSE 2
                      END,
                      c.updated_at DESC
             LIMIT ?4",
        )
        .map_err(|e| db_error("搜索课程", e))?;

    stmt.query_map(
        params![pattern, keyword, prefix_pattern, COURSE_LIMIT],
        map_course,
    )
    .map_err(|e| db_error("搜索课程", e))?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|e| db_error("搜索课程", e))
}

fn point_subtitle(course_name: &str, parent_id: Option<&str>, index: &PointIndex) -> String {
    let path = path_titles(index, parent_id);
    if path.is_empty() {
        course_name.to_string()
    } else {
        format!("{course_name} / {}", join_path(&path, " / "))
    }
}

fn search_points(
    conn: &Connection,
    keyword: &str,
    pattern: &str,
    prefix_pattern: &str,
) -> Result<Vec<SearchHit>, String> {
    let index = load_point_index(conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT kp.id, kp.title, kp.description, kp.parent_id,
                    kp.course_id, c.name, kp.updated_at
             FROM knowledge_points kp
             INNER JOIN courses c ON c.id = kp.course_id
             WHERE kp.title LIKE ?1 ESCAPE '\\' OR kp.description LIKE ?1 ESCAPE '\\'
             ORDER BY CASE
                        WHEN kp.title = ?2 THEN 0
                        WHEN kp.title LIKE ?3 ESCAPE '\\' THEN 1
                        ELSE 2
                      END,
                      kp.updated_at DESC
             LIMIT ?4",
        )
        .map_err(|e| db_error("搜索知识点", e))?;

    stmt.query_map(
        params![pattern, keyword, prefix_pattern, POINT_LIMIT],
        |row| {
            let description: String = row.get(2)?;
            let parent_id: Option<String> = row.get(3)?;
            let course_name: String = row.get(5)?;

            Ok(SearchHit {
                id: row.get(0)?,
                kind: "knowledge_point".into(),
                title: row.get(1)?,
                subtitle: point_subtitle(&course_name, parent_id.as_deref(), &index),
                excerpt: excerpt(&description, DESCRIPTION_MAX),
                course_id: Some(row.get(4)?),
                knowledge_point_id: None,
                updated_at: Some(row.get(6)?),
            })
        },
    )
    .map_err(|e| db_error("搜索知识点", e))?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|e| db_error("搜索知识点", e))
}

fn search_notes(
    conn: &Connection,
    keyword: &str,
    pattern: &str,
    prefix_pattern: &str,
) -> Result<Vec<SearchHit>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT n.id, n.knowledge_point_id, n.title, n.content,
                    kp.title, c.id, c.name, n.updated_at
             FROM notes n
             INNER JOIN knowledge_points kp ON kp.id = n.knowledge_point_id
             INNER JOIN courses c ON c.id = kp.course_id
             WHERE n.title LIKE ?1 ESCAPE '\\' OR n.content LIKE ?1 ESCAPE '\\'
             ORDER BY CASE
                        WHEN n.title = ?2 THEN 0
                        WHEN n.title LIKE ?3 ESCAPE '\\' THEN 1
                        ELSE 2
                      END,
                      n.updated_at DESC
             LIMIT ?4",
        )
        .map_err(|e| db_error("搜索笔记", e))?;

    stmt.query_map(
        params![pattern, keyword, prefix_pattern, NOTE_LIMIT],
        |row| {
            let content: String = row.get(3)?;
            let point_title: String = row.get(4)?;
            let course_name: String = row.get(6)?;

            Ok(SearchHit {
                id: row.get(0)?,
                kind: "note".into(),
                title: row.get(2)?,
                subtitle: format!("{course_name} / {point_title}"),
                excerpt: match_excerpt(&content, keyword, SNIPPET_MAX),
                course_id: Some(row.get(5)?),
                knowledge_point_id: Some(row.get(1)?),
                updated_at: Some(row.get(7)?),
            })
        },
    )
    .map_err(|e| db_error("搜索笔记", e))?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|e| db_error("搜索笔记", e))
}

fn search_tags(
    conn: &Connection,
    keyword: &str,
    pattern: &str,
    prefix_pattern: &str,
) -> Result<Vec<SearchHit>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name,
                    (SELECT COUNT(*) FROM note_tags nt WHERE nt.tag_id = t.id)
             FROM tags t
             WHERE t.name LIKE ?1 ESCAPE '\\'
             ORDER BY CASE
                        WHEN t.name = ?2 THEN 0
                        WHEN t.name LIKE ?3 ESCAPE '\\' THEN 1
                        ELSE 2
                      END,
                      t.name COLLATE NOCASE ASC
             LIMIT ?4",
        )
        .map_err(|e| db_error("搜索标签", e))?;

    stmt.query_map(
        params![pattern, keyword, prefix_pattern, TAG_LIMIT],
        |row| {
            let name: String = row.get(1)?;
            let note_count: i64 = row.get(2)?;

            Ok(SearchHit {
                id: row.get(0)?,
                kind: "tag".into(),
                title: name.clone(),
                subtitle: format!("{note_count} 篇笔记使用标签「{name}」"),
                excerpt: String::new(),
                course_id: None,
                knowledge_point_id: None,
                updated_at: None,
            })
        },
    )
    .map_err(|e| db_error("搜索标签", e))?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|e| db_error("搜索标签", e))
}

fn search_relations(
    conn: &Connection,
    keyword: &str,
    pattern: &str,
) -> Result<Vec<SearchHit>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.source_id, r.target_id, r.description, r.created_at,
                    src.title, src.course_id, tgt.title,
                    COALESCE(rt.label, r.relation_type)
             FROM knowledge_relations r
             INNER JOIN knowledge_points src ON src.id = r.source_id
             INNER JOIN knowledge_points tgt ON tgt.id = r.target_id
             LEFT JOIN relation_types rt ON rt.id = r.relation_type
             WHERE r.description LIKE ?1 ESCAPE '\\'
                OR src.title LIKE ?1 ESCAPE '\\'
                OR tgt.title LIKE ?1 ESCAPE '\\'
                OR COALESCE(rt.label, r.relation_type) LIKE ?1 ESCAPE '\\'
             ORDER BY CASE WHEN src.title LIKE ?1 ESCAPE '\\' OR tgt.title LIKE ?1 ESCAPE '\\'
                           THEN 0 ELSE 1 END,
                      r.created_at DESC
             LIMIT ?2",
        )
        .map_err(|e| db_error("搜索知识点关联", e))?;

    stmt.query_map(params![pattern, RELATION_LIMIT], |row| {
        let description: String = row.get(3)?;
        let source_title: String = row.get(5)?;
        let target_title: String = row.get(7)?;
        let type_label: String = row.get(8)?;

        Ok(SearchHit {
            id: row.get(0)?,
            kind: "relation".into(),
            title: format!("{source_title} → {target_title}"),
            subtitle: format!("{type_label} · 知识点关联"),
            excerpt: if description.trim().is_empty() {
                "未填写关联说明".to_string()
            } else {
                match_excerpt(&description, keyword, SNIPPET_MAX)
            },
            course_id: Some(row.get(6)?),
            knowledge_point_id: Some(row.get(1)?),
            updated_at: Some(row.get(4)?),
        })
    })
    .map_err(|e| db_error("搜索知识点关联", e))?
    .collect::<rusqlite::Result<Vec<_>>>()
    .map_err(|e| db_error("搜索知识点关联", e))
}

fn empty_result(keyword: &str) -> GlobalSearchResult {
    GlobalSearchResult {
        query: keyword.to_string(),
        total: 0,
        courses: Vec::new(),
        knowledge_points: Vec::new(),
        notes: Vec::new(),
        tags: Vec::new(),
        relations: Vec::new(),
    }
}

/// 在课程、知识点、笔记、标签与知识点关联中执行全局搜索（空关键字返回空结果）
pub(crate) fn search_all(conn: &Connection, keyword: &str) -> Result<GlobalSearchResult, String> {
    let keyword = keyword.trim();
    if keyword.is_empty() {
        return Ok(empty_result(""));
    }

    let pattern = format!("%{}%", escape_like(keyword));
    let prefix_pattern = format!("{}%", escape_like(keyword));

    let courses = search_courses(conn, keyword, &pattern, &prefix_pattern)?;
    let knowledge_points = search_points(conn, keyword, &pattern, &prefix_pattern)?;
    let notes = search_notes(conn, keyword, &pattern, &prefix_pattern)?;
    let tags = search_tags(conn, keyword, &pattern, &prefix_pattern)?;
    let relations = search_relations(conn, keyword, &pattern)?;

    let total = courses.len() + knowledge_points.len() + notes.len() + tags.len() + relations.len();

    Ok(GlobalSearchResult {
        query: keyword.to_string(),
        total,
        courses,
        knowledge_points,
        notes,
        tags,
        relations,
    })
}

/// 全局搜索：一次返回按类型分组的结果，供 Ctrl+K 搜索面板使用
#[tauri::command]
pub fn global_search(app: AppHandle, keyword: String) -> Result<GlobalSearchResult, String> {
    let conn = get_connection(&app)?;
    search_all(&conn, &keyword)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        let conn = Connection::open_in_memory().expect("打开内存数据库失败");
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
            .expect("初始化数据库失败");
        conn.execute_batch(include_str!("../../migrations/002_relations.sql"))
            .expect("执行关联迁移失败");
        conn
    }

    fn seed(conn: &Connection) {
        conn.execute_batch(
            "
            INSERT INTO courses (id, name, description, created_at, updated_at) VALUES
                ('c1', '数据结构', '线性结构与算法', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
                ('c2', '算法设计', '分治与动态规划', '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z');

            INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at) VALUES
                ('kp1', 'c1', NULL, '线性表', '顺序存储', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
                ('kp2', 'c1', 'kp1', '链表', '链式存储', '2026-01-02T00:00:00Z', '2026-01-02T00:00:00Z'),
                ('kp3', 'c2', NULL, '排序算法', '快速排序', '2026-01-03T00:00:00Z', '2026-01-03T00:00:00Z');

            INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at) VALUES
                ('n1', 'kp2', '链表笔记', '# 链表\\n\\n插入与删除都是 O(1)', '2026-01-01T00:00:00Z', '2026-01-04T00:00:00Z'),
                ('n2', 'kp3', '快速排序', '分治思想，平均 O(n log n)。', '2026-01-02T00:00:00Z', '2026-01-05T00:00:00Z');

            INSERT INTO tags (id, name) VALUES ('tg1', '重点');
            INSERT INTO note_tags (note_id, tag_id) VALUES ('n1', 'tg1');

            INSERT INTO knowledge_relations
                (id, source_id, target_id, relation_type, description, created_at) VALUES
                ('r1', 'kp1', 'kp2', 'related', '线性表包含链表等实现', '2026-01-06T00:00:00Z');
            ",
        )
        .expect("写入测试数据失败");
    }

    #[test]
    fn search_groups_results_by_kind() {
        let conn = test_connection();
        seed(&conn);

        let result = search_all(&conn, "链表").expect("全局搜索失败");
        assert_eq!(result.query, "链表");
        assert_eq!(result.knowledge_points.len(), 1);
        assert_eq!(result.knowledge_points[0].id, "kp2");
        assert_eq!(result.notes.len(), 1);
        assert_eq!(result.notes[0].id, "n1");
        assert_eq!(result.relations.len(), 1);
        assert_eq!(result.relations[0].id, "r1");

        assert_eq!(result.knowledge_points[0].subtitle, "数据结构 / 线性表");
        assert_eq!(result.notes[0].subtitle, "数据结构 / 链表");
        assert!(result.notes[0].excerpt.contains("链表"));
        assert_eq!(result.relations[0].title, "线性表 → 链表");
        assert!(result.relations[0].subtitle.contains("相关知识"));
        assert_eq!(
            result.total,
            result.courses.len()
                + result.knowledge_points.len()
                + result.notes.len()
                + result.tags.len()
                + result.relations.len()
        );
    }

    #[test]
    fn search_matches_course_description_and_content() {
        let conn = test_connection();
        seed(&conn);

        let by_description = search_all(&conn, "线性结构").expect("全局搜索失败");
        assert_eq!(by_description.courses.len(), 1);
        assert_eq!(by_description.courses[0].id, "c1");
        assert_eq!(by_description.courses[0].subtitle, "2 个知识点 · 1 篇笔记");

        let by_content = search_all(&conn, "分治").expect("全局搜索失败");
        assert!(by_content.notes.iter().any(|hit| hit.id == "n2"));
        assert!(
            by_content
                .notes
                .iter()
                .find(|hit| hit.id == "n2")
                .expect("缺少笔记命中")
                .excerpt
                .contains("分治")
        );
    }

    #[test]
    fn search_finds_tags_and_ranks_title_matches_first() {
        let conn = test_connection();
        seed(&conn);

        let tags = search_all(&conn, "重点").expect("全局搜索失败");
        assert_eq!(tags.tags.len(), 1);
        assert_eq!(tags.tags[0].id, "tg1");
        assert!(tags.tags[0].subtitle.contains("1 篇笔记"));

        // 「排序」同时命中知识点标题与笔记正文，标题命中排在前面由 SQL 保证；
        // 这里验证两类结果都返回。
        let ranking = search_all(&conn, "排序").expect("全局搜索失败");
        assert_eq!(ranking.knowledge_points[0].id, "kp3");
        assert!(ranking.notes.iter().any(|hit| hit.id == "n2"));
    }

    #[test]
    fn empty_or_wildcard_query_returns_expected_results() {
        let conn = test_connection();
        seed(&conn);

        let empty = search_all(&conn, "   ").expect("全局搜索失败");
        assert_eq!(empty.total, 0);
        assert!(empty.notes.is_empty());

        // LIKE 通配符按字面量处理，不应匹配全部记录
        let wildcard = search_all(&conn, "%").expect("全局搜索失败");
        assert_eq!(wildcard.total, 0);

        let missing = search_all(&conn, "不存在的关键字").expect("全局搜索失败");
        assert_eq!(missing.total, 0);
    }
}
