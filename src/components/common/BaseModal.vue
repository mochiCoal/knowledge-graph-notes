<script lang="ts">
/**
 * 模块级弹窗打开顺序栈（所有 BaseModal 实例共享）。
 * 组件常挂载、`open` 切换显隐，也可能多层叠加（例如表单弹窗上再开确认框）。
 */
const openStack: symbol[] = []
</script>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'

const props = withDefaults(
  defineProps<{
    open: boolean
    title: string
    /** 卡片最大宽度 */
    width?: number
  }>(),
  {
    width: 460,
  },
)

const emit = defineEmits<{ close: [] }>()

const modalId = Symbol('modal')

function syncOpenState(open: boolean): void {
  const index = openStack.indexOf(modalId)
  if (open && index === -1) openStack.push(modalId)
  if (!open && index !== -1) openStack.splice(index, 1)
}

watch(() => props.open, syncOpenState, { immediate: true })

// 只有位于栈顶（最上层）的弹窗才响应 Esc，
// 避免按一次 Esc 关掉所有弹窗、或关闭根本没打开的弹窗。
function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape') return
  if (openStack[openStack.length - 1] !== modalId) return
  emit('close')
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKeydown)
  syncOpenState(false)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="open" class="modal-backdrop" @mousedown.self="emit('close')">
        <div
          class="modal-card"
          role="dialog"
          aria-modal="true"
          :aria-label="title"
          :style="{ maxWidth: `${width}px` }"
        >
          <header class="modal-header">
            <h2>{{ title }}</h2>
            <button type="button" class="icon-button" aria-label="关闭" @click="emit('close')">
              <AppIcon name="x" :size="16" />
            </button>
          </header>

          <div class="modal-body">
            <slot />
          </div>

          <footer v-if="$slots.footer" class="modal-footer">
            <slot name="footer" />
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: grid;
  place-items: center;
  padding: 24px;
  background: var(--color-overlay);
}

.modal-card {
  width: 100%;
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-pop);
}

.modal-header {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px 14px 18px;
  border-bottom: 1px solid var(--color-border);
}

.modal-header h2 {
  font-size: 14.5px;
  font-weight: 600;
}

.modal-body {
  padding: 18px;
}

.modal-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding: 0 18px 18px;
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 150ms ease;
}

.modal-enter-active .modal-card,
.modal-leave-active .modal-card {
  transition:
    transform 150ms ease,
    opacity 150ms ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-card,
.modal-leave-to .modal-card {
  opacity: 0;
  transform: translateY(6px) scale(0.99);
}
</style>
