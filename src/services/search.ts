import { invokeCommand } from './tauri'
import type { GlobalSearchResult } from '@/types'

/**
 * 全局搜索：在课程、知识点、笔记、标签与知识点关联中查找关键字。
 *
 * 空关键字由后端直接返回空结果，前端无需额外判断。
 */
export const searchApi = {
  global: (keyword: string) =>
    invokeCommand<GlobalSearchResult>('global_search', { keyword }),
}
