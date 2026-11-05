<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import { useKnowledgePointsStore } from '@/stores/knowledgePoints'
import type { KnowledgePoint, KnowledgePointNode } from '@/types'

const props = withDefaults(
  defineProps<{
    open: boolean
    /** 归属课程名称，仅用于提示 */
    courseName?: string
    /** 当前课程的知识点树，用于生成父知识点选项 */
    tree?: KnowledgePointNode[]
    /** 传入知识点表示编辑，为空表示新建 */
    point?: KnowledgePoint | null
    /** 新建时默认选中的父知识点 */
    defaultParentId?: string | null
  }>(),
  {
    courseName: '',
    tree: () => [],
    point: null,
    defaultParentId: null,
  },
)

const emit = defineEmits<{
  close: []
  saved: [point: KnowledgePoint]
}>()

const store = useKnowledgePointsStore()

const title = ref('')
const description = ref('')
const parentId = ref('')
const errorMessage = ref('')
const submitting = ref(false)
const titleInput = ref<HTMLInputElement | null>(null)

const isEditing = computed(() => props.point !== null)
const modalTitle = computed(() => (isEditing.value ? '编辑知识点' : '新建知识点'))
const submitLabel = computed(() => {
  if (submitting.value) return isEditing.value ? '保存中…' : '创建中…'
  return isEditing.value ? '保存修改' : '创建知识点'
})

interface ParentOption {
  id: string
  label: string
}

/** 编辑时要把自己和自己的后代从父节点候选中排除，避免形成环 */
const excludedIds = computed(() => {
  const excluded = new Set<string>()
  if (!props.point) return excluded

  const collect = (node: KnowledgePointNode): void => {
    excluded.add(node.id)
    node.children.forEach(collect)
  }

  const find = (nodes: KnowledgePointNode[]): KnowledgePointNode | null => {
    for (const node of nodes) {
      if (node.id === props.point?.id) return node
      const found = find(node.children)
      if (found) return found
    }
    return null
  }

  const self = find(props.tree)
  if (self) {
    collect(self)
  } else {
    excluded.add(props.point.id)
  }

  return excluded
})

const parentOptions = computed<ParentOption[]>(() => {
  const options: ParentOption[] = []

  const walk = (nodes: KnowledgePointNode[], depth: number): void => {
    for (const node of nodes) {
      if (excludedIds.value.has(node.id)) continue

      const indent = depth > 0 ? `${'· '.repeat(depth)}` : ''
      options.push({ id: node.id, label: `${indent}${node.title}` })
      walk(node.children, depth + 1)
    }
  }

  walk(props.tree, 0)
  return options
})

watch(
  [() => props.open, () => props.point?.id],
  ([open]) => {
    if (!open) return

    title.value = props.point?.title ?? ''
    description.value = props.point?.description ?? ''
    parentId.value = props.point?.parent_id ?? props.defaultParentId ?? ''
    errorMessage.value = ''
    submitting.value = false
    void nextTick(() => titleInput.value?.focus())
  },
)

async function submit(): Promise<void> {
  if (submitting.value) return

  const trimmedTitle = title.value.trim()
  if (!trimmedTitle) {
    errorMessage.value = '知识点名称不能为空'
    return
  }

  errorMessage.value = ''
  submitting.value = true

  const input = {
    title: trimmedTitle,
    description: description.value,
    parentId: parentId.value || null,
  }

  try {
    const saved = props.point
      ? await store.updatePoint(props.point.id, input)
      : await store.createPoint(input)

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
  <BaseModal :open="open" :title="modalTitle" @close="emit('close')">
    <form id="knowledge-point-form" class="form-stack" @submit.prevent="submit">
      <p v-if="courseName" class="course-note">
        归属课程
        <strong>{{ courseName }}</strong>
        <span>（可在详情面板迁移到其它课程）</span>
      </p>

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
          <span class="field__hint">可选，最多 1000 字</span>
        </span>
        <textarea
          v-model="description"
          class="textarea"
          rows="3"
          maxlength="1000"
          placeholder="一句话说明这个知识点要掌握的内容"
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
        :disabled="submitting || !title.trim()"
      >
        {{ submitLabel }}
      </button>
    </template>
  </BaseModal>
</template>

<style scoped>
.course-note {
  padding: 8px 11px;
  color: var(--color-text-muted);
  font-size: 11.5px;
  line-height: 1.6;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.course-note strong {
  color: var(--color-text-secondary);
  font-weight: 600;
}
</style>
