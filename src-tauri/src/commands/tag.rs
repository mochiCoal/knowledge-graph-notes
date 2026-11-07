use rusqlite::{params, Connection, OptionalExtension, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, escape_like, get_connection};
use crate::models::tag::{Tag, TagSummary};

/// 标签名称最大长度（与前端输入框限制一致）
const TAG_NAME_MAX: usize = 30;

fn map_tag(row: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
    })
}

fn map_tag_summary(row: &Row<'_>) -> rusqlite::Result<TagSummary> {
    Ok(TagSummary {
        id: row.get(0)?,
        name: row.get(1)?,
        note_count: row.get(2)?,
    })
}

/// 校验并规范化标签名称（去除首尾空白，不能为空且不超过 30 字）
fn validate_tag_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("标签名称不能为空".into());
    }
    if name.chars().count() > TAG_NAME_MAX {
        return Err(format!("标签名称不能超过 {TAG_NAME_MAX} 个字符"));
    }

    Ok(name.to_string())
}

fn find_tag(conn: &Connection, tag_id: &str) -> Result<Option<Tag>, String> {
    conn.query_row("SELECT id, name FROM tags WHERE id = ?1", [tag_id], map_tag)
        .optional()
        .map_err(|e| db_error("读取标签", e))
}

fn require_tag(conn: &Connection, tag_id: &str) -> Result<Tag, String> {
    find_tag(conn, tag_id)?.ok_or_else(|| "标签不存在或已被删除，请刷新后重试".to_string())
}

/// 忽略大小写按名称查找标签
fn find_tag_by_name(conn: &Connection, name: &str) -> Result<Option<Tag>, String> {
    conn.query_row(
        "SELECT id, name FROM tags WHERE name = ?1 COLLATE NOCASE",
        [name],
        map_tag,
    )
    .optional()
    .map_err(|e| db_error("读取标签", e))
}

/// 同名标签校验（忽略 ASCII 大小写），`exclude_id` 用于重命名时排除自身
fn ensure_name_available(
    conn: &Connection,
    name: &str,
    exclude_id: Option<&str>,
) -> Result<(), String> {
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tags
             WHERE name = ?1 COLLATE NOCASE
               AND (?2 IS NULL OR id <> ?2)",
            params![name, exclude_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("校验标签名称", e))?;

    if existing > 0 {
        return Err("已存在同名标签，请换一个名称".into());
    }

    Ok(())
}

fn list_all(conn: &Connection) -> Result<Vec<Tag>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name FROM tags ORDER BY name COLLATE NOCASE ASC")
        .map_err(|e| db_error("读取标签列表", e))?;

    stmt.query_map([], map_tag)
        .map_err(|e| db_error("读取标签列表", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取标签列表", e))
}

/// 查询标签及其关联笔记数量；`keyword` 为空时返回全部标签
fn list_tag_summaries(
    conn: &Connection,
    keyword: Option<&str>,
) -> Result<Vec<TagSummary>, String> {
    let base = "SELECT t.id, t.name,
                       (SELECT COUNT(*) FROM note_tags nt WHERE nt.tag_id = t.id)
                FROM tags t";

    let sql = match keyword.map(str::trim).filter(|keyword| !keyword.is_empty()) {
        None => format!("{base} ORDER BY t.name COLLATE NOCASE ASC"),
        Some(_) => format!(
            "{base}
             WHERE t.name LIKE ?1 ESCAPE '\\'
             ORDER BY t.name COLLATE NOCASE ASC"
        ),
    };

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取标签列表", e))?;

    let rows = match keyword.map(str::trim).filter(|keyword| !keyword.is_empty()) {
        None => stmt.query_map([], map_tag_summary),
        Some(keyword) => {
            let pattern = format!("%{}%", escape_like(keyword));
            stmt.query_map([pattern], map_tag_summary)
        }
    }
    .map_err(|e| db_error("读取标签列表", e))?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取标签列表", e))
}

/// 创建标签；同名标签直接复用已有记录
fn insert_or_reuse_tag(conn: &Connection, name: &str) -> Result<Tag, String> {
    let name = validate_tag_name(name)?;

    if let Some(existing) = find_tag_by_name(conn, &name)? {
        return Ok(existing);
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tags (id, name) VALUES (?1, ?2)",
        params![id, name],
    )
    .map_err(|e| db_error("创建标签", e))?;

    require_tag(conn, &id)
}

/// 重命名标签
fn rename_tag(conn: &Connection, tag_id: &str, name: &str) -> Result<Tag, String> {
    let existing = require_tag(conn, tag_id)?;
    let name = validate_tag_name(name)?;

    if existing.name == name {
        return Ok(existing);
    }

    ensure_name_available(conn, &name, Some(tag_id))?;

    conn.execute("UPDATE tags SET name = ?1 WHERE id = ?2", params![name, tag_id])
        .map_err(|e| db_error("修改标签", e))?;

    require_tag(conn, tag_id)
}

/// 删除标签，返回被解除关联的笔记数量（笔记本身不受影响）
fn remove_tag(conn: &Connection, tag_id: &str) -> Result<i64, String> {
    require_tag(conn, tag_id)?;

    let linked: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note_tags WHERE tag_id = ?1",
            [tag_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("统计标签笔记", e))?;

    conn.execute("DELETE FROM tags WHERE id = ?1", [tag_id])
        .map_err(|e| db_error("删除标签", e))?;

    Ok(linked)
}

fn tags_of_note(conn: &Connection, note_id: &str) -> Result<Vec<Tag>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name
             FROM tags t
             INNER JOIN note_tags nt ON nt.tag_id = t.id
             WHERE nt.note_id = ?1
             ORDER BY t.name COLLATE NOCASE ASC",
        )
        .map_err(|e| db_error("读取笔记标签", e))?;

    stmt.query_map([note_id], map_tag)
        .map_err(|e| db_error("读取笔记标签", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取笔记标签", e))
}

fn attach_tag(conn: &Connection, note_id: &str, tag_id: &str) -> Result<(), String> {
    let note_exists: i64 = conn
        .query_row("SELECT COUNT(*) FROM notes WHERE id = ?1", [note_id], |row| {
            row.get(0)
        })
        .map_err(|e| db_error("读取笔记", e))?;
    if note_exists == 0 {
        return Err("笔记不存在或已被删除，请刷新后重试".into());
    }

    require_tag(conn, tag_id)?;

    conn.execute(
        "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
        params![note_id, tag_id],
    )
    .map_err(|e| db_error("添加标签", e))?;

    Ok(())
}

fn detach_tag(conn: &Connection, note_id: &str, tag_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2",
        params![note_id, tag_id],
    )
    .map_err(|e| db_error("移除标签", e))?;

    Ok(())
}

