use rusqlite::{params, Connection, OptionalExtension, Row};
use tauri::AppHandle;
use uuid::Uuid;

use crate::db::{db_error, get_connection};
use crate::models::note::Note;

fn map_note(row: &Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        knowledge_point_id: row.get(1)?,
        title: row.get(2)?,
        content: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn point_title(conn: &Connection, knowledge_point_id: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT title FROM knowledge_points WHERE id = ?1",
        [knowledge_point_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|e| db_error("读取知识点", e))?
    .ok_or_else(|| "知识点不存在，无法保存笔记".to_string())
}

fn load_note(conn: &Connection, note_id: &str) -> Result<Note, String> {
    conn.query_row(
        "SELECT id, knowledge_point_id, title, content, created_at, updated_at
         FROM notes WHERE id = ?1",
        [note_id],
        map_note,
    )
    .map_err(|e| db_error("读取笔记", e))
}

/// 查询某个知识点最近编辑的一条笔记
#[tauri::command]
pub fn get_note(app: AppHandle, knowledge_point_id: String) -> Result<Option<Note>, String> {
    let conn = get_connection(&app)?;

    conn.query_row(
        "SELECT id, knowledge_point_id, title, content, created_at, updated_at
         FROM notes
         WHERE knowledge_point_id = ?1
         ORDER BY updated_at DESC
         LIMIT 1",
        [&knowledge_point_id],
        map_note,
    )
    .optional()
    .map_err(|e| db_error("读取笔记", e))
}

/// 保存笔记：note_id 为空时新建（若该知识点已有笔记则复用最近一条）
#[tauri::command]
pub fn save_note(
    app: AppHandle,
    note_id: Option<String>,
    knowledge_point_id: String,
    title: String,
    content: String,
) -> Result<Note, String> {
    let conn = get_connection(&app)?;
    let now = chrono::Utc::now().to_rfc3339();

    let default_title = point_title(&conn, &knowledge_point_id)?;
    let mut note_title = title.trim().to_string();
    if note_title.is_empty() {
        note_title = default_title;
    }
    if note_title.chars().count() > 200 {
        return Err("笔记标题不能超过 200 个字符".into());
    }

    if let Some(existing_id) = note_id {
        let affected = conn
            .execute(
                "UPDATE notes
                 SET title = ?1, content = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![note_title, content, now, existing_id],
            )
            .map_err(|e| db_error("保存笔记", e))?;

        if affected == 0 {
            return Err("笔记不存在或已被删除，请刷新后重试".into());
        }

        return load_note(&conn, &existing_id);
    }

    // 未提供 note_id 时，优先复用该知识点已有的笔记，避免重复创建
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM notes
             WHERE knowledge_point_id = ?1
             ORDER BY updated_at DESC
             LIMIT 1",
            [&knowledge_point_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| db_error("读取笔记", e))?;

    if let Some(id) = existing_id {
        conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
            params![note_title, content, now, id],
        )
        .map_err(|e| db_error("保存笔记", e))?;
        return load_note(&conn, &id);
    }

    let note = Note {
        id: Uuid::new_v4().to_string(),
        knowledge_point_id,
        title: note_title,
        content,
        created_at: now.clone(),
        updated_at: now,
    };

    conn.execute(
        "INSERT INTO notes (id, knowledge_point_id, title, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            note.id,
            note.knowledge_point_id,
            note.title,
            note.content,
            note.created_at,
            note.updated_at
        ],
    )
    .map_err(|e| db_error("创建笔记", e))?;

    Ok(note)
}

/// 删除笔记
#[tauri::command]
pub fn delete_note(app: AppHandle, note_id: String) -> Result<(), String> {
    let conn = get_connection(&app)?;
    conn.execute("DELETE FROM notes WHERE id = ?1", [&note_id])
        .map_err(|e| db_error("删除笔记", e))?;
    Ok(())
}
