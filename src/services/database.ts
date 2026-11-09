import { open, save } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { invokeCommand } from './tauri'
import type { BackupResult, DatabaseInfo, RestoreResult } from '@/types'

/** 数据库文件过滤器，备份 / 恢复共用 */
const DATABASE_FILTER = {
  name: 'SQLite 数据库',
  extensions: ['db', 'sqlite', 'sqlite3'],
}

/** 生成备份文件名中的时间戳，例如 20261107-1710 */
function fileTimestamp(date = new Date()): string {
  const pad = (value: number): string => String(value).padStart(2, '0')
  return (
    `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}` +
    `-${pad(date.getHours())}${pad(date.getMinutes())}`
  )
}

/**
 * 数据库信息、备份与恢复的后端命令封装。
 *
 * 文件位置由系统保存 / 打开对话框选择，真正的复制与校验全部在 Rust 端完成。
 */
export const databaseApi = {
  info: () => invokeCommand<DatabaseInfo>('get_database_info'),

  /** 把当前数据库备份到指定路径（目标已存在时覆盖） */
  backup: (destination: string) =>
    invokeCommand<BackupResult>('backup_database', { destination }),

  /** 从指定备份文件恢复数据库 */
  restore: (source: string) => invokeCommand<RestoreResult>('restore_database', { source }),
}

/** 选择备份保存位置；用户取消时返回 null */
export async function pickDatabaseBackupPath(): Promise<string | null> {
  const selected = await save({
    title: '备份数据库',
    defaultPath: `知识图谱笔记-备份-${fileTimestamp()}.db`,
    filters: [DATABASE_FILTER],
  })

  return selected ?? null
}

/** 选择要恢复的备份文件；用户取消时返回 null */
export async function pickDatabaseRestoreFile(): Promise<string | null> {
  const selected = await open({
    title: '选择数据库备份文件',
    multiple: false,
    directory: false,
    filters: [DATABASE_FILTER],
  })

  return typeof selected === 'string' ? selected : null
}

/** 在系统文件管理器中定位数据库文件 */
export async function revealDatabaseFile(path: string): Promise<void> {
  await revealItemInDir(path)
}
