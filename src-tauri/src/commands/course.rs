use rusqlite::{params, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, get_connection};
use crate::models::course::Course;

fn map_course(row: &Row<'_>) -> rusqlite::Result<Course> {
    Ok(Course {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

/// 查询全部课程（按创建时间升序）
#[tauri::command]
pub fn list_courses(app: AppHandle) -> Result<Vec<Course>, String> {
    let conn = get_connection(&app)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, created_at, updated_at
             FROM courses
             ORDER BY created_at ASC",
        )
        .map_err(|e| db_error("读取课程列表", e))?;

    stmt.query_map([], map_course)
        .map_err(|e| db_error("读取课程列表", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取课程列表", e))
}

/// 新增课程
#[tauri::command]
pub fn create_course(app: AppHandle, name: String, description: String) -> Result<Course, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("课程名称不能为空".into());
    }
    if name.chars().count() > 60 {
        return Err("课程名称不能超过 60 个字符".into());
    }

    let description = description.trim().to_string();
    if description.chars().count() > 500 {
        return Err("课程描述不能超过 500 个字符".into());
    }

    let now = chrono::Utc::now().to_rfc3339();
    let course = Course {
        id: Uuid::new_v4().to_string(),
        name,
        description,
        created_at: now.clone(),
        updated_at: now,
    };

    let conn = get_connection(&app)?;
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
