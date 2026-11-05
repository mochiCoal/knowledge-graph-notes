use serde::Serialize;

/// 课程
#[derive(Debug, Clone, Serialize)]
pub struct Course {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    /// 课程下的知识点数量（查询时聚合，新建课程为 0）
    pub knowledge_point_count: i64,
    /// 课程下的笔记数量（查询时聚合，新建课程为 0）
    pub note_count: i64,
}
