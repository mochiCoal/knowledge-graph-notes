<script setup lang="ts">
import BaseModal from './BaseModal.vue'

withDefaults(
  defineProps<{
    open: boolean
    title: string
    message: string
    confirmText?: string
    busy?: boolean
  }>(),
  {
    confirmText: '删除',
    busy: false,
  },
)

const emit = defineEmits<{ cancel: []; confirm: [] }>()
</script>

<template>
  <BaseModal :open="open" :title="title" :width="420" @close="emit('cancel')">
    <p class="confirm-message">{{ message }}</p>

    <template #footer>
      <button type="button" class="ghost-button" @click="emit('cancel')">取消</button>
      <button type="button" class="danger-button" :disabled="busy" @click="emit('confirm')">
        {{ busy ? '处理中…' : confirmText }}
      </button>
    </template>
  </BaseModal>
</template>

<style scoped>
.confirm-message {
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 1.7;
}
</style>
