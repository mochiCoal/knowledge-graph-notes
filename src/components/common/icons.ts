/**
 * 内置图标集
 *
 * 统一采用 24×24 视窗、描边风格（参考 Lucide 图标设计），
 * 由 AppIcon 组件用 currentColor 渲染，因此无需引入额外图标依赖。
 */
export const ICON_PATHS = {
  home: '<path d="m3 9.5 9-7 9 7V20a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="M9 22v-8h6v8"/>',
  courses:
    '<path d="M22 9 12 4 2 9l10 5 10-5Z"/><path d="M6 11.6V17c0 1.7 2.7 3 6 3s6-1.3 6-3v-5.4"/><path d="M22 9v6"/>',
  notes:
    '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M8 13h8"/><path d="M8 17h5"/>',
  graph:
    '<circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><path d="m8.6 10.4 6.8-3.8"/><path d="m8.6 13.6 6.8 3.8"/>',
  settings:
    '<path d="M12.2 2h-.4a2 2 0 0 0-2 2v.2a2 2 0 0 1-1 1.7l-.4.3a2 2 0 0 1-2 0l-.2-.1a2 2 0 0 0-2.7.7l-.2.4a2 2 0 0 0 .7 2.7l.2.1a2 2 0 0 1 1 1.7v.5a2 2 0 0 1-1 1.7l-.2.1a2 2 0 0 0-.7 2.7l.2.4a2 2 0 0 0 2.7.7l.2-.1a2 2 0 0 1 2 0l.4.3a2 2 0 0 1 1 1.7v.2a2 2 0 0 0 2 2h.4a2 2 0 0 0 2-2v-.2a2 2 0 0 1 1-1.7l.4-.3a2 2 0 0 1 2 0l.2.1a2 2 0 0 0 2.7-.7l.2-.4a2 2 0 0 0-.7-2.7l-.2-.1a2 2 0 0 1-1-1.7v-.5a2 2 0 0 1 1-1.7l.2-.1a2 2 0 0 0 .7-2.7l-.2-.4a2 2 0 0 0-2.7-.7l-.2.1a2 2 0 0 1-2 0l-.4-.3a2 2 0 0 1-1-1.7V4a2 2 0 0 0-2-2Z"/><circle cx="12" cy="12" r="3"/>',
  'panel-left': '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/>',
  'panel-right': '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M15 4v16"/>',
  'arrow-right': '<path d="M5 12h14"/><path d="m13 6 6 6-6 6"/>',
  'chevron-right': '<path d="m9 5 7 7-7 7"/>',
  plus: '<path d="M12 5v14"/><path d="M5 12h14"/>',
  x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.9 4.9 1.4 1.4"/><path d="m17.7 17.7 1.4 1.4"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m4.9 19.1 1.4-1.4"/><path d="m17.7 6.3 1.4-1.4"/>',
  moon: '<path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8Z"/>',
  monitor:
    '<rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/>',
  minimize: '<path d="M5 12h14"/>',
  maximize: '<rect x="4.5" y="4.5" width="15" height="15" rx="1.5"/>',
  restore:
    '<rect x="8" y="8" width="11.5" height="11.5" rx="1.5"/><path d="M16 8V5.5A1.5 1.5 0 0 0 14.5 4h-9A1.5 1.5 0 0 0 4 5.5v9A1.5 1.5 0 0 0 5.5 16H8"/>',
  pencil: '<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"/>',
  eye: '<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/>',
  split: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M12 4v16"/>',
  trash:
    '<path d="M3 6h18"/><path d="M8 6V4h8v2"/><path d="m19 6-1 14H6L5 6"/><path d="M10 11v6"/><path d="M14 11v6"/>',
  tag: '<path d="M20.6 13.4 12 22l-9-9V3h10l7.6 7.6a2 2 0 0 1 0 2.8Z"/><circle cx="7.5" cy="7.5" r="1.5"/>',
  link: '<path d="M10 13a5 5 0 0 0 7 0l2-2a5 5 0 0 0-7-7l-1 1"/><path d="M14 11a5 5 0 0 0-7 0l-2 2a5 5 0 0 0 7 7l1-1"/>',
  alert: '<circle cx="12" cy="12" r="9"/><path d="M12 8v4"/><path d="M12 16h.01"/>',
  sparkles:
    '<path d="m12 3 1.9 4.7 4.6 1.8-4.6 1.9L12 16l-1.9-4.6L5.5 9.5l4.6-1.8Z"/><path d="M19 15.5 19.9 18l2.1.8-2.1.9-.9 2.3-.9-2.3-2.1-.9 2.1-.8Z"/>',
} as const

export type IconName = keyof typeof ICON_PATHS
