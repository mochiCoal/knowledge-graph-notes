<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'

withDefaults(
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

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') emit('close')
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))
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
