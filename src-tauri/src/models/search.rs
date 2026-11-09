use serde::Serialize;

/// 全局搜索结果的类型
#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    /// 对应实体的 ID（课程 / 知识点 / 笔记 / 标签 / 关联）
    pub id: String,
    /// 实体类型：course / knowledge_point / note / tag / relation
    pub kind: String,
    /// 主标题
    pub title: String,
    /// 次级说明：所属课程、层级路径、标签使用量等
    pub subtitle: String,
    /// 命中内容片段；没有正文时为空字符串
    pub excerpt: String,
    /// 结果所属课程（用于跳转知识点管理与知识关联页）
    pub course_id: Option<String>,
    /// 结果关联的知识点（用于跳转笔记工作区与知识关联页）
    pub knowledge_point_id: Option<String>,
    pub updated_at: Option<String>,
}

/// 全局搜索结果：按实体类型分组，前端按分组展示
#[derive(Debug, Clone, Serialize)]
pub struct GlobalSearchResult {
    pub query: String,
    /// 所有分组的结果总数
    pub total: usize,
    pub courses: Vec<SearchHit>,
    pub knowledge_points: Vec<SearchHit>,
    pub notes: Vec<SearchHit>,
    pub tags: Vec<SearchHit>,
    pub relations: Vec<SearchHit>,
}
