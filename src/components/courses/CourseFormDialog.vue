<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import { useCoursesStore } from '@/stores/courses'
import type { Course } from '@/types'

const props = withDefaults(
  defineProps<{
    open: boolean
    /** 传入课程表示编辑，为空表示新建 */
    course?: Course | null
  }>(),
  {
    course: null,
  },
)

const emit = defineEmits<{
  close: []
  saved: [course: Course]
}>()

const courses = useCoursesStore()

const name = ref('')
const description = ref('')
const errorMessage = ref('')
const submitting = ref(false)
const nameInput = ref<HTMLInputElement | null>(null)

const isEditing = computed(() => props.course !== null)
const title = computed(() => (isEditing.value ? '编辑课程' : '新建课程'))
const submitLabel = computed(() => {
  if (submitting.value) return isEditing.value ? '保存中…' : '创建中…'
  return isEditing.value ? '保存修改' : '创建课程'
})

watch(
  [() => props.open, () => props.course?.id],
  ([open]) => {
    if (!open) return

    name.value = props.course?.name ?? ''
    description.value = props.course?.description ?? ''
    errorMessage.value = ''
    submitting.value = false
    void nextTick(() => nameInput.value?.focus())
  },
)

async function submit(): Promise<void> {
  if (submitting.value) return

  const trimmedName = name.value.trim()
  if (!trimmedName) {
    errorMessage.value = '课程名称不能为空'
    return
  }

  errorMessage.value = ''
  submitting.value = true

  try {
    const saved = props.course
      ? await courses.updateCourse(props.course.id, {
          name: trimmedName,
          description: description.value,
        })
      : await courses.createCourse({
          name: trimmedName,
          description: description.value,
        })

    emit('saved', saved)
    emit('close')
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <BaseModal :open="open" :title="title" @close="emit('close')">
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
          <span class="field__hint">可选，最多 500 字</span>
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
        {{ submitLabel }}
      </button>
    </template>
  </BaseModal>
</template>
