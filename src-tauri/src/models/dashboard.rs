use serde::Serialize;

use crate::db::TableCounts;

/// 首页概览：统计数字 + 最近编辑的笔记 + 最近创建的知识点
#[derive(Debug, Clone, Serialize)]
pub struct DashboardOverview {
    #[serde(flatten)]
    pub stats: TableCounts,
    pub recent_notes: Vec<RecentNote>,
    pub recent_points: Vec<RecentKnowledgePoint>,
}

/// 首页「最近编辑」中的笔记条目
#[derive(Debug, Clone, Serialize)]
pub struct RecentNote {
    pub id: String,
    pub knowledge_point_id: String,
    pub title: String,
    /// 正文摘要（已去除常见 Markdown 标记并截断）
    pub excerpt: String,
    pub knowledge_point_title: String,
    pub course_id: String,
    pub course_name: String,
    /// 正文字符数（忽略空白）
    pub character_count: i64,
    pub updated_at: String,
}

/// 首页「最近创建的知识点」条目
#[derive(Debug, Clone, Serialize)]
pub struct RecentKnowledgePoint {
    pub id: String,
    pub course_id: String,
    pub course_name: String,
    pub title: String,
    /// 描述摘要
    pub excerpt: String,
    pub parent_id: Option<String>,
    /// 从顶层知识点到父知识点的路径（不含自身），用于展示所属层级
    pub path: Vec<String>,
    /// 直接挂在该知识点上的笔记数量
    pub note_count: i64,
    pub created_at: String,
}
