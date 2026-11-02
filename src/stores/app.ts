import { ref, watch } from 'vue'
import { defineStore } from 'pinia'

const SIDEBAR_STORAGE_KEY = 'kgn:sidebar-collapsed'

function readStoredCollapsed(): boolean {
  try {
    return window.localStorage.getItem(SIDEBAR_STORAGE_KEY) === '1'
  } catch {
    return false
  }
}

function persistCollapsed(collapsed: boolean): void {
  try {
    window.localStorage.setItem(SIDEBAR_STORAGE_KEY, collapsed ? '1' : '0')
  } catch {
    // localStorage 不可用时忽略，仅影响侧边栏折叠状态的记忆
  }
}

/**
 * 应用级界面状态。
 *
 * 目前只管理侧边栏折叠，并持久化到 localStorage；
 * 后续的主题、最近访问、全局搜索面板等界面状态也放在这里。
 */
export const useAppStore = defineStore('app', () => {
  const sidebarCollapsed = ref(readStoredCollapsed())

  function toggleSidebar(): void {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  function setSidebarCollapsed(collapsed: boolean): void {
    sidebarCollapsed.value = collapsed
  }

  watch(sidebarCollapsed, persistCollapsed)

  return {
    sidebarCollapsed,
    toggleSidebar,
    setSidebarCollapsed,
  }
})
