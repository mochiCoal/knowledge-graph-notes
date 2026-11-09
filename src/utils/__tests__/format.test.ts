import { describe, expect, it } from 'vitest'

import {
  countCharacters,
  formatDate,
  formatFileSize,
  formatNumber,
  formatTime,
  toFileName,
} from '@/utils/format'

describe('countCharacters', () => {
  it('忽略空白字符统计正文字数', () => {
    expect(countCharacters(' 你好\n 世界\t')).toBe(4)
    expect(countCharacters('   \n ')).toBe(0)
  })
})

describe('formatFileSize', () => {
  it('按单位换算并处理异常输入', () => {
    expect(formatFileSize(0)).toBe('0 B')
    expect(formatFileSize(-5)).toBe('0 B')
    expect(formatFileSize(Number.NaN)).toBe('0 B')
    expect(formatFileSize(512)).toBe('512 B')
    expect(formatFileSize(1024)).toBe('1 KB')
    expect(formatFileSize(12_800)).toBe('12.5 KB')
    expect(formatFileSize(1024 * 1024)).toBe('1 MB')
  })
})

describe('toFileName', () => {
  it('替换 Windows 非法字符并折叠空白', () => {
    expect(toFileName('a/b\\c:d*e?f"g<h>i|j')).toBe('a_b_c_d_e_f_g_h_i_j')
    expect(toFileName('  多个   空格  ')).toBe('多个 空格')
  })

  it('空标题回退默认名并限制长度', () => {
    expect(toFileName('   ')).toBe('笔记')
    expect(toFileName('')).toBe('笔记')
    expect(toFileName('长'.repeat(200))).toHaveLength(80)
  })
})

describe('日期与数字格式化', () => {
  it('非法日期返回空串而不是抛错', () => {
    expect(formatTime('not-a-date')).toBe('')
    expect(formatDate('')).toBe('')
  })

  it('按本地时间格式化', () => {
    const value = new Date(2026, 10, 7, 14, 5)
    expect(formatDate(value)).toBe('2026-11-07 14:05')
    expect(formatTime(value)).toBe('14:05')
  })

  it('数字按千分位显示', () => {
    expect(formatNumber(1234567)).toBe('1,234,567')
  })
})
