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

/** 标签 */
export interface Tag {
  id: string
  name: string
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
  /** 关联类型，取值由阶段六确定，例如 prerequisite / related / contains */
  relation_type: string
  description: string
  created_at: string
}

/** 笔记编辑区的三种视图模式 */
export type EditorMode = 'edit' | 'split' | 'preview'

/** 笔记保存状态 */
export type SaveStatus = 'idle' | 'dirty' | 'saving' | 'saved' | 'error'

