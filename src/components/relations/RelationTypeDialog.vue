<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import { useRelationsStore } from '@/stores/relations'
import type { RelationType } from '@/types'

const props = defineProps<{ open: boolean }>()

const emit = defineEmits<{
  close: []
  changed: [message: string]
}>()

const store = useRelationsStore()

const editingId = ref<string | null>(null)
const label = ref('')
const color = ref('#337ea9')
const directed = ref(true)
const description = ref('')
const errorMessage = ref('')
const submitting = ref(false)

const pendingDelete = ref<RelationType | null>(null)
const deleting = ref(false)

const isEditing = computed(() => editingId.value !== null)
const title = computed(() => (isEditing.value ? '编辑关联类型' : '新增关联类型'))
const submitLabel = computed(() => {
  if (submitting.value) return isEditing.value ? '保存中…' : '创建中…'
  return isEditing.value ? '保存修改' : '创建类型'
})

watch(
  () => props.open,
  (open) => {
    if (!open) return
    resetForm()
  },
)

function resetForm(): void {
  editingId.value = null
  label.value = ''
  color.value = '#337ea9'
  directed.value = true
  description.value = ''
  errorMessage.value = ''
  submitting.value = false
  pendingDelete.value = null
  deleting.value = false
}

function startEdit(type: RelationType): void {
  editingId.value = type.id
  label.value = type.label
  color.value = type.color
  directed.value = type.directed
  description.value = type.description
  errorMessage.value = ''
  pendingDelete.value = null
}

async function submit(): Promise<void> {
  if (submitting.value) return

  const trimmed = label.value.trim()
  if (!trimmed) {
    errorMessage.value = '关联类型名称不能为空'
    return
  }

  errorMessage.value = ''
  submitting.value = true

  const payload = {
    label: trimmed,
    color: color.value,
    directed: directed.value,
    description: description.value,
  }

  try {
    if (editingId.value) {
      const updated = await store.updateType(editingId.value, payload)
      emit('changed', `已保存关联类型「${updated.label}」`)
    } else {
      const created = await store.createType(payload)
      emit('changed', `已创建关联类型「${created.label}」`)
    }

    resetForm()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    submitting.value = false
  }
}

function requestDelete(type: RelationType): void {
  errorMessage.value = ''
  pendingDelete.value = type
}

function cancelDelete(): void {
  if (deleting.value) return
  pendingDelete.value = null
}

async function confirmDelete(): Promise<void> {
  const target = pendingDelete.value
  if (!target || deleting.value) return

  deleting.value = true
  errorMessage.value = ''

  try {
    await store.deleteType(target.id)
    pendingDelete.value = null
    emit('changed', `已删除关联类型「${target.label}」`)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
    pendingDelete.value = null
  } finally {
    deleting.value = false
  }
}

function deleteHint(type: RelationType): string {
  if (type.is_builtin) return '内置类型不能删除，可以修改名称或颜色'
  if (type.usage_count > 0) return `已有 ${type.usage_count} 条关联使用该类型，不能删除`
  return `删除关联类型「${type.label}」`
}
</script>

<template>
  <BaseModal :open="open" title="关联类型管理" :width="660" @close="emit('close')">
    <div class="type-manager">
      <p class="type-manager__intro">
        关联类型决定关联的语义：有向类型用于前置、包含、延伸，无向类型用于相关、对比。
        图谱阶段会按这里的颜色绘制关联边。
      </p>

      <form class="type-form" @submit.prevent="submit">
        <header class="type-form__head">
          <h3>{{ title }}</h3>
          <button
            v-if="isEditing"
            type="button"
            class="text-button"
            @click="resetForm"
          >
            取消编辑
          </button>
        </header>

        <div class="type-form__grid">
          <label class="field">
            <span class="field__label">类型名称</span>
            <input
              v-model="label"
              class="input"
              type="text"
              maxlength="20"
              placeholder="例如：易混淆、实践应用"
            />
          </label>

          <label class="field">
            <span class="field__label">边颜色</span>
            <span class="color-field">
              <input v-model="color" class="color-field__picker" type="color" />
              <span class="color-field__value">{{ color }}</span>
            </span>
          </label>
        </div>

        <div class="type-form__grid">
          <label class="field">
            <span class="field__label">方向</span>
            <span class="direction-options">
              <label class="radio-option">
                <input v-model="directed" type="radio" :value="true" />
                有向（A → B）
              </label>
              <label class="radio-option">
                <input v-model="directed" type="radio" :value="false" />
                无向（A ↔ B）
              </label>
            </span>
          </label>

          <label class="field">
            <span class="field__label">类型说明</span>
            <input
              v-model="description"
              class="input"
              type="text"
              maxlength="200"
              placeholder="可选，说明该类型的使用场景"
            />
          </label>
        </div>

        <p v-if="errorMessage" class="form-error">{{ errorMessage }}</p>

        <div class="type-form__actions">
          <button type="submit" class="primary-button" :disabled="submitting || !label.trim()">
            {{ submitLabel }}
          </button>
        </div>
      </form>

      <section class="type-list" aria-label="关联类型列表">
        <header class="type-list__head">
          <h3>全部类型</h3>
          <span class="count-badge">{{ store.types.length }}</span>
        </header>

        <ul class="type-list__items">
          <li v-for="type in store.types" :key="type.id" class="type-item">
            <span class="type-item__dot" :style="{ background: type.color }" />
            <div class="type-item__body">
              <div class="type-item__title-row">
                <span class="type-item__label">{{ type.label }}</span>
                <span class="mini-badge">{{ type.directed ? '有向' : '无向' }}</span>
                <span class="mini-badge mini-badge--muted">
                  {{ type.is_builtin ? '内置' : '自定义' }}
                </span>
                <span class="type-item__usage">{{ type.usage_count }} 条关联</span>
              </div>
              <p v-if="type.description" class="type-item__desc">{{ type.description }}</p>
            </div>

            <div class="type-item__actions">
              <button
                type="button"
                class="icon-button"
                :title="`编辑「${type.label}」`"
                :aria-label="`编辑「${type.label}」`"
                @click="startEdit(type)"
              >
                <AppIcon name="pencil" :size="14" />
              </button>
              <button
                type="button"
                class="icon-button type-item__delete"
                :title="deleteHint(type)"
                :aria-label="`删除「${type.label}」`"
                :disabled="type.is_builtin || type.usage_count > 0"
                @click="requestDelete(type)"
              >
                <AppIcon name="trash" :size="14" />
              </button>
            </div>

            <div v-if="pendingDelete?.id === type.id" class="type-confirm">
              <span>确定删除「{{ type.label }}」吗？删除后无法恢复。</span>
              <button type="button" class="ghost-button" :disabled="deleting" @click="cancelDelete">
                取消
              </button>
              <button
                type="button"
                class="danger-button"
                :disabled="deleting"
                @click="confirmDelete"
              >
                {{ deleting ? '删除中…' : '删除' }}
              </button>
            </div>
          </li>
        </ul>
      </section>
    </div>

    <template #footer>
      <button type="button" class="ghost-button" @click="emit('close')">关闭</button>
    </template>
  </BaseModal>
