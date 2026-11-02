use rusqlite::{params, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, get_connection};
use crate::models::knowledge_point::KnowledgePoint;

fn map_knowledge_point(row: &Row<'_>) -> rusqlite::Result<KnowledgePoint> {
    Ok(KnowledgePoint {
        id: row.get(0)?,
        course_id: row.get(1)?,
        parent_id: row.get(2)?,
        title: row.get(3)?,
        description: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

/// 查询全部知识点（按创建时间升序，前端按 course_id / parent_id 组装树）
#[tauri::command]
pub fn list_knowledge_points(app: AppHandle) -> Result<Vec<KnowledgePoint>, String> {
    let conn = get_connection(&app)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, course_id, parent_id, title, description, created_at, updated_at
             FROM knowledge_points
             ORDER BY created_at ASC",
        )
        .map_err(|e| db_error("读取知识点列表", e))?;

    stmt.query_map([], map_knowledge_point)
        .map_err(|e| db_error("读取知识点列表", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取知识点列表", e))
}

/// 新增知识点，可指定父知识点形成层级
#[tauri::command]
pub fn create_knowledge_point(
    app: AppHandle,
    course_id: String,
    parent_id: Option<String>,
    title: String,
    description: String,
) -> Result<KnowledgePoint, String> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err("知识点名称不能为空".into());
    }
    if title.chars().count() > 80 {
        return Err("知识点名称不能超过 80 个字符".into());
    }

    let description = description.trim().to_string();
    if description.chars().count() > 1000 {
        return Err("知识点描述不能超过 1000 个字符".into());
    }

    let conn = get_connection(&app)?;

    let course_exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM courses WHERE id = ?1",
            [&course_id],
            |row| row.get(0),
        )
        .map_err(|e| db_error("读取课程", e))?;
    if course_exists == 0 {
        return Err("课程不存在，请刷新后重试".into());
    }

    if let Some(parent) = parent_id.as_deref() {
        let parent_in_course: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_points WHERE id = ?1 AND course_id = ?2",
                params![parent, course_id],
                |row| row.get(0),
            )
            .map_err(|e| db_error("读取知识点", e))?;
        if parent_in_course == 0 {
            return Err("父知识点不存在或不属于该课程".into());
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    let point = KnowledgePoint {
        id: Uuid::new_v4().to_string(),
        course_id,
        parent_id,
        title,
        description,
        created_at: now.clone(),
        updated_at: now,
    };

    conn.execute(
        "INSERT INTO knowledge_points
            (id, course_id, parent_id, title, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            point.id,
            point.course_id,
            point.parent_id,
            point.title,
            point.description,
            point.created_at,
            point.updated_at
        ],
    )
    .map_err(|e| db_error("创建知识点", e))?;

    Ok(point)
}
