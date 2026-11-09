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
 * 管理侧边栏折叠（持久化到 localStorage）与全局搜索面板的开关；
 * 搜索面板由 AppLayout 挂载，任何页面都可以通过 `openSearch()` 打开。
 */
export const useAppStore = defineStore('app', () => {
  const sidebarCollapsed = ref(readStoredCollapsed())
  const searchOpen = ref(false)

  function toggleSidebar(): void {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  function setSidebarCollapsed(collapsed: boolean): void {
    sidebarCollapsed.value = collapsed
  }

  function openSearch(): void {
    searchOpen.value = true
  }

  function closeSearch(): void {
    searchOpen.value = false
  }

  function toggleSearch(): void {
    searchOpen.value = !searchOpen.value
  }

  watch(sidebarCollapsed, persistCollapsed)

  return {
    sidebarCollapsed,
    searchOpen,
    toggleSidebar,
    setSidebarCollapsed,
    openSearch,
    closeSearch,
    toggleSearch,
  }
})