</template>

<style scoped>
.type-manager {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.type-manager__intro {
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1.7;
}

.type-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.type-form__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.type-form__head h3 {
  font-size: 12.5px;
  font-weight: 600;
}

.type-form__grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.type-form__actions {
  display: flex;
  justify-content: flex-end;
}

.color-field {
  display: flex;
  gap: 8px;
  align-items: center;
  height: 35px;
  padding: 3px 10px 3px 4px;
  background: var(--color-surface);
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
}

.color-field__picker {
  width: 28px;
  height: 26px;
  padding: 0;
  background: transparent;
  border: none;
  cursor: pointer;
}

.color-field__value {
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  font-size: 11.5px;
  text-transform: lowercase;
}

.direction-options {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 14px;
  align-items: center;
  min-height: 35px;
}

.radio-option {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  color: var(--color-text-secondary);
  font-size: 12px;
  cursor: pointer;
}

.radio-option input {
  accent-color: var(--color-accent);
}

.type-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.type-list__head {
  display: flex;
  gap: 8px;
  align-items: center;
}

.type-list__head h3 {
  font-size: 12px;
  font-weight: 600;
}

.count-badge {
  padding: 0 7px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  line-height: 18px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.type-list__items {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: min(38vh, 320px);
  padding-right: 2px;
  overflow-y: auto;
}

.type-item {
  position: relative;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: flex-start;
  padding: 10px 10px 10px 12px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.type-item__dot {
  flex: 0 0 auto;
  width: 8px;
  height: 8px;
  margin-top: 5px;
  border-radius: 50%;
}

.type-item__body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.type-item__title-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.type-item__label {
  font-size: 12.5px;
  font-weight: 600;
}

.mini-badge {
  padding: 0 6px;
  color: var(--color-accent-strong);
  font-size: 10px;
  line-height: 16px;
  background: var(--color-accent-soft);
  border-radius: 999px;
}

.mini-badge--muted {
  color: var(--color-text-muted);
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
}

.type-item__usage {
  color: var(--color-text-muted);
  font-size: 10.5px;
}

.type-item__desc {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.55;
  text-overflow: ellipsis;
}

.type-item__actions {
  display: flex;
  flex: 0 0 auto;
  gap: 2px;
}

.type-item__delete:disabled {
  cursor: not-allowed;
  opacity: 0.35;
}

.type-item__delete:not(:disabled):hover {
  color: var(--color-danger);
  background: var(--color-danger-soft);
}

.type-confirm {
  display: flex;
  grid-column: 1 / -1;
  flex-basis: 100%;
  gap: 8px;
  align-items: center;
  width: 100%;
  padding-top: 9px;
  margin-top: 2px;
  color: var(--color-text-secondary);
  font-size: 11.5px;
  border-top: 1px dashed var(--color-border);
}

.type-confirm .ghost-button,
.type-confirm .danger-button {
  height: 28px;
  margin-left: auto;
  padding: 0 10px;
  font-size: 11.5px;
}

.type-confirm .danger-button {
  margin-left: 0;
}

.text-button {
  padding: 0;
  color: var(--color-accent-strong);
  font-size: 11.5px;
  background: none;
  border: none;
}

.text-button:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
}

@media (max-width: 620px) {
  .type-form__grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
