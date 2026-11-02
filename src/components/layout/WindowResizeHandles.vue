<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

/** 与 @tauri-apps/api 的 ResizeDirection 保持一致（该类型未直接导出） */
type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West'

/**
 * 无边框窗口的边缘缩放手柄
 *
 * Tauri 的 `decorations: false` 会去掉系统标题栏和默认缩放边框，
 * 这里在窗口四边与四角放置透明手柄，按下时调用原生
 * `startResizeDragging`，保留与系统一致的缩放行为。
 */
const appWindow = isTauri() ? getCurrentWindow() : null
const isMaximized = ref(false)

let unlistenResized: (() => void) | null = null
let disposed = false

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

  const stop = await appWindow.onResized(() => {
    void syncMaximized()
  })
  if (disposed) {
    stop()
  } else {
    unlistenResized = stop
  }
})

onBeforeUnmount(() => {
  disposed = true
  unlistenResized?.()
  unlistenResized = null
})

interface ResizeHandle {
  direction: ResizeDirection
  className: string
}

const handles: ResizeHandle[] = [
  { direction: 'North', className: 'is-n' },
  { direction: 'South', className: 'is-s' },
  { direction: 'East', className: 'is-e' },
  { direction: 'West', className: 'is-w' },
  { direction: 'NorthEast', className: 'is-ne' },
  { direction: 'NorthWest', className: 'is-nw' },
  { direction: 'SouthEast', className: 'is-se' },
  { direction: 'SouthWest', className: 'is-sw' },
]

function startResize(event: PointerEvent, direction: ResizeDirection): void {
  if (!appWindow || isMaximized.value || event.button !== 0) return

  event.preventDefault()
  void appWindow.startResizeDragging(direction)
}
</script>

<template>
  <div v-if="!isMaximized" class="resize-handles" aria-hidden="true">
    <div
      v-for="handle in handles"
      :key="handle.direction"
      class="resize-handle"
      :class="handle.className"
      @pointerdown="startResize($event, handle.direction)"
    />
  </div>
</template>

<style scoped>
.resize-handles {
  position: fixed;
  inset: 0;
  z-index: 90;
  pointer-events: none;
}

.resize-handle {
  position: absolute;
  pointer-events: auto;
}

.is-n {
  top: 0;
  right: 8px;
  left: 8px;
  height: 4px;
  cursor: ns-resize;
}

.is-s {
  right: 8px;
  bottom: 0;
  left: 8px;
  height: 4px;
  cursor: ns-resize;
}

.is-e {
  top: 8px;
  right: 0;
  bottom: 8px;
  width: 4px;
  cursor: ew-resize;
}

.is-w {
  top: 8px;
  bottom: 8px;
  left: 0;
  width: 4px;
  cursor: ew-resize;
}

.is-ne {
  top: 0;
  right: 0;
  width: 10px;
  height: 10px;
  cursor: nesw-resize;
}

.is-nw {
  top: 0;
  left: 0;
  width: 10px;
  height: 10px;
  cursor: nwse-resize;
}

.is-se {
  right: 0;
  bottom: 0;
  width: 10px;
  height: 10px;
  cursor: nwse-resize;
}

.is-sw {
  bottom: 0;
  left: 0;
  width: 10px;
  height: 10px;
  cursor: nesw-resize;
}
</style>
