use serde::Serialize;

/// 标签
#[derive(Debug, Clone, Serialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
}
