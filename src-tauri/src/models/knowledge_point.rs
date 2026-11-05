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

/// 知识点树节点（`get_knowledge_point_tree` 的返回元素）
///
/// 使用 `flatten` 展开，前端拿到的字段与 [`KnowledgePoint`] 完全一致，
/// 只是额外带有层级深度、子节点统计与 `children`。
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgePointNode {
    #[serde(flatten)]
    pub point: KnowledgePoint,
    /// 层级深度，顶层知识点为 0
    pub depth: i64,
    /// 直接子知识点数量
    pub child_count: i64,
    /// 直接挂在该知识点上的笔记数量（不含后代）
    pub note_count: i64,
    pub children: Vec<KnowledgePointNode>,
}

/// 知识点路径上的单个节点（详情页面包屑）
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgePointPathItem {
    pub id: String,
    pub title: String,
}

/// 知识点详情（`get_knowledge_point_detail` 的返回值）
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgePointDetail {
    pub point: KnowledgePoint,
    pub course_name: String,
    /// 从顶层知识点到当前知识点的完整路径
    pub path: Vec<KnowledgePointPathItem>,
    /// 当前知识点所在层级，顶层为 0
    pub depth: i64,
    /// 直接子知识点数量
    pub child_count: i64,
    /// 全部后代知识点数量（不含自身）
    pub descendant_count: i64,
    /// 直接挂在该知识点上的笔记数量
    pub note_count: i64,
    /// 自身及全部后代知识点上的笔记数量（删除提示使用）
    pub subtree_note_count: i64,
}

/// 删除知识点的结果
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgePointDeletion {
    /// 删除的知识点数量（包含自身与全部后代）
    pub deleted_points: i64,
    /// 随之级联删除的笔记数量
    pub deleted_notes: i64,
}
