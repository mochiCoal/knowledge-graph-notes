use serde::Serialize;

/// 知识点关联（语义关系，区别于 parent_id 表示的层级关系）
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeRelation {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    /// 关联类型，取值在阶段六确定，例如 prerequisite / related / contains
    pub relation_type: String,
    pub description: String,
    pub created_at: String,
}
