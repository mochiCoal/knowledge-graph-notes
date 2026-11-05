use rusqlite::{params, Connection, OptionalExtension, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, get_connection};
use crate::models::course::Course;

/// 课程查询统一使用的 SELECT 片段，附带知识点 / 笔记数量聚合。
const COURSE_SELECT: &str = "
SELECT c.id, c.name, c.description, c.created_at, c.updated_at,
       (SELECT COUNT(*) FROM knowledge_points kp WHERE kp.course_id = c.id),
       (SELECT COUNT(*) FROM notes n
          JOIN knowledge_points kp ON n.knowledge_point_id = kp.id
         WHERE kp.course_id = c.id)
FROM courses c
";

fn map_course(row: &Row<'_>) -> rusqlite::Result<Course> {
    Ok(Course {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        knowledge_point_count: row.get(5)?,
        note_count: row.get(6)?,
    })
}

/// 读取单个课程；不存在时返回 `None`。
fn find_course(conn: &Connection, course_id: &str) -> Result<Option<Course>, String> {
    let sql = format!("{COURSE_SELECT} WHERE c.id = ?1");
    conn.query_row(&sql, [course_id], map_course)
        .optional()
        .map_err(|e| db_error("读取课程", e))
}

/// 校验并规范化课程输入（名称必填且不超过 60 字，描述不超过 500 字）。
fn validate_course_input(name: &str, description: &str) -> Result<(String, String), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("课程名称不能为空".into());
    }
    if name.chars().count() > 60 {
        return Err("课程名称不能超过 60 个字符".into());
    }

    let description = description.trim();
    if description.chars().count() > 500 {
        return Err("课程描述不能超过 500 个字符".into());
    }

    Ok((name.to_string(), description.to_string()))
}

/// 同名课程校验：忽略 ASCII 大小写，`exclude_id` 用于编辑时排除自身。
fn ensure_name_available(
    conn: &Connection,
    name: &str,
    exclude_id: Option<&str>,
) -> Result<(), String> {
    let existing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM courses
             WHERE name = ?1 COLLATE NOCASE
               AND (?2 IS NULL OR id <> ?2)",
            params![name, exclude_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("校验课程名称", e))?;

    if existing > 0 {
        return Err("已存在同名课程，请换一个名称".into());
    }

    Ok(())
}

/// 查询全部课程（按创建时间升序）
#[tauri::command]
pub fn list_courses(app: AppHandle) -> Result<Vec<Course>, String> {
    let conn = get_connection(&app)?;
    let sql = format!("{COURSE_SELECT} ORDER BY c.created_at ASC");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| db_error("读取课程列表", e))?;

    stmt.query_map([], map_course)
        .map_err(|e| db_error("读取课程列表", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取课程列表", e))
}

/// 查询单个课程（课程详情页使用）
#[tauri::command]
pub fn get_course(app: AppHandle, course_id: String) -> Result<Course, String> {
    let conn = get_connection(&app)?;
    find_course(&conn, &course_id)?.ok_or_else(|| "课程不存在或已被删除".into())
}

/// 新增课程
#[tauri::command]
pub fn create_course(app: AppHandle, name: String, description: String) -> Result<Course, String> {
    let (name, description) = validate_course_input(&name, &description)?;

    let conn = get_connection(&app)?;
    ensure_name_available(&conn, &name, None)?;

    let now = chrono::Utc::now().to_rfc3339();
    let course = Course {
        id: Uuid::new_v4().to_string(),
        name,
        description,
        created_at: now.clone(),
        updated_at: now,
        knowledge_point_count: 0,
        note_count: 0,
    };

    conn.execute(
        "INSERT INTO courses (id, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            course.id,
            course.name,
            course.description,
            course.created_at,
            course.updated_at
        ],
    )
    .map_err(|e| db_error("创建课程", e))?;

    Ok(course)
}