/// 查询全部标签
#[tauri::command]
pub fn list_tags(app: AppHandle) -> Result<Vec<Tag>, String> {
    let conn = get_connection(&app)?;
    list_all(&conn)
}

/// 查询标签管理列表（含关联笔记数量）
#[tauri::command]
pub fn list_tag_overview(app: AppHandle) -> Result<Vec<TagSummary>, String> {
    let conn = get_connection(&app)?;
    list_tag_summaries(&conn, None)
}

/// 按名称搜索标签（空关键字等价于查询全部）
#[tauri::command]
pub fn search_tags(app: AppHandle, keyword: String) -> Result<Vec<TagSummary>, String> {
    let conn = get_connection(&app)?;
    list_tag_summaries(&conn, Some(&keyword))
}

/// 创建标签（同名标签直接复用已有记录）
#[tauri::command]
pub fn create_tag(app: AppHandle, name: String) -> Result<Tag, String> {
    let conn = get_connection(&app)?;
    insert_or_reuse_tag(&conn, &name)
}

/// 重命名标签
#[tauri::command]
pub fn update_tag(app: AppHandle, tag_id: String, name: String) -> Result<Tag, String> {
    let conn = get_connection(&app)?;
    rename_tag(&conn, &tag_id, &name)
}

/// 删除标签，返回解除关联的笔记数量
#[tauri::command]
pub fn delete_tag(app: AppHandle, tag_id: String) -> Result<i64, String> {
    let conn = get_connection(&app)?;
    remove_tag(&conn, &tag_id)
}

/// 查询某条笔记的标签
#[tauri::command]
pub fn list_note_tags(app: AppHandle, note_id: String) -> Result<Vec<Tag>, String> {
    let conn = get_connection(&app)?;
    tags_of_note(&conn, &note_id)
}

/// 为笔记添加标签
#[tauri::command]
pub fn attach_note_tag(app: AppHandle, note_id: String, tag_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    attach_tag(&conn, &note_id, &tag_id)
}

