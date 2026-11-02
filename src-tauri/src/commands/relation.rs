use rusqlite::Row;
use tauri::AppHandle;

use crate::db::{db_error, get_connection};
use crate::models::relation::KnowledgeRelation;

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

/// 查询全部知识点关联（笔记页右侧面板展示相关知识点）
#[tauri::command]
pub fn list_relations(app: AppHandle) -> Result<Vec<KnowledgeRelation>, String> {
    let conn = get_connection(&app)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, source_id, target_id, relation_type, description, created_at
             FROM knowledge_relations
             ORDER BY created_at ASC",
        )
        .map_err(|e| db_error("读取知识点关联", e))?;

    stmt.query_map([], map_relation)
        .map_err(|e| db_error("读取知识点关联", e))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| db_error("读取知识点关联", e))
}
