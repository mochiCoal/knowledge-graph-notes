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

/// 笔记列表 / 搜索结果的条目
///
/// 只携带摘要与所属课程 / 知识点信息，避免列表页加载全部 Markdown 正文。
#[derive(Debug, Clone, Serialize)]
pub struct NoteSummary {
    pub id: String,
    pub knowledge_point_id: String,
    pub title: String,
    /// 正文摘要（已去除常见 Markdown 标记并截断）
    pub excerpt: String,
    pub created_at: String,
    pub updated_at: String,
    /// 所属知识点标题
    pub knowledge_point_title: String,
    pub course_id: String,
    pub course_name: String,
    /// 正文字符数（忽略空白）
    pub character_count: i64,
}

/// 从本地读取到的 Markdown 文件（导入前预览）
#[derive(Debug, Clone, Serialize)]
pub struct MarkdownFile {
    pub file_name: String,
    /// 从一级标题或文件名解析出的笔记标题
    pub title: String,
    /// 去掉首行一级标题后的 Markdown 正文
    pub content: String,
    pub character_count: i64,
}