/// 移除笔记标签
#[tauri::command]
pub fn detach_note_tag(app: AppHandle, note_id: String, tag_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    detach_tag(&conn, &note_id, &tag_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_connection() -> Connection {
        let conn = Connection::open_in_memory().expect("打开内存数据库失败");
        conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
            .expect("初始化数据库失败");
        conn
    }

    fn seed(conn: &Connection) {
        conn.execute(
            "INSERT INTO courses (id, name, description, created_at, updated_at)
             VALUES ('c1', '数据结构', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        )
        .expect("插入课程失败");

        for (id, title) in [("kp1", "链表"), ("kp2", "排序算法")] {
            conn.execute(
                "INSERT INTO knowledge_points
                    (id, course_id, parent_id, title, description, created_at, updated_at)
                 VALUES (?1, 'c1', NULL, ?2, '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                params![id, title],
            )
            .expect("插入知识点失败");
        }

        for (id, point_id, title) in [
            ("n1", "kp1", "链表笔记"),
            ("n2", "kp2", "快速排序"),
        ] {
            conn.execute(
                "INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
                 VALUES (?1, ?2, ?3, '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                params![id, point_id, title],
            )
            .expect("插入笔记失败");
        }

        for (id, name) in [("t1", "algo"), ("t2", "基础"), ("t3", "未使用")] {
            conn.execute("INSERT INTO tags (id, name) VALUES (?1, ?2)", params![id, name])
                .expect("插入标签失败");
        }

        for (note_id, tag_id) in [("n1", "t1"), ("n1", "t2"), ("n2", "t1")] {
            conn.execute(
                "INSERT INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                params![note_id, tag_id],
            )
            .expect("插入笔记标签失败");
        }
    }

    #[test]
    fn tag_name_is_trimmed_and_validated() {
        assert_eq!(validate_tag_name("  重点  ").expect("校验应通过"), "重点");
        assert!(validate_tag_name("   ").is_err());
        assert!(validate_tag_name(&"标".repeat(31)).is_err());
        assert!(validate_tag_name(&"标".repeat(30)).is_ok());
    }

    #[test]
    fn create_tag_reuses_existing_name_ignoring_case() {
        let conn = test_connection();
        seed(&conn);

        let created = insert_or_reuse_tag(&conn, "  重点  ").expect("创建标签失败");
        assert_eq!(created.name, "重点");

        let same = insert_or_reuse_tag(&conn, "重点").expect("重复创建应返回已有标签");
        assert_eq!(same.id, created.id);

        let case_insensitive = insert_or_reuse_tag(&conn, "ALGO").expect("大小写视为同名");
        assert_eq!(case_insensitive.id, "t1");
        assert_eq!(case_insensitive.name, "algo");
    }

    #[test]
    fn update_tag_renames_and_rejects_duplicates() {
        let conn = test_connection();
        seed(&conn);

        let renamed = rename_tag(&conn, "t3", "  复习  ").expect("重命名失败");
        assert_eq!(renamed.name, "复习");

        // 名称未变化时直接返回
        assert_eq!(rename_tag(&conn, "t3", "复习").expect("重命名失败").name, "复习");

        // 与其它标签重名（忽略大小写）
        assert!(rename_tag(&conn, "t3", "ALGO").is_err());
        assert!(rename_tag(&conn, "t3", "基础").is_err());
        assert!(rename_tag(&conn, "missing", "新标签").is_err());
        assert!(rename_tag(&conn, "t3", "  ").is_err());
    }

    #[test]
    fn tag_overview_counts_notes_and_supports_search() {
        let conn = test_connection();
        seed(&conn);

        let overview = list_tag_summaries(&conn, None).expect("读取标签列表失败");
        assert_eq!(
            overview
                .iter()
                .map(|tag| (tag.id.as_str(), tag.note_count))
                .collect::<Vec<_>>(),
            vec![("t1", 2), ("t2", 1), ("t3", 0)]
        );

        let by_name = list_tag_summaries(&conn, Some("基")).expect("搜索标签失败");
        assert_eq!(by_name.len(), 1);
        assert_eq!(by_name[0].id, "t2");

        let case_insensitive = list_tag_summaries(&conn, Some("ALG")).expect("搜索标签失败");
        assert_eq!(case_insensitive.len(), 1);
        assert_eq!(case_insensitive[0].id, "t1");

        assert!(list_tag_summaries(&conn, Some("zzz")).expect("搜索标签失败").is_empty());
        // 通配符按字面量处理
        assert!(list_tag_summaries(&conn, Some("%")).expect("搜索标签失败").is_empty());
        assert_eq!(list_tag_summaries(&conn, Some("   ")).expect("搜索标签失败").len(), 3);
    }

    #[test]
    fn delete_tag_removes_links_but_keeps_notes() {
        let conn = test_connection();
        seed(&conn);

        let linked = remove_tag(&conn, "t1").expect("删除标签失败");
        assert_eq!(linked, 2);

        assert!(find_tag(&conn, "t1").expect("读取标签失败").is_none());

        let links: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM note_tags WHERE tag_id = 't1'",
                [],
                |row| row.get(0),
            )
            .expect("统计笔记标签失败");
        assert_eq!(links, 0);

        let notes: i64 = conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .expect("统计笔记失败");
        assert_eq!(notes, 2);

        assert!(remove_tag(&conn, "missing").is_err());
    }

    #[test]
    fn attach_and_detach_validate_rows() {
        let conn = test_connection();
        seed(&conn);

        attach_tag(&conn, "n2", "t2").expect("添加标签失败");
        // 重复添加不报错
        attach_tag(&conn, "n2", "t2").expect("重复添加标签失败");

        let tags = tags_of_note(&conn, "n2").expect("读取笔记标签失败");
        assert_eq!(
            tags.iter().map(|tag| tag.name.as_str()).collect::<Vec<_>>(),
            vec!["algo", "基础"]
        );

        assert!(attach_tag(&conn, "missing", "t2").is_err());
        assert!(attach_tag(&conn, "n2", "missing").is_err());

        detach_tag(&conn, "n2", "t2").expect("移除标签失败");
        assert_eq!(tags_of_note(&conn, "n2").expect("读取笔记标签失败").len(), 1);
    }
}