/// 修改课程
#[tauri::command]
pub fn update_course(
    app: AppHandle,
    course_id: String,
    name: String,
    description: String,
) -> Result<Course, String> {
    let (name, description) = validate_course_input(&name, &description)?;

    let conn = get_connection(&app)?;
    let existing = find_course(&conn, &course_id)?
        .ok_or_else(|| "课程不存在或已被删除，请刷新后重试".to_string())?;

    // 名称未变化时不重复做同名校验，避免旧数据中已存在的同名课程无法修改描述。
    if existing.name != name {
        ensure_name_available(&conn, &name, Some(&course_id))?;
    }

    let updated_at = chrono::Utc::now().to_rfc3339();
    let changed = conn
        .execute(
            "UPDATE courses
                SET name = ?1, description = ?2, updated_at = ?3
              WHERE id = ?4",
            params![name, description, updated_at, course_id],
        )
        .map_err(|e| db_error("修改课程", e))?;

    if changed == 0 {
        return Err("课程不存在或已被删除，请刷新后重试".into());
    }

    find_course(&conn, &course_id)?.ok_or_else(|| "课程不存在或已被删除".into())
}

/// 删除课程
///
/// 数据库外键为 `ON DELETE CASCADE`：课程下的知识点、笔记、
/// 笔记标签关联与知识点关联会一并删除。
#[tauri::command]
pub fn delete_course(app: AppHandle, course_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    let changed = conn
        .execute("DELETE FROM courses WHERE id = ?1", [&course_id])
        .map_err(|e| db_error("删除课程", e))?;

    if changed == 0 {
        return Err("课程不存在或已被删除，请刷新后重试".into());
    }

    Ok(())
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

    fn insert_course(conn: &Connection, id: &str, name: &str) {
        conn.execute(
            "INSERT INTO courses (id, name, description, created_at, updated_at)
             VALUES (?1, ?2, '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            params![id, name],
        )
        .expect("插入课程失败");
    }

    fn insert_point_and_note(conn: &Connection, course_id: &str) {
        conn.execute(
            "INSERT INTO knowledge_points
                (id, course_id, parent_id, title, description, created_at, updated_at)
             VALUES ('kp-1', ?1, NULL, '知识点', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [course_id],
        )
        .expect("插入知识点失败");

        conn.execute(
            "INSERT INTO notes
                (id, knowledge_point_id, title, content, created_at, updated_at)
             VALUES ('note-1', 'kp-1', '笔记', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        )
        .expect("插入笔记失败");
    }

    #[test]
    fn course_query_returns_aggregated_counts() {
        let conn = test_connection();
        insert_course(&conn, "course-1", "数据结构");
        insert_point_and_note(&conn, "course-1");

        let course = find_course(&conn, "course-1")
            .expect("查询课程失败")
            .expect("课程应存在");
        assert_eq!(course.knowledge_point_count, 1);
        assert_eq!(course.note_count, 1);

        assert!(find_course(&conn, "missing")
            .expect("查询课程失败")
            .is_none());
    }

    #[test]
    fn deleting_course_cascades_points_and_notes() {
        let conn = test_connection();
        insert_course(&conn, "course-1", "数据结构");
        insert_point_and_note(&conn, "course-1");

        conn.execute("DELETE FROM courses WHERE id = 'course-1'", [])
            .expect("删除课程失败");

        let points: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge_points", [], |row| row.get(0))
            .expect("统计知识点失败");
        let notes: i64 = conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .expect("统计笔记失败");

        assert_eq!(points, 0);
        assert_eq!(notes, 0);
    }

    #[test]
    fn course_input_is_trimmed_and_validated() {
        assert!(validate_course_input("   ", "").is_err());
        assert!(validate_course_input(&"课".repeat(61), "").is_err());
        assert!(validate_course_input("课程", &"描".repeat(501)).is_err());

        let (name, description) =
            validate_course_input("  数据结构  ", "  课程描述  ").expect("校验应通过");
        assert_eq!(name, "数据结构");
        assert_eq!(description, "课程描述");
    }

    #[test]
    fn duplicate_name_is_rejected_but_self_is_excluded() {
        let conn = test_connection();
        insert_course(&conn, "course-1", "Data Structures");

        assert!(ensure_name_available(&conn, "data structures", None).is_err());
        assert!(ensure_name_available(&conn, "data structures", Some("course-1")).is_ok());
        assert!(ensure_name_available(&conn, "操作系统", None).is_ok());
    }
}
