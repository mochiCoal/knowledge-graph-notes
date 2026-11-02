import MarkdownIt from 'markdown-it'

/**
 * Markdown 渲染实例
 *
 * - html: false —— 禁用内联 HTML，避免预览中执行不可信内容；
 * - linkify: true —— 自动识别链接；
 * - breaks: true —— 单个换行也渲染为换行，符合中文笔记的书写习惯。
 */
const markdown = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: true,
  typographer: false,
})

// 外链在新窗口打开，避免覆盖应用界面
const defaultLinkOpen =
  markdown.renderer.rules.link_open ??
  ((tokens, index, options, _env, self) => self.renderToken(tokens, index, options))

markdown.renderer.rules.link_open = (tokens, index, options, env, self) => {
  tokens[index].attrSet('target', '_blank')
  tokens[index].attrSet('rel', 'noreferrer noopener')
  return defaultLinkOpen(tokens, index, options, env, self)
}

export function renderMarkdown(source: string): string {
  return markdown.render(source)
}
