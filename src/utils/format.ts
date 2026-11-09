/** 时间与文本格式化工具 */

function toDate(value: string | Date): Date | null {
  const date = value instanceof Date ? value : new Date(value)
  return Number.isNaN(date.getTime()) ? null : date
}

/** 只显示时分，例如「14:05」，用于保存状态 */
export function formatTime(value: string | Date): string {
  const date = toDate(value)
  if (!date) return ''

  return new Intl.DateTimeFormat('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

/** 显示月日与时分，例如「11-02 14:05」，用于笔记更新时间 */
export function formatDateTime(value: string | Date): string {
  const date = toDate(value)
  if (!date) return ''

  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

/** 显示完整年月日与时分，例如「2026-11-02 14:05」，用于课程创建 / 更新时间 */
export function formatDate(value: string | Date): string {
  const date = toDate(value)
  if (!date) return ''

  const pad = (part: number): string => String(part).padStart(2, '0')
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ` +
    `${pad(date.getHours())}:${pad(date.getMinutes())}`
  )
}

/** 统计正文字符数（忽略空白字符） */
export function countCharacters(text: string): number {
  return text.replace(/\s/g, '').length
}

/** 数字千分位显示，例如「1,234」 */
export function formatNumber(value: number): string {
  return new Intl.NumberFormat('zh-CN').format(value)
}

/** 文件大小显示，例如「512 B」「12.4 KB」「3.8 MB」 */
export function formatFileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let value = bytes
  let unit = 0

  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024
    unit += 1
  }

  if (unit === 0) return `${Math.round(value)} ${units[unit]}`

  const digits = value >= 10 ? 1 : 2
  return `${value.toFixed(digits).replace(/\.0+$/, '')} ${units[unit]}`
}

/** 把笔记标题转换成可用的文件名（去掉 Windows 不允许的文件名字符） */
export function toFileName(title: string, fallback = '笔记'): string {
  const cleaned = title
    .replace(/[\\/:*?"<>|]/g, '_')
    .replace(/\s+/g, ' ')
    .trim()
    .slice(0, 80)

  return cleaned || fallback
}
