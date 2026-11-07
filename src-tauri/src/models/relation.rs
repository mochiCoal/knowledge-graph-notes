use serde::Serialize;

/// 知识点关联（语义关系，区别于 parent_id 表示的层级关系）
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeRelation {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    /// 关联类型字典 ID，例如 prerequisite / related / contains
    pub relation_type: String,
    pub description: String,
    pub created_at: String,
}

/// 关联详情（关联记录 + 两端知识点 / 课程与类型展示信息）
///
/// `relation` 使用 `flatten` 展开，前端拿到的结构与 [`KnowledgeRelation`]
/// 保持一致，另外附带标题、课程名、类型标签、颜色与是否有向。
#[derive(Debug, Clone, Serialize)]
pub struct RelationDetail {
    #[serde(flatten)]
    pub relation: KnowledgeRelation,
    pub source_title: String,
    pub target_title: String,
    pub source_course_id: String,
    pub source_course_name: String,
    pub target_course_id: String,
    pub target_course_name: String,
    /// 类型显示名称，自定义类型被删除时回退为原始类型 ID
    pub relation_label: String,
    pub relation_color: String,
    pub directed: bool,
}

/// 关联类型字典项（含使用次数）
#[derive(Debug, Clone, Serialize)]
pub struct RelationType {
    pub id: String,
    pub label: String,
    /// `#RRGGBB` 格式，图谱阶段用于边着色
    pub color: String,
    /// true 表示有向关系（前置、包含、延伸）
    pub directed: bool,
    pub description: String,
    /// 内置类型允许修改名称 / 颜色，但不允许删除
    pub is_builtin: bool,
    pub sort_order: i64,
    /// 使用该类型的关联数量
    pub usage_count: i64,
    pub created_at: String,
}

/// 关联建议（本地文本相似度计算出的候选知识点对）
#[derive(Debug, Clone, Serialize)]
pub struct RelationSuggestion {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub source_title: String,
    pub target_title: String,
    pub relation_type: String,
    pub relation_type_label: String,
    /// 综合相似度，0 ~ 1
    pub score: f64,
    /// 可直接展示的推荐理由
    pub reasons: Vec<String>,
    /// pending / accepted / rejected
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
