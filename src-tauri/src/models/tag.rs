use serde::Serialize;

/// 标签
#[derive(Debug, Clone, Serialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
}

/// 标签管理列表条目：标签 + 关联笔记数量
#[derive(Debug, Clone, Serialize)]
pub struct TagSummary {
    pub id: String,
    pub name: String,
    /// 使用该标签的笔记数量
    pub note_count: i64,
}
