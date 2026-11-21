/**
 * 前端领域模型类型
 *
 * 字段命名与 SQLite 表结构（src-tauri/migrations/001_initial.sql）、
 * Rust serde 默认序列化结果保持一致（snake_case），避免前后端来回转换字段名。
 */

/** 课程 */
export interface Course {
  id: string
  name: string
  description: string
  created_at: string
  updated_at: string
  /** 课程下的知识点数量（后端聚合返回） */
  knowledge_point_count: number
  /** 课程下的笔记数量（后端聚合返回） */
  note_count: number
}

/** 知识点（通过 parent_id 形成层级结构） */
export interface KnowledgePoint {
  id: string
  course_id: string
  /** 父知识点 ID，顶层知识点为 null */
  parent_id: string | null
  title: string
  description: string
  created_at: string
  updated_at: string
}

/**
 * 知识点树节点（后端 `get_knowledge_point_tree` 返回）
 *
 * 后端使用 serde flatten 展开，因此字段与 KnowledgePoint 完全一致，
 * 另外附带层级深度、子节点 / 笔记数量与 children。
 */
export interface KnowledgePointNode extends KnowledgePoint {
  /** 层级深度，顶层为 0 */
  depth: number
  /** 直接子知识点数量 */
  child_count: number
  /** 直接挂在该知识点上的笔记数量（不含后代） */
  note_count: number
  children: KnowledgePointNode[]
}

/** 知识图谱中的知识点（由树节点扁平化而来） */
export interface GraphPoint extends KnowledgePoint {
  /** 层级深度，顶层为 0 */
  depth: number
  /** 直接子知识点数量 */
  child_count: number
  /** 直接挂在该知识点上的笔记数量 */
  note_count: number
}

/** 知识图谱的两种布局 */
export type GraphLayoutMode = 'hierarchy' | 'force'

/** 自定义图谱节点携带的数据 */
export interface GraphNodeData {
  point: GraphPoint
  /** 参与当前图谱绘制的语义关联数量 */
  relationCount: number
  /** 选中其他节点时，不在一跳邻域内需要弱化 */
  dimmed: boolean
  /** 同名知识点的区分信息（父级标题或序号），无歧义时为 null */
  scope: string | null
  /** 完整层级路径，用于悬浮提示 */
  pathLabel: string
}

/** 自定义图谱连线的数据 */
export interface GraphEdgeData {
  /** 层级边或语义关联边，决定连线样式（虚线 / 实线着色） */
  kind: 'hierarchy' | 'relation'
  /** 选中节点的一跳关联：需要加粗并显示标签 */
  highlighted: boolean
  /** 不在一跳邻域内需要弱化 */
  dimmed: boolean
  /** 当前是否需要渲染关系标签 */
  labelText?: string
  /** 同一对节点之间平行边的序号（从 0 开始），用于把多条边分散到不同弧线 */
  parallelIndex?: number
  /** 同一对节点之间的平行边总数 */
  parallelCount?: number
}

/** 知识点详情面包屑中的单个节点 */
export interface KnowledgePointPathItem {
  id: string
  title: string
}

/** 知识点详情（后端 `get_knowledge_point_detail` 返回） */
export interface KnowledgePointDetail {
  point: KnowledgePoint
  course_name: string
  /** 从顶层知识点到当前知识点的完整路径 */
  path: KnowledgePointPathItem[]
  /** 当前层级，顶层为 0 */
  depth: number
  /** 直接子知识点数量 */
  child_count: number
  /** 全部后代知识点数量（不含自身） */
  descendant_count: number
  /** 直接挂在该知识点上的笔记数量 */
  note_count: number
  /** 自身及全部后代的笔记数量 */
  subtree_note_count: number
  /** 与该知识点相关的语义关联总数（出边 + 入边） */
  relation_count: number
  /** 以该知识点为源点的关联数量 */
  outgoing_relation_count: number
  /** 以该知识点为目标点的关联数量 */
  incoming_relation_count: number
}

/** 删除知识点的结果（后端 `delete_knowledge_point` 返回） */
export interface KnowledgePointDeletion {
  /** 删除的知识点数量（含自身与全部后代） */
  deleted_points: number
  /** 随之级联删除的笔记数量 */
  deleted_notes: number
}

/** 笔记（属于某个知识点） */
export interface Note {
  id: string
  knowledge_point_id: string
  title: string
  /** Markdown 原文 */
  content: string
  created_at: string
  updated_at: string
}

/** 笔记列表 / 搜索结果条目（后端 `list_notes` / `search_notes` 返回） */
export interface NoteSummary {
  id: string
  knowledge_point_id: string
  title: string
  /** 正文摘要（已去除常见 Markdown 标记并截断） */
  excerpt: string
  created_at: string
  updated_at: string
  /** 所属知识点标题 */
  knowledge_point_title: string
  course_id: string
  course_name: string
  /** 正文字符数（忽略空白） */
  character_count: number
  /** 笔记的标签名称（按名称排序） */
  tag_names: string[]
}

/** 导入前读取到的本地 Markdown 文件（后端 `read_markdown_file` 返回） */
export interface MarkdownFile {
  file_name: string
  /** 从一级标题或文件名解析出的标题 */
  title: string
  /** 去掉首行一级标题后的正文 */
  content: string
  character_count: number
}

