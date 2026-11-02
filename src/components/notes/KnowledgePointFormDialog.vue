<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import { useNotesStore } from '@/stores/notes'
import type { KnowledgePoint } from '@/types'

const props = defineProps<{
  open: boolean
  /** 打开时默认选中的课程 */
  courseId?: string | null
  /** 打开时默认选中的父知识点 */
  parentId?: string | null
}>()

const emit = defineEmits<{ close: [] }>()

const notes = useNotesStore()

const courseId = ref('')
const parentId = ref('')
const title = ref('')
const description = ref('')
const errorMessage = ref('')
const submitting = ref(false)
const titleInput = ref<HTMLInputElement | null>(null)

const parentOptions = computed(() => {
  if (!courseId.value) return []
  return notes
    .pointsOfCourse(courseId.value)
    .map((point) => ({ id: point.id, label: pathLabel(point) }))
})

/** 用手写路径生成下拉选项文案，例如「线性表 / 链表」 */
function pathLabel(point: KnowledgePoint): string {
  const byId = new Map(notes.knowledgePoints.map((item) => [item.id, item] as const))
  const parts: string[] = []
  const visited = new Set<string>()
  let current: KnowledgePoint | null = point

  while (current && !visited.has(current.id)) {
    visited.add(current.id)
    parts.unshift(current.title)
    const parentId: string | null = current.parent_id
    current = parentId ? (byId.get(parentId) ?? null) : null
  }

  return parts.join(' / ')
}

watch(
  () => props.open,
  (open) => {
    if (!open) return

    courseId.value = props.courseId ?? notes.selectedPoint?.course_id ?? notes.courses[0]?.id ?? ''
    parentId.value = props.parentId ?? ''
    title.value = ''
    description.value = ''
    errorMessage.value = ''
    submitting.value = false
    void nextTick(() => titleInput.value?.focus())
  },
)

// 切换课程后，清空不属于该课程的父知识点
watch(courseId, (nextCourseId) => {
  if (!parentId.value) return
  const parent = notes.knowledgePoints.find((point) => point.id === parentId.value)
  if (!parent || parent.course_id !== nextCourseId) {
    parentId.value = ''
  }
})

async function submit(): Promise<void> {
  if (submitting.value) return

  errorMessage.value = ''
  submitting.value = true

  try {
    await notes.createKnowledgePoint({
      courseId: courseId.value,
      parentId: parentId.value || null,
      title: title.value,
      description: description.value,
    })
    emit('close')
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <BaseModal :open="open" title="新建知识点" @close="emit('close')">
    <form id="knowledge-point-form" class="form-stack" @submit.prevent="submit">
      <label class="field">
        <span class="field__label">所属课程</span>
        <select v-model="courseId" class="select">
          <option v-if="!notes.courses.length" value="">暂无课程</option>
          <option v-for="course in notes.courses" :key="course.id" :value="course.id">
            {{ course.name }}
          </option>
        </select>
      </label>

      <label class="field">
        <span class="field__label">
          父知识点
          <span class="field__hint">不选择则作为顶层知识点</span>
        </span>
        <select v-model="parentId" class="select">
          <option value="">作为顶层知识点</option>
          <option v-for="option in parentOptions" :key="option.id" :value="option.id">
            {{ option.label }}
          </option>
        </select>
      </label>

      <label class="field">
        <span class="field__label">知识点名称</span>
        <input
          ref="titleInput"
          v-model="title"
          class="input"
          type="text"
          maxlength="80"
          placeholder="例如：链表"
        />
      </label>

      <label class="field">
        <span class="field__label">
          知识点描述
          <span class="field__hint">可选</span>
        </span>
        <textarea
          v-model="description"
          class="textarea"
          rows="2"
          maxlength="1000"
          placeholder="一句话说明这个知识点"
        />
      </label>

      <p v-if="errorMessage" class="form-error">{{ errorMessage }}</p>
    </form>

    <template #footer>
      <button type="button" class="ghost-button" @click="emit('close')">取消</button>
      <button
        type="submit"
        form="knowledge-point-form"
        class="primary-button"
        :disabled="submitting || !title.trim() || !courseId"
      >
        {{ submitting ? '创建中…' : '创建知识点' }}
      </button>
    </template>
  </BaseModal>
</template>
