<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import BaseModal from '@/components/common/BaseModal.vue'
import { useTagsStore } from '@/stores/tags'
import type { Tag, TagSummary } from '@/types'

const props = withDefaults(
  defineProps<{
    open: boolean
    /** 传入标签表示重命名，为空表示新建 */
    tag?: TagSummary | null
  }>(),
  {
    tag: null,
  },
)

const emit = defineEmits<{
  close: []
  saved: [tag: Tag]
}>()

const store = useTagsStore()

const name = ref('')
const errorMessage = ref('')
const submitting = ref(false)
const nameInput = ref<HTMLInputElement | null>(null)

const isEditing = computed(() => props.tag !== null)
const title = computed(() => (isEditing.value ? '重命名标签' : '新建标签'))
const submitLabel = computed(() => {
  if (submitting.value) return isEditing.value ? '保存中…' : '创建中…'
  return isEditing.value ? '保存修改' : '创建标签'
})

watch(
  [() => props.open, () => props.tag?.id],
  ([open]) => {
    if (!open) return

    name.value = props.tag?.name ?? ''
    errorMessage.value = ''
    submitting.value = false
    void nextTick(() => nameInput.value?.focus())
  },
)

async function submit(): Promise<void> {
  if (submitting.value) return

  const trimmed = name.value.trim()
  if (!trimmed) {
    errorMessage.value = '标签名称不能为空'
    return
  }

  errorMessage.value = ''
  submitting.value = true

  try {
    const saved = props.tag
      ? await store.renameTag(props.tag.id, trimmed)
      : await store.createTag(trimmed)

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
    <form id="tag-form" class="form-stack" @submit.prevent="submit">
      <label class="field">
        <span class="field__label">
          标签名称
          <span class="field__hint">
            {{ isEditing ? '需与其它标签不同' : '同名标签会复用已有标签' }}
          </span>
        </span>
        <input
          ref="nameInput"
          v-model="name"
          class="input"
          type="text"
          maxlength="30"
          placeholder="例如：重点、面试、易错"
        />
      </label>

      <p v-if="errorMessage" class="form-error">{{ errorMessage }}</p>
    </form>

    <template #footer>
      <button type="button" class="ghost-button" @click="emit('close')">取消</button>
      <button
        type="submit"
        form="tag-form"
        class="primary-button"
        :disabled="submitting || !name.trim()"
      >
        {{ submitLabel }}
      </button>
    </template>
  </BaseModal>
</template>
