import { computed, ref, watch } from 'vue'
import { defineStore } from 'pinia'
import {
  applyTheme,
  getSystemPrefersDark,
  persistThemeMode,
  readStoredThemeMode,
  resolveTheme,
  type ResolvedTheme,
  type ThemeMode,
} from '@/services/theme'

/** 系统深浅色媒体查询（应用运行期间常驻，用于自动模式实时响应系统变化） */
const systemDarkQuery =
  typeof window !== 'undefined' && typeof window.matchMedia === 'function'
    ? window.matchMedia('(prefers-color-scheme: dark)')
    : null

/**
 * 主题状态（全局唯一）
 *
 * 所有主题相关状态和副作用集中在这里：
 * 1. 读取 / 保存用户偏好；
 * 2. 解析 auto 模式的最终主题；
 * 3. 将最终主题写入 <html data-theme>，全局 CSS 变量随之切换；
 * 4. 监听系统主题变化，仅在 auto 模式下生效。
 */
export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>(readStoredThemeMode())
  const systemPrefersDark = ref(systemDarkQuery?.matches ?? getSystemPrefersDark())

  const resolvedTheme = computed<ResolvedTheme>(() =>
    resolveTheme(mode.value, systemPrefersDark.value),
  )
  const isDark = computed(() => resolvedTheme.value === 'dark')

  function setMode(next: ThemeMode): void {
    mode.value = next
  }

  // 主题变化立即生效；启动时也会执行一次，与 index.html 的首屏脚本保持一致
  watch(resolvedTheme, (theme) => applyTheme(theme), { immediate: true })
  watch(mode, (value) => persistThemeMode(value))

  systemDarkQuery?.addEventListener('change', (event) => {
    systemPrefersDark.value = event.matches
  })

  return {
    mode,
    resolvedTheme,
    isDark,
    systemPrefersDark,
    setMode,
  }
})
