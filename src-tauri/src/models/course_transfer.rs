use serde::{Deserialize, Serialize};

/// 课程包格式标识，导入时用于确认文件类型
pub const COURSE_PACKAGE_FORMAT: &str = "kgn-course";

/// 课程包格式版本；升级格式时递增，并在导入逻辑中做兼容处理
pub const COURSE_PACKAGE_VERSION: u32 = 1;

/// 课程包文件：一门课程及其下的知识点、笔记、标签与课程内关联
///
/// 所有记录只保存课程内部的相对引用（旧 id），导入时统一重新生成 UUID，
/// 因此同一个课程包可以重复导入到同一数据库而不会冲突。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoursePackage {
    pub format: String,
    pub version: u32,
    pub exported_at: String,
    pub course: PackageCourse,
    #[serde(default)]
    pub knowledge_points: Vec<PackageKnowledgePoint>,
    #[serde(default)]
    pub notes: Vec<PackageNote>,
    #[serde(default)]
    pub relations: Vec<PackageRelation>,
    /// 课程关联实际用到的关联类型定义（内置类型通常已存在，自定义类型随包补齐）
    #[serde(default)]
    pub relation_types: Vec<PackageRelationType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCourse {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageKnowledgePoint {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageNote {
    pub id: String,
    pub knowledge_point_id: String,
    pub title: String,
    #[serde(default)]
    pub content: String,
    /// 标签以名称形式保存，导入时按名称复用已有标签，避免重复创建
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRelation {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    #[serde(default)]
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRelationType {
    pub id: String,
    pub label: String,
    pub color: String,
    pub directed: bool,
    #[serde(default)]
    pub description: String,
    pub sort_order: i64,
}

/// 课程导出结果
#[derive(Debug, Clone, Serialize)]
pub struct CourseExportResult {
    pub path: String,
    pub course_name: String,
    pub knowledge_point_count: i64,
    pub note_count: i64,
    pub relation_count: i64,
    pub tag_count: i64,
}

/// 课程导入结果
#[derive(Debug, Clone, Serialize)]
pub struct CourseImportResult {
    pub course_id: String,
    pub course_name: String,
    /// 同名课程被自动重命名时为 true，前端可提示实际使用的名称
    pub renamed: bool,
    pub knowledge_point_count: i64,
    pub note_count: i64,
    pub relation_count: i64,
    pub tag_count: i64,
}