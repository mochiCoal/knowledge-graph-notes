import { open, save } from '@tauri-apps/plugin-dialog'
import { notesApi } from './notes'
import { toFileName } from '@/utils/format'
import type { MarkdownFile } from '@/types'

/** Markdown 文件过滤器，导入 / 导出共用 */
const MARKDOWN_FILTER = {
  name: 'Markdown 文件',
  extensions: ['md', 'markdown', 'txt'],
}

/** 选择要导入的 Markdown 文件；用户取消时返回 null */
export async function pickMarkdownFile(): Promise<string | null> {
  const selected = await open({
    title: '选择要导入的 Markdown 文件',
    multiple: false,
    directory: false,
    filters: [MARKDOWN_FILTER],
  })

  return typeof selected === 'string' ? selected : null
}

/** 读取并解析 Markdown 文件（标题取首行一级标题或文件名） */
export async function readMarkdownFile(path: string): Promise<MarkdownFile> {
  return notesApi.readMarkdownFile(path)
}

/** 选择导出位置；用户取消时返回 null */
export async function pickMarkdownSavePath(title: string): Promise<string | null> {
  const selected = await save({
    title: '导出 Markdown 笔记',
    defaultPath: `${toFileName(title)}.md`,
    filters: [MARKDOWN_FILTER],
  })

  return selected ?? null
}

/**
 * 弹出保存对话框并把笔记导出为 Markdown 文件。
 *
 * @returns 实际写入的文件路径；用户取消时返回 null
 */
export async function exportNoteToMarkdown(note: {
  id: string
  title: string
}): Promise<string | null> {
  const path = await pickMarkdownSavePath(note.title)
  if (!path) return null

  return notesApi.exportNote(note.id, path)
}
