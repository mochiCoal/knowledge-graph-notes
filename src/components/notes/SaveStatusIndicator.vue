<script setup lang="ts">
import { computed } from 'vue'
import { formatTime } from '@/utils/format'
import type { SaveStatus } from '@/types'

const props = defineProps<{
  status: SaveStatus
  savedAt: Date | null
}>()

const emit = defineEmits<{ retry: [] }>()

const label = computed(() => {
  switch (props.status) {
    case 'dirty':
      return '未保存'
    case 'saving':
      return '保存中'
    case 'saved':
      return '已保存'
    case 'error':
      return '保存失败'
    default:
      return ''
  }
})

const time = computed(() =>
  props.status === 'saved' && props.savedAt ? formatTime(props.savedAt) : '',
)
</script>

<template>
  <span
    v-if="label"
    class="save-status"
    :class="`save-status--${status}`"
    role="status"
    aria-live="polite"
  >
    <span class="save-status__dot" aria-hidden="true" />
    <span>{{ label }}</span>
    <span v-if="time" class="save-status__time">{{ time }}</span>
    <button
      v-if="status === 'error'"
      type="button"
      class="save-status__retry"
      @click="emit('retry')"
    >
      重试
    </button>
  </span>
</template>

<style scoped>
.save-status {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 5px;
  align-items: center;
  color: var(--color-text-muted);
  font-size: 11.5px;
  white-space: nowrap;
}

.save-status__dot {
  width: 6px;
  height: 6px;
  background: currentColor;
  border-radius: 50%;
}

.save-status--saving {
  color: var(--color-accent);
}

.save-status--saving .save-status__dot {
  animation: save-pulse 1s ease-in-out infinite;
}

.save-status--saved {
  color: var(--color-success);
}

.save-status--error {
  color: var(--color-danger);
}

.save-status__time {
  color: var(--color-text-muted);
}

.save-status__retry {
  padding: 0;
  color: var(--color-accent);
  font-size: 11.5px;
  text-decoration: underline;
  background: none;
  border: none;
}

@keyframes save-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.3;
  }
}
</style>
