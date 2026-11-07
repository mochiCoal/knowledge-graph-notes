<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import { useRelationsStore } from '@/stores/relations'
import type { RelationDetail } from '@/types'

const props = withDefaults(
  defineProps<{
    open: boolean
    /** 传入关联表示编辑；为空表示新建 */
    relation?: RelationDetail | null
    /** 新建时预选的源知识点 */
    defaultSourceId?: string | null
  }>(),
  {
    relation: null,
    defaultSourceId: null,
  },
)

const emit = defineEmits<{
  close: []
  saved: [relation: RelationDetail]
}>()

const store = useRelationsStore()

const sourceId = ref('')
const targetId = ref('')
const typeId = ref('')
const description = ref('')
const errorMessage = ref('')
const submitting = ref(false)

const isEditing = computed(() => props.relation !== null)
const title = computed(() => (isEditing.value ? '编辑知识点关联' : '新建知识点关联'))
const submitLabel = computed(() => {
  if (submitting.value) return isEditing.value ? '保存中…' : '创建中…'
  return isEditing.value ? '保存修改' : '创建关联'
})

const selectedType = computed(
  () => store.types.find((type) => type.id === typeId.value) ?? null,
)

const directionHint = computed(() => {
  if (!selectedType.value) return '请选择关联类型'
  return selectedType.value.directed
    ? '有向关联：源知识点 → 目标知识点'
    : '无向关联：两个知识点互为关联，方向不影响含义'
})

const canSubmit = computed(
  () =>
    !submitting.value &&
    typeId.value !== '' &&
    (isEditing.value
      ? true
      : sourceId.value !== '' &&
        targetId.value !== '' &&
        sourceId.value !== targetId.value),
)

/** 按课程分组的知识点，方便在两个下拉框中快速定位 */
const pointsByCourse = computed(() =>
  store.courses.map((course) => ({
    course,
    points: store.allPoints.filter((point) => point.course_id === course.id),
  })),
)

watch(
  [() => props.open, () => props.relation?.id],
  ([open]) => {
    if (!open) return

    errorMessage.value = ''
    submitting.value = false

    if (props.relation) {
      sourceId.value = props.relation.source_id
      targetId.value = props.relation.target_id
      typeId.value = props.relation.relation_type
      description.value = props.relation.description
      return
    }

    const points = store.coursePoints.length ? store.coursePoints : store.allPoints
    const preferred = props.defaultSourceId ?? store.selectedPointId
    const source = points.find((point) => point.id === preferred) ?? points[0] ?? null

    sourceId.value = source?.id ?? ''
    targetId.value = points.find((point) => point.id !== source?.id)?.id ?? ''
    typeId.value =
      store.types.find((type) => type.id === 'related')?.id ?? store.types[0]?.id ?? ''
    description.value = ''
  },
)

watch(sourceId, (value) => {
  if (targetId.value === value) {
    targetId.value = ''
  }
})

async function submit(): Promise<void> {
  if (!canSubmit.value) return

  errorMessage.value = ''
  submitting.value = true

  try {
    const saved = props.relation
      ? await store.updateRelation(props.relation.id, {
          relationType: typeId.value,
          description: description.value,
        })
      : await store.createRelation({
          sourceId: sourceId.value,
          targetId: targetId.value,
          relationType: typeId.value,
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
  <BaseModal :open="open" :title="title" :width="520" @close="emit('close')">
    <form id="relation-form" class="form-stack" @submit.prevent="submit">
      <div v-if="isEditing && relation" class="endpoint-summary">
        <span class="endpoint-summary__point" :title="relation.source_title">
          {{ relation.source_title }}
        </span>
        <span class="endpoint-summary__arrow">
          {{ selectedType?.directed ? '→' : '↔' }}
        </span>
        <span class="endpoint-summary__point" :title="relation.target_title">
          {{ relation.target_title }}
        </span>
      </div>

      <template v-else>
        <label class="field">
          <span class="field__label">
            源知识点
            <span class="field__hint">关联的起点</span>
          </span>
          <select v-model="sourceId" class="select">
            <option value="" disabled>请选择源知识点</option>
            <optgroup
              v-for="{ course, points } in pointsByCourse"
              :key="course.id"
              :label="course.name"
            >
              <option v-for="point in points" :key="point.id" :value="point.id">
                {{ point.title }}
              </option>
            </optgroup>
          </select>
        </label>

        <label class="field">
          <span class="field__label">
            目标知识点
            <span class="field__hint">不能与源知识点相同</span>
          </span>
          <select v-model="targetId" class="select">
            <option value="" disabled>请选择目标知识点</option>
            <optgroup
              v-for="{ course, points } in pointsByCourse"
              :key="course.id"
              :label="course.name"
            >
              <option
                v-for="point in points"
                :key="point.id"
                :value="point.id"
                :disabled="point.id === sourceId"
              >
                {{ point.title }}
              </option>
            </optgroup>
          </select>
        </label>
      </template>

      <label class="field">
        <span class="field__label">
          关联类型
          <span class="field__hint">{{ directionHint }}</span>
        </span>
        <select v-model="typeId" class="select">
          <option value="" disabled>请选择关联类型</option>
          <option v-for="type in store.types" :key="type.id" :value="type.id">
            {{ type.label }}{{ type.directed ? '（有向）' : '（无向）' }}
          </option>
        </select>
      </label>

      <label class="field">
        <span class="field__label">
          关联描述
          <span class="field__hint">可选，最多 500 字</span>
        </span>
        <textarea
          v-model="description"
          class="textarea"
          maxlength="500"
          rows="3"
          placeholder="例如：先掌握链表遍历，再学习双向链表的插入与删除"
        />
      </label>

      <p v-if="store.allPoints.length < 2" class="form-hint">
        至少需要两个知识点才能建立关联，请先到知识点管理中补充。
      </p>
      <p v-if="!store.types.length" class="form-hint">
        当前没有可用的关联类型，请先在「关联类型」中创建。
      </p>
      <p v-if="errorMessage" class="form-error">{{ errorMessage }}</p>
    </form>

    <template #footer>
      <button type="button" class="ghost-button" @click="emit('close')">取消</button>
      <button
        type="submit"
        form="relation-form"
        class="primary-button"
        :disabled="!canSubmit"
      >
        {{ submitLabel }}
      </button>
    </template>
  </BaseModal>
</template>

<style scoped>
.endpoint-summary {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 10px 12px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.endpoint-summary__point {
  flex: 1;
  overflow: hidden;
  font-size: 13px;
  font-weight: 500;
  text-align: center;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.endpoint-summary__arrow {
  flex: 0 0 auto;
  color: var(--color-accent-strong);
  font-size: 15px;
}

.form-hint {
  color: var(--color-text-muted);
  font-size: 11.5px;
  line-height: 1.6;
}
</style>
