import { describe, expect, it } from 'vitest'

import { renderMarkdown } from '@/utils/markdown'

describe('renderMarkdown', () => {
  it('渲染常见 Markdown 语法', () => {
    const html = renderMarkdown('# 标题\n\n**重点** 与 `代码`')
    expect(html).toContain('<h1>标题</h1>')
    expect(html).toContain('<strong>重点</strong>')
    expect(html).toContain('<code>代码</code>')
  })

  it('禁用内联 HTML，脚本与事件属性按文本转义（防 XSS）', () => {
    const script = renderMarkdown('<script>alert(1)</script>')
    expect(script).not.toContain('<script>')
    expect(script).toContain('&lt;script&gt;')

    const image = renderMarkdown('<img src=x onerror=alert(1)>')
    expect(image).not.toContain('<img')
    expect(image).toContain('&lt;img')
  })

  it('链接在新窗口打开并附带安全 rel', () => {
    const html = renderMarkdown('[示例](https://example.com)')
    expect(html).toContain('target="_blank"')
    expect(html).toContain('rel="noreferrer noopener"')
    expect(html).toContain('href="https://example.com"')
  })

  it('单个换行渲染为换行，空输入返回空串', () => {
    expect(renderMarkdown('第一行\n第二行')).toContain('<br')
    expect(renderMarkdown('')).toBe('')
  })
})
