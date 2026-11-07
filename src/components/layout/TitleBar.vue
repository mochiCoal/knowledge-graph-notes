<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import AppIcon from '@/components/common/AppIcon.vue'
import { APP_NAME } from '@/config'
import { useNotesStore } from '@/stores/notes'
import { useThemeStore } from '@/stores/theme'

/**
 * 自定义标题栏
 *
 * - 无边框窗口（tauri.conf.json 中 decorations: false）下承担系统标题栏职责；
 * - 整条标题栏与空白区域可拖动窗口，双击行为由 Tauri 的拖拽区域处理；
 * - 最小化 / 最大化 / 关闭按钮通过 Tauri Window API 操作系统窗口；
 * - 关闭前先保存笔记草稿，避免直接关窗丢失最后几秒的输入；
 * - 全部颜色使用主题变量，随日间 / 夜间主题同步变化。
 */
const appWindow = isTauri() ? getCurrentWindow() : null
const notesStore = useNotesStore()
const themeStore = useThemeStore()

const isMaximized = ref(false)
const isClosing = ref(false)

let unlistenResized: (() => void) | null = null
let unlistenClose: (() => void) | null = null
let disposed = false

/**
 * 无边框窗口在 Windows 上会保留一小圈非客户区（用于缩放和圆角），
 * 这里把窗口背景色同步为当前主题的背景色，避免深色主题下出现白边 / 透出桌面。
 */
watch(
  () => themeStore.resolvedTheme,
  () => {
    if (!appWindow) return
    const color = getComputedStyle(document.documentElement)
      .getPropertyValue('--color-bg')
      .trim()
    if (!color) return
    void appWindow.setBackgroundColor(color).catch(() => {})
  },
  { immediate: true },
)

async function syncMaximized(): Promise<void> {
  if (!appWindow) return
  try {
    isMaximized.value = await appWindow.isMaximized()
  } catch {
    // 窗口尚未就绪时忽略
  }
}

onMounted(async () => {
  if (!appWindow) return

  await syncMaximized()

  const stopResize = await appWindow.onResized(() => {
    void syncMaximized()
  })
  if (disposed) {
    stopResize()
  } else {
    unlistenResized = stopResize
  }

  const stopClose = await appWindow.onCloseRequested(async (event) => {
    if (isClosing.value) return

    event.preventDefault()
    isClosing.value = true

    try {
      await notesStore.flushSave()
    } finally {
      // 使用 destroy 直接销毁窗口，避免再次触发 closeRequested 造成循环
      await appWindow.destroy()
    }
  })
  if (disposed) {
    stopClose()
  } else {
    unlistenClose = stopClose
  }
})

onBeforeUnmount(() => {
  disposed = true
  unlistenResized?.()
  unlistenClose?.()
  unlistenResized = null
  unlistenClose = null
})

function minimize(): void {
  void appWindow?.minimize()
}

function toggleMaximize(): void {
  void appWindow?.toggleMaximize()
}

function close(): void {
  void appWindow?.close()
}

const maximizeLabel = computed(() => (isMaximized.value ? '还原' : '最大化'))
</script>

<template>
  <header class="title-bar" data-tauri-drag-region>
    <div class="title-bar__brand" data-tauri-drag-region>
      <span class="title-bar__mark" data-tauri-drag-region>
        <AppIcon name="graph" :size="11" />
      </span>
      <span class="title-bar__title" data-tauri-drag-region>{{ APP_NAME }}</span>
    </div>

    <div class="title-bar__spacer" data-tauri-drag-region />

    <div class="window-controls">
      <button
        type="button"
        class="window-control"
        title="最小化"
        aria-label="最小化"
        @click="minimize"
      >
        <AppIcon name="minimize" :size="14" />
      </button>
      <button
        type="button"
        class="window-control"
        :title="maximizeLabel"
        :aria-label="maximizeLabel"
        @click="toggleMaximize"
      >
        <AppIcon :name="isMaximized ? 'restore' : 'maximize'" :size="13" />
      </button>
      <button
        type="button"
        class="window-control window-control--close"
        title="关闭"
        aria-label="关闭"
        @click="close"
      >
        <AppIcon name="x" :size="14" />
      </button>
    </div>
  </header>
</template>

<style scoped>
.title-bar {
  display: flex;
  flex: 0 0 36px;
  align-items: center;
  height: 36px;
  padding-left: 12px;
  background: var(--color-bg);
  border-bottom: 1px solid var(--color-border);
  user-select: none;
  -webkit-user-select: none;
  transition:
    background-color var(--transition-base),
    border-color var(--transition-base);
}

.title-bar__brand {
  display: flex;
  gap: 7px;
  align-items: center;
  min-width: 0;
}

.title-bar__mark {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 18px;
  height: 18px;
  color: var(--color-primary-contrast);
  background: var(--color-primary);
  border-radius: 4px;
}

.title-bar__title {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.01em;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.title-bar__spacer {
  flex: 1;
  align-self: stretch;
}

.window-controls {
  display: flex;
  align-self: stretch;
}

.window-control {
  display: grid;
  place-items: center;
  width: 46px;
  height: 100%;
  padding: 0;
  color: var(--color-text-secondary);
  background: transparent;
  border: none;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.window-control:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.window-control:active {
  background: var(--color-border);
}

.window-control:focus-visible {
  outline-offset: -2px;
}

.window-control--close:hover {
  color: #fff;
  background: #e81123;
}

.window-control--close:active {
  color: #fff;
  background: #c50f1f;
}
</style>
