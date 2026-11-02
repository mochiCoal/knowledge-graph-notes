use serde::Serialize;

/// 知识点（通过 parent_id 形成层级结构）
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgePoint {
    pub id: String,
    pub course_id: String,
    /// 父知识点 ID，顶层知识点为 None
    pub parent_id: Option<String>,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}
