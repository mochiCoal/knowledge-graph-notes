import { invokeCommand } from './tauri'
import type { Tag, TagSummary } from '@/types'

/**
 * 标签管理模块的后端命令封装。
 *
 * 注意：Tauri 会把 Rust 的 snake_case 参数名映射为 camelCase，
 * 因此这里调用时统一使用 camelCase 参数。
 */
export const tagsApi = {
  /** 全部标签（笔记编辑器的标签建议使用） */
  list: () => invokeCommand<Tag[]>('list_tags'),

  /** 标签管理列表（含关联笔记数量） */
  overview: () => invokeCommand<TagSummary[]>('list_tag_overview'),

  /** 按名称搜索标签（空关键字等价于查询全部） */
  search: (keyword: string) => invokeCommand<TagSummary[]>('search_tags', { keyword }),

  /** 创建标签（同名标签会复用已有记录） */
  create: (name: string) => invokeCommand<Tag>('create_tag', { name }),

  /** 重命名标签 */
  update: (tagId: string, name: string) =>
    invokeCommand<Tag>('update_tag', { tagId, name }),

  /** 删除标签，返回解除关联的笔记数量 */
  remove: (tagId: string) => invokeCommand<number>('delete_tag', { tagId }),

  /** 查询某条笔记的标签 */
  listNoteTags: (noteId: string) => invokeCommand<Tag[]>('list_note_tags', { noteId }),

  /** 为笔记添加标签 */
  attachToNote: (noteId: string, tagId: string) =>
    invokeCommand<void>('attach_note_tag', { noteId, tagId }),

  /** 移除笔记标签 */
  detachFromNote: (noteId: string, tagId: string) =>
    invokeCommand<void>('detach_note_tag', { noteId, tagId }),
}
