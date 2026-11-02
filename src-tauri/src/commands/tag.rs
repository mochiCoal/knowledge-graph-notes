use rusqlite::{params, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, get_connection};
use crate::models::tag::Tag;

fn map_tag(row: &Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
    })
}

/// 查询全部标签
#[tauri::command]
pub fn list_tags(app: AppHandle) -> Result<Vec<Tag>, String> {
    let conn = get_connection(&app)?;
    let mut stmt = conn
        .prepare("SELECT id, name FROM tags ORDER BY name ASC")
        .map_err(|e| db_error("读取标签列表", e))?;

    stmt.query_map([], map_tag)
        .map_err(|e| db_error("读取标签列表", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取标签列表", e))
}

/// 创建标签（同名标签直接复用已有记录）
#[tauri::command]
pub fn create_tag(app: AppHandle, name: String) -> Result<Tag, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("标签名称不能为空".into());
    }
    if name.chars().count() > 30 {
        return Err("标签名称不能超过 30 个字符".into());
    }

    let conn = get_connection(&app)?;
    conn.execute(
        "INSERT OR IGNORE INTO tags (id, name) VALUES (?1, ?2)",
        params![Uuid::new_v4().to_string(), name],
    )
    .map_err(|e| db_error("创建标签", e))?;

    conn.query_row("SELECT id, name FROM tags WHERE name = ?1", [&name], map_tag)
        .map_err(|e| db_error("读取标签", e))
}

/// 查询某条笔记的标签
#[tauri::command]
pub fn list_note_tags(app: AppHandle, note_id: String) -> Result<Vec<Tag>, String> {
    let conn = get_connection(&app)?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name
             FROM tags t
             INNER JOIN note_tags nt ON nt.tag_id = t.id
             WHERE nt.note_id = ?1
             ORDER BY t.name ASC",
        )
        .map_err(|e| db_error("读取笔记标签", e))?;

    stmt.query_map([&note_id], map_tag)
        .map_err(|e| db_error("读取笔记标签", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取笔记标签", e))
}

/// 为笔记添加标签
#[tauri::command]
pub fn attach_note_tag(app: AppHandle, note_id: String, tag_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    conn.execute(
        "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
        params![note_id, tag_id],
    )
    .map_err(|e| db_error("添加标签", e))?;
    Ok(())
}

/// 移除笔记标签
#[tauri::command]
pub fn detach_note_tag(app: AppHandle, note_id: String, tag_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    conn.execute(
        "DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2",
        params![note_id, tag_id],
    )
    .map_err(|e| db_error("移除标签", e))?;
    Ok(())
}