/** 标签 */
export interface Tag {
  id: string
  name: string
}

/** 标签管理列表条目（后端 `list_tag_overview` / `search_tags` 返回） */
export interface TagSummary {
  id: string
  name: string
  /** 使用该标签的笔记数量 */
  note_count: number
}

/** 笔记与标签的关联 */
export interface NoteTag {
  note_id: string
  tag_id: string
}

/** 知识点关联（语义关系，区别于 parent_id 表示的层级关系） */
export interface KnowledgeRelation {
  id: string
  source_id: string
  target_id: string
  /** 关联类型字典 ID，例如 prerequisite / related / contains */
  relation_type: string
  description: string
  created_at: string
}

/** 关联详情（关联记录 + 两端知识点 / 课程与类型展示信息） */
export interface RelationDetail extends KnowledgeRelation {
  source_title: string
  target_title: string
  source_course_id: string
  source_course_name: string
  target_course_id: string
  target_course_name: string
  /** 类型显示名称（自定义类型被删除时回退为类型 ID） */
  relation_label: string
  relation_color: string
  /** 有向关系（前置 / 包含 / 延伸）为 true */
  directed: boolean
}

/** 关联类型字典项（含使用次数） */
export interface RelationType {
  id: string
  label: string
  /** `#RRGGBB` 格式，图谱阶段用于边着色 */
  color: string
  directed: boolean
  description: string
  is_builtin: boolean
  sort_order: number
  usage_count: number
  created_at: string
}

/** 关联建议（本地文本相似度计算出的候选知识点对） */
export interface RelationSuggestion {
  id: string
  source_id: string
  target_id: string
  source_title: string
  target_title: string
  relation_type: string
  relation_type_label: string
  /** 综合相似度，0 ~ 1 */
  score: number
  /** 可直接展示的推荐理由 */
  reasons: string[]
  /** pending / accepted / rejected */
  status: string
  created_at: string
  updated_at: string
}

/** 笔记编辑区的三种视图模式 */
export type EditorMode = 'edit' | 'split' | 'preview'

/** 笔记保存状态 */
export type SaveStatus = 'idle' | 'dirty' | 'saving' | 'saved' | 'error'

/** 首页概览统计（后端 `get_dashboard_overview` 返回） */
export interface DashboardStats {
  course_count: number
  knowledge_point_count: number
  note_count: number
  tag_count: number
  relation_count: number
}

/** 首页「最近编辑」中的笔记 */
export interface RecentNote {
  id: string
  knowledge_point_id: string
  title: string
  /** 正文摘要（已去除常见 Markdown 标记并截断） */
  excerpt: string
  knowledge_point_title: string
  course_id: string
  course_name: string
  character_count: number
  updated_at: string
}

/** 首页「最近创建的知识点」 */
export interface RecentKnowledgePoint {
  id: string
  course_id: string
  course_name: string
  title: string
  /** 描述摘要 */
  excerpt: string
  parent_id: string | null
  /** 从顶层知识点到父知识点的路径（不含自身） */
  path: string[]
  note_count: number
  created_at: string
}

/** 首页概览数据 */
export interface DashboardOverview extends DashboardStats {
  recent_notes: RecentNote[]
  recent_points: RecentKnowledgePoint[]
}

/** 全局搜索命中的实体类型 */
export type SearchHitKind = 'course' | 'knowledge_point' | 'note' | 'tag' | 'relation'

/** 全局搜索的单条结果 */
export interface SearchHit {
  id: string
  kind: SearchHitKind
  title: string
  /** 所属课程 / 层级路径 / 标签使用量等说明 */
  subtitle: string
  /** 命中内容片段 */
  excerpt: string
  course_id: string | null
  knowledge_point_id: string | null
  updated_at: string | null
}

/** 全局搜索结果（按实体类型分组） */
export interface GlobalSearchResult {
  query: string
  /** 所有分组的结果总数 */
  total: number
  courses: SearchHit[]
  knowledge_points: SearchHit[]
  notes: SearchHit[]
  tags: SearchHit[]
  relations: SearchHit[]
}

/** 数据库文件与数据量信息 */
export interface DatabaseInfo extends DashboardStats {
  path: string
  directory: string
  size_bytes: number
  modified_at: string | null
  sqlite_version: string
}

/** 数据库备份结果 */
export interface BackupResult {
  path: string
  size_bytes: number
  created_at: string
}

/** 数据库恢复结果 */
export interface RestoreResult {
  source_path: string
  /** 恢复前自动保存的安全副本 */
  safety_backup_path: string
  restored_at: string
  database: DatabaseInfo
}

/** 课程包导出结果（后端 `export_course_package` 返回） */
export interface CourseExportResult {
  /** 实际写入的课程包文件路径 */
  path: string
  course_name: string
  knowledge_point_count: number
  note_count: number
  relation_count: number
  tag_count: number
}

/** 课程包导入结果（后端 `import_course_package` 返回） */
export interface CourseImportResult {
  /** 新课程的 ID，可用于跳转课程详情 */
  course_id: string
  /** 实际写入的课程名称（同名时自动追加「（导入）」后缀） */
  course_name: string
  /** 课程因重名被自动重命名时为 true */
  renamed: boolean
  knowledge_point_count: number
  note_count: number
  relation_count: number
  tag_count: number
}

