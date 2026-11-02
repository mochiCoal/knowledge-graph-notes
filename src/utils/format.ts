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

/** 统计正文字符数（忽略空白字符） */
export function countCharacters(text: string): number {
  return text.replace(/\s/g, '').length
}
