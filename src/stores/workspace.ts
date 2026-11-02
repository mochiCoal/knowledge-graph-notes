import { ref, watch } from 'vue'
import { defineStore } from 'pinia'
import type { EditorMode } from '@/types'

/**
 * 笔记工作区的布局偏好（与业务数据无关，单独放在一个 store）
 *
 * 包括：左侧目录宽度、是否折叠、右侧辅助面板是否显示、编辑区视图模式。
 * 全部持久化到 localStorage，重新打开应用后保持上次的布局。
 */

const STORAGE_KEY = 'kgn:notes:ui'
const MIN_SIDEBAR_WIDTH = 200
const MAX_SIDEBAR_WIDTH = 380

interface StoredLayout {
  sidebarWidth: number
  sidebarCollapsed: boolean
  metaOpen: boolean
  editorMode: EditorMode
}

function isEditorMode(value: unknown): value is EditorMode {
  return value === 'edit' || value === 'split' || value === 'preview'
}

function clampWidth(value: number): number {
  if (!Number.isFinite(value)) return 260
  return Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, Math.round(value)))
}

function readLayout(): StoredLayout {
  const fallback: StoredLayout = {
    sidebarWidth: 260,
    sidebarCollapsed: false,
    // 窗口较窄时默认隐藏右侧面板，把空间留给编辑区
    metaOpen: typeof window !== 'undefined' ? window.innerWidth >= 1180 : true,
    editorMode: 'split',
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return fallback

    const parsed = JSON.parse(raw) as Partial<StoredLayout>

    return {
      sidebarWidth: clampWidth(
        typeof parsed.sidebarWidth === 'number' ? parsed.sidebarWidth : fallback.sidebarWidth,
      ),
      sidebarCollapsed:
        typeof parsed.sidebarCollapsed === 'boolean'
          ? parsed.sidebarCollapsed
          : fallback.sidebarCollapsed,
      metaOpen: typeof parsed.metaOpen === 'boolean' ? parsed.metaOpen : fallback.metaOpen,
      editorMode: isEditorMode(parsed.editorMode) ? parsed.editorMode : fallback.editorMode,
    }
  } catch {
    return fallback
  }
}

export const useWorkspaceStore = defineStore('workspace', () => {
  const initial = readLayout()

  const sidebarWidth = ref(initial.sidebarWidth)
  const sidebarCollapsed = ref(initial.sidebarCollapsed)
  const metaOpen = ref(initial.metaOpen)
  const editorMode = ref<EditorMode>(initial.editorMode)
  const resizing = ref(false)

  function setSidebarWidth(width: number): void {
    sidebarWidth.value = clampWidth(width)
  }

  function toggleSidebar(): void {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  function toggleMeta(): void {
    metaOpen.value = !metaOpen.value
  }

  function setEditorMode(mode: EditorMode): void {
    editorMode.value = mode
  }

  function beginResize(): void {
    resizing.value = true
  }

  function endResize(): void {
    resizing.value = false
  }

  watch([sidebarWidth, sidebarCollapsed, metaOpen, editorMode], () => {
    try {
      const layout: StoredLayout = {
        sidebarWidth: sidebarWidth.value,
        sidebarCollapsed: sidebarCollapsed.value,
        metaOpen: metaOpen.value,
        editorMode: editorMode.value,
      }
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(layout))
    } catch {
      // localStorage 不可用时忽略，仅影响布局记忆
    }
  })

  return {
    sidebarWidth,
    sidebarCollapsed,
    metaOpen,
    editorMode,
    resizing,
    setSidebarWidth,
    toggleSidebar,
    toggleMeta,
    setEditorMode,
    beginResize,
    endResize,
  }
})
