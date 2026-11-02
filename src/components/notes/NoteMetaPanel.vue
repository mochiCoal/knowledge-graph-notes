<script setup lang="ts">
import { computed, ref } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import { useNotesStore } from '@/stores/notes'
import { useWorkspaceStore } from '@/stores/workspace'
import { formatDateTime } from '@/utils/format'
import type { Tag } from '@/types'

const notes = useNotesStore()
const workspace = useWorkspaceStore()

const tagInput = ref('')
const tagError = ref('')
const removingTagId = ref<string | null>(null)

const hasSelection = computed(() => notes.selectedPoint !== null)

const updatedLabel = computed(() =>
  notes.note ? formatDateTime(notes.note.updated_at) : '尚未保存',
)

const RELATION_LABELS: Record<string, string> = {
  prerequisite: '前置知识',
  related: '相关知识',
  contains: '包含关系',
}

function relationLabel(type: string): string {
  return RELATION_LABELS[type] ?? type
}

async function submitTag(): Promise<void> {
  const value = tagInput.value.trim()
  if (!value) return

  tagError.value = ''

  try {
    await notes.addTagToNote(value)
    tagInput.value = ''
  } catch (error) {
    tagError.value = error instanceof Error ? error.message : String(error)
  }
}

async function removeTag(tag: Tag): Promise<void> {
  if (removingTagId.value) return

  removingTagId.value = tag.id
  tagError.value = ''

  try {
    await notes.removeTagFromNote(tag)
  } catch (error) {
    tagError.value = error instanceof Error ? error.message : String(error)
  } finally {
    removingTagId.value = null
  }
}
</script>

<template>
  <aside class="note-meta" aria-label="笔记辅助信息">
    <header class="note-meta__header">
      <h2>笔记信息</h2>
      <button
        type="button"
        class="icon-button"
        title="隐藏辅助面板"
        aria-label="隐藏辅助面板"
        @click="workspace.toggleMeta()"
      >
        <AppIcon name="x" :size="15" />
      </button>
    </header>

    <div v-if="!hasSelection" class="empty-panel note-meta__empty">
      <div class="empty-panel__icon">
        <AppIcon name="tag" :size="18" />
      </div>
      <h3>未选择知识点</h3>
      <p>选择知识点后可以查看所属课程、标签与相关知识点。</p>
    </div>

    <div v-else class="note-meta__body">
      <section class="meta-section">
        <h3>所属知识点</h3>
        <div class="point-card">
          <div class="point-card__title">{{ notes.selectedPoint?.title }}</div>
          <div class="point-card__course">{{ notes.selectedCourse?.name }}</div>
          <p v-if="notes.selectedPoint?.description" class="point-card__description">
            {{ notes.selectedPoint.description }}
          </p>
        </div>
        <p class="meta-stat">
          更新于 {{ updatedLabel }} · {{ notes.characterCount }} 字
        </p>
      </section>

      <section class="meta-section">
        <h3>标签</h3>

        <div v-if="notes.noteTags.length" class="tag-list">
          <span v-for="tag in notes.noteTags" :key="tag.id" class="tag-chip">
            {{ tag.name }}
            <button
              type="button"
              class="tag-chip__remove"
              :title="`移除标签 ${tag.name}`"
              :aria-label="`移除标签 ${tag.name}`"
              :disabled="removingTagId === tag.id"
              @click="removeTag(tag)"
            >
              <AppIcon name="x" :size="11" />
            </button>
          </span>
        </div>
        <p v-else class="meta-hint">还没有标签，输入名称即可添加。</p>

        <form class="tag-form" @submit.prevent="submitTag">
          <input
            v-model="tagInput"
            class="input tag-form__input"
            type="text"
            maxlength="30"
            list="note-tag-options"
            placeholder="添加标签…"
            aria-label="添加标签"
          />
          <datalist id="note-tag-options">
            <option v-for="tag in notes.allTags" :key="tag.id" :value="tag.name" />
          </datalist>
          <button
            type="submit"
            class="ghost-button tag-form__submit"
            :disabled="!tagInput.trim()"
          >
            添加
          </button>
        </form>
        <p v-if="tagError" class="form-error">{{ tagError }}</p>
      </section>

      <section class="meta-section">
        <h3>相关知识点</h3>

        <ul v-if="notes.relatedItems.length" class="related-list">
          <li v-for="item in notes.relatedItems" :key="item.relation.id">
            <button
              type="button"
              class="related-item"
              @click="notes.selectPoint(item.point.id)"
            >
              <span class="related-item__title">{{ item.point.title }}</span>
              <span class="related-item__type">
                {{ relationLabel(item.relation.relation_type) }}
              </span>
            </button>
          </li>
        </ul>
        <p v-else class="meta-hint">暂无关联知识点，关联功能将在阶段六实现。</p>
      </section>

      <section class="meta-section">
        <h3>关联建议</h3>
        <p class="meta-hint">关联建议会根据笔记内容自动生成，计划在阶段六接入。</p>
      </section>
    </div>
  </aside>
</template>

<style scoped>
.note-meta {
  display: flex;
  flex: 0 0 288px;
  flex-direction: column;
  width: 288px;
  height: 100%;
  overflow: hidden;
  background: var(--color-surface);
  border-left: 1px solid var(--color-border);
  transition:
    background-color var(--transition-base),
    border-color var(--transition-base);
}

.note-meta__header {
  display: flex;
  flex: 0 0 auto;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
  height: 50px;
  padding: 0 10px 0 16px;
  border-bottom: 1px solid var(--color-border);
}

.note-meta__header h2 {
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.note-meta__body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 22px;
  padding: 18px 16px 40px;
  overflow-y: auto;
}

.note-meta__empty {
  padding-top: 40px;
}

.meta-section h3 {
  margin-bottom: 8px;
  color: var(--color-text-muted);
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.04em;
}

.point-card {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 10px 12px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.point-card__title {
  font-size: 13px;
  font-weight: 500;
}

.point-card__course {
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.point-card__description {
  margin-top: 6px;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

.meta-stat {
  margin-top: 8px;
  color: var(--color-text-muted);
  font-size: 11px;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 10px;
}

.tag-chip {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  height: 24px;
  padding: 0 4px 0 9px;
  color: var(--color-text-secondary);
  font-size: 11.5px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.tag-chip__remove {
  display: grid;
  place-items: center;
  width: 16px;
  height: 16px;
  padding: 0;
  color: var(--color-text-muted);
  background: transparent;
  border: none;
  border-radius: 50%;
}

.tag-chip__remove:hover {
  color: var(--color-danger);
  background: var(--color-surface-hover);
}

.tag-form {
  display: flex;
  gap: 6px;
}

.tag-form__input {
  flex: 1;
  min-width: 0;
  height: 30px;
  padding: 0 9px;
  font-size: 12px;
}

.tag-form__submit {
  height: 30px;
  padding: 0 10px;
  font-size: 12px;
}

.meta-hint {
  color: var(--color-text-muted);
  font-size: 11.5px;
  line-height: 1.7;
}

.related-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.related-item {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 7px 9px;
  text-align: left;
  background: transparent;
  border: none;
  border-radius: 6px;
  transition: background-color var(--transition-fast);
}

.related-item:hover {
  background: var(--color-surface-hover);
}

.related-item__title {
  overflow: hidden;
  color: var(--color-text);
  font-size: 12.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.related-item__type {
  flex: 0 0 auto;
  color: var(--color-text-muted);
  font-size: 10.5px;
}
</style>
