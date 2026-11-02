use serde::Serialize;

/// 笔记（属于某个知识点）
#[derive(Debug, Clone, Serialize)]
pub struct Note {
    pub id: String,
    pub knowledge_point_id: String,
    pub title: String,
    /// Markdown 原文
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}
