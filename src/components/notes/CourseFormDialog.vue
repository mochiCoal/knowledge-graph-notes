<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import { useNotesStore } from '@/stores/notes'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const notes = useNotesStore()

const name = ref('')
const description = ref('')
const errorMessage = ref('')
const submitting = ref(false)
const nameInput = ref<HTMLInputElement | null>(null)

watch(
  () => props.open,
  (open) => {
    if (!open) return

    name.value = ''
    description.value = ''
    errorMessage.value = ''
    submitting.value = false
    void nextTick(() => nameInput.value?.focus())
  },
)

async function submit(): Promise<void> {
  if (submitting.value) return

  errorMessage.value = ''
  submitting.value = true

  try {
    await notes.createCourse(name.value, description.value)
    emit('close')
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <BaseModal :open="open" title="新建课程" @close="emit('close')">
    <form id="course-form" class="form-stack" @submit.prevent="submit">
      <label class="field">
        <span class="field__label">课程名称</span>
        <input
          ref="nameInput"
          v-model="name"
          class="input"
          type="text"
          maxlength="60"
          placeholder="例如：数据结构与算法"
        />
      </label>

      <label class="field">
        <span class="field__label">
          课程描述
          <span class="field__hint">可选</span>
        </span>
        <textarea
          v-model="description"
          class="textarea"
          rows="3"
          maxlength="500"
          placeholder="简单描述这门课程的内容"
        />
      </label>

      <p v-if="errorMessage" class="form-error">{{ errorMessage }}</p>
    </form>

    <template #footer>
      <button type="button" class="ghost-button" @click="emit('close')">取消</button>
      <button
        type="submit"
        form="course-form"
        class="primary-button"
        :disabled="submitting || !name.trim()"
      >
        {{ submitting ? '创建中…' : '创建课程' }}
      </button>
    </template>
  </BaseModal>
</template>
