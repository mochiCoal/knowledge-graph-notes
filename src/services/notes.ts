import { invokeCommand } from './tauri'
import type { KnowledgePoint, KnowledgeRelation, MarkdownFile, Note, NoteSummary, Tag } from '@/types'

/** 新建知识点入参 */
export interface CreateKnowledgePointInput {
  courseId: string
  parentId: string | null
  title: string
  description: string
}

/** 保存笔记入参（noteId 为空表示新建） */
export interface SaveNoteInput {
  noteId: string | null
  knowledgePointId: string
  title: string
  content: string
}

/**
 * 笔记工作区使用的后端命令封装。
 *
 * 注意：Tauri 会把 Rust 的 snake_case 参数名映射为 camelCase，
 * 因此这里调用时统一使用 camelCase 参数。
 */
export const notesApi = {
  listKnowledgePoints: () => invokeCommand<KnowledgePoint[]>('list_knowledge_points'),

  createKnowledgePoint: (input: CreateKnowledgePointInput) =>
    invokeCommand<KnowledgePoint>('create_knowledge_point', {
      courseId: input.courseId,
      parentId: input.parentId,
      title: input.title,
      description: input.description,
    }),

  getNote: (knowledgePointId: string) =>
    invokeCommand<Note | null>('get_note', { knowledgePointId }),

  saveNote: (input: SaveNoteInput) =>
    invokeCommand<Note>('save_note', {
      noteId: input.noteId,
      knowledgePointId: input.knowledgePointId,
      title: input.title,
      content: input.content,
    }),

  deleteNote: (noteId: string) => invokeCommand<void>('delete_note', { noteId }),

  /** 全部笔记摘要（笔记库列表） */
  listNotes: () => invokeCommand<NoteSummary[]>('list_notes'),

  /** 按关键字搜索笔记标题与正文 */
  searchNotes: (keyword: string) => invokeCommand<NoteSummary[]>('search_notes', { keyword }),

  /** 把笔记导出为 Markdown 文件，返回实际写入路径 */
  exportNote: (noteId: string, filePath: string) =>
    invokeCommand<string>('export_note_markdown', { noteId, filePath }),

  /** 读取本地 Markdown 文件（导入前预览标题与正文） */
  readMarkdownFile: (filePath: string) =>
    invokeCommand<MarkdownFile>('read_markdown_file', { filePath }),

  listTags: () => invokeCommand<Tag[]>('list_tags'),

  createTag: (name: string) => invokeCommand<Tag>('create_tag', { name }),

  listNoteTags: (noteId: string) => invokeCommand<Tag[]>('list_note_tags', { noteId }),

  attachTag: (noteId: string, tagId: string) =>
    invokeCommand<void>('attach_note_tag', { noteId, tagId }),

  detachTag: (noteId: string, tagId: string) =>
    invokeCommand<void>('detach_note_tag', { noteId, tagId }),

  listRelations: () => invokeCommand<KnowledgeRelation[]>('list_relations'),
}
