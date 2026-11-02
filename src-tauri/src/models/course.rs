use serde::Serialize;

/// 课程
#[derive(Debug, Clone, Serialize)]
pub struct Course {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}
