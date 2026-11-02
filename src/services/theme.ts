/**
 * 主题基础能力
 *
 * 主题有三种模式：
 * - auto：跟随操作系统（Windows）深浅色变化
 * - light：强制日间
 * - dark：强制夜间
 *
 * 真正的颜色定义在 src/styles/tokens.css 的 `[data-theme]` 中，
 * 这里只负责读取偏好、解析最终主题并写入 <html data-theme="...">。
 */

export type ThemeMode = 'auto' | 'light' | 'dark'
export type ResolvedTheme = 'light' | 'dark'

/** 与 index.html 首屏脚本中的键保持一致 */
export const THEME_STORAGE_KEY = 'kgn:theme-mode'

export interface ThemeOption {
  value: ThemeMode
  label: string
  description: string
}

export const THEME_OPTIONS: readonly ThemeOption[] = [
  { value: 'auto', label: '自动', description: '跟随 Windows 系统主题' },
  { value: 'light', label: '日间', description: '始终使用浅色主题' },
  { value: 'dark', label: '夜间', description: '始终使用深色主题' },
]

export function isThemeMode(value: unknown): value is ThemeMode {
  return value === 'auto' || value === 'light' || value === 'dark'
}

export function readStoredThemeMode(): ThemeMode {
  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
    return isThemeMode(stored) ? stored : 'auto'
  } catch {
    return 'auto'
  }
}

export function persistThemeMode(mode: ThemeMode): void {
  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, mode)
  } catch {
    // localStorage 不可用时忽略，仅影响主题偏好的记忆
  }
}

export function getSystemPrefersDark(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return false
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

export function resolveTheme(mode: ThemeMode, systemPrefersDark: boolean): ResolvedTheme {
  if (mode === 'auto') {
    return systemPrefersDark ? 'dark' : 'light'
  }
  return mode
}

export function applyTheme(theme: ResolvedTheme): void {
  document.documentElement.setAttribute('data-theme', theme)
}
