<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import { useNotesStore } from '@/stores/notes'
import { useWorkspaceStore } from '@/stores/workspace'
import { formatDateTime } from '@/utils/format'
import type { RelationSuggestion, Tag } from '@/types'

const notes = useNotesStore()
const workspace = useWorkspaceStore()

const tagInput = ref('')
const tagError = ref('')
const removingTagId = ref<string | null>(null)

const suggestionBusyId = ref<string | null>(null)
const suggestionError = ref('')
const suggestionNotice = ref('')

const hasSelection = computed(() => notes.selectedPoint !== null)

watch(
  () => notes.selectedPointId,
  () => {
    suggestionBusyId.value = null
    suggestionError.value = ''
    suggestionNotice.value = ''
  },
)

const updatedLabel = computed(() =>
  notes.note ? formatDateTime(notes.note.updated_at) : '尚未保存',
)

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function suggestionTitle(suggestion: RelationSuggestion): string {
  return suggestion.source_id === notes.selectedPointId
    ? suggestion.target_title
    : suggestion.source_title
}

function scorePercent(score: number): number {
  return Math.max(1, Math.min(100, Math.round(score * 100)))
}

async function submitTag(): Promise<void> {
  const value = tagInput.value.trim()
  if (!value) return

  tagError.value = ''

  try {
    await notes.addTagToNote(value)
    tagInput.value = ''
  } catch (error) {
    tagError.value = errorText(error)
  }
}

async function removeTag(tag: Tag): Promise<void> {
  if (removingTagId.value) return

  removingTagId.value = tag.id
  tagError.value = ''

  try {
    await notes.removeTagFromNote(tag)
  } catch (error) {
    tagError.value = errorText(error)
  } finally {
    removingTagId.value = null
  }
}

async function refreshSuggestions(): Promise<void> {
  suggestionError.value = ''
  suggestionNotice.value = ''
  await notes.refreshSuggestions()
}

async function acceptSuggestion(suggestion: RelationSuggestion): Promise<void> {
  if (suggestionBusyId.value) return

  suggestionBusyId.value = suggestion.id
  suggestionError.value = ''
  suggestionNotice.value = ''

  try {
    await notes.acceptSuggestion(suggestion.id)
    suggestionNotice.value = `已建立与「${suggestionTitle(suggestion)}」的关联`
  } catch (error) {
    suggestionError.value = errorText(error)
  } finally {
    suggestionBusyId.value = null
  }
}

async function rejectSuggestion(suggestion: RelationSuggestion): Promise<void> {
  if (suggestionBusyId.value) return

  suggestionBusyId.value = suggestion.id
  suggestionError.value = ''
  suggestionNotice.value = ''

  try {
    await notes.rejectSuggestion(suggestion.id)
    suggestionNotice.value = '已忽略该建议'
  } catch (error) {
    suggestionError.value = errorText(error)
  } finally {
    suggestionBusyId.value = null
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
      <p>选择知识点后可以查看所属课程、标签、相关知识点与关联建议。</p>
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

        <RouterLink to="/tags" class="meta-link">
          <AppIcon name="tag" :size="12" />
          管理全部标签
        </RouterLink>
      </section>

      <section class="meta-section">
        <h3>
          相关知识点
          <span class="meta-count">{{ notes.relatedItems.length }}</span>
        </h3>

        <ul v-if="notes.relatedItems.length" class="related-list">
          <li v-for="item in notes.relatedItems" :key="item.relation.id">
            <button
              type="button"
              class="related-item"
              @click="notes.selectPoint(item.point.id)"
            >
              <span
                class="related-item__dot"
                :style="{ background: notes.relationColor(item.relation.relation_type) }"
              />
              <span class="related-item__title">{{ item.point.title }}</span>
              <span class="related-item__type">
                {{ notes.relationLabel(item.relation.relation_type) }}
              </span>
            </button>
          </li>
        </ul>
        <p v-else class="meta-hint">暂无关联知识点，可以在知识关联页手动创建。</p>

        <RouterLink
          :to="{ name: 'relations', query: { point: notes.selectedPointId ?? undefined } }"
          class="meta-link"
        >
          <AppIcon name="link" :size="12" />
          管理知识点关联
        </RouterLink>
      </section>

      <section class="meta-section">
        <h3>
          关联建议
          <button
            type="button"
            class="section-action"
            :disabled="notes.isLoadingSuggestions"
            @click="refreshSuggestions"
          >
            刷新
          </button>
        </h3>

        <div v-if="notes.isLoadingSuggestions" class="suggestion-state">
          <span class="spinner" />
          <span>正在分析文本…</span>
        </div>
        <p v-else-if="notes.suggestionsError" class="form-error">
          {{ notes.suggestionsError }}
        </p>
        <p v-else-if="!notes.suggestions.length" class="meta-hint">
          暂无合适的关联建议。补充知识点描述或笔记内容后，可点击「刷新」重新分析。
        </p>
        <ul v-else class="suggestion-list">
          <li
            v-for="suggestion in notes.suggestions"
            :key="suggestion.id"
            class="meta-suggestion"
          >
            <div class="meta-suggestion__head">
              <span class="meta-suggestion__title" :title="suggestionTitle(suggestion)">
                {{ suggestionTitle(suggestion) }}
              </span>
              <span class="meta-suggestion__score">
                {{ scorePercent(suggestion.score) }}%
              </span>
            </div>
            <p class="meta-suggestion__reason">
              {{ suggestion.reasons.join('；') }}
            </p>
            <div class="meta-suggestion__actions">
              <button
                type="button"
                class="ghost-button meta-suggestion__button"
                :disabled="suggestionBusyId !== null"
                @click="rejectSuggestion(suggestion)"
              >
                忽略
              </button>
              <button
                type="button"
                class="primary-button meta-suggestion__button"
                :disabled="suggestionBusyId !== null"
                @click="acceptSuggestion(suggestion)"
              >
                {{ suggestionBusyId === suggestion.id ? '处理中…' : '接受' }}
              </button>
            </div>
          </li>
        </ul>

        <p v-if="suggestionNotice" class="meta-success">{{ suggestionNotice }}</p>
        <p v-if="suggestionError" class="form-error">{{ suggestionError }}</p>
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
  height: 46px;
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
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 8px;
  color: var(--color-text-muted);
  font-size: 11.5px;
  font-weight: 600;
  letter-spacing: 0.04em;
}

.meta-count {
  padding: 0 6px;
  font-size: 10px;
  font-weight: 500;
  line-height: 16px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.section-action {
  margin-left: auto;
  padding: 0;
  color: var(--color-accent-strong);
  font-size: 10.5px;
  font-weight: 400;
  background: none;
  border: none;
}

.section-action:hover:not(:disabled) {
  text-decoration: underline;
  text-underline-offset: 3px;
}

.section-action:disabled {
  cursor: not-allowed;
  opacity: 0.5;
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

.meta-link {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  margin-top: 9px;
  color: var(--color-accent-strong);
  font-size: 11px;
}

.meta-link:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
}

.meta-success {
  margin-top: 7px;
  color: var(--color-success);
  font-size: 11px;
  line-height: 1.5;
}

.related-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.related-item {
  display: flex;
  gap: 7px;
  align-items: center;
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

.related-item__dot {
  flex: 0 0 auto;
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.related-item__title {
  flex: 1;
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

.suggestion-state {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px 2px;
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.suggestion-list {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.meta-suggestion {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 9px 10px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.meta-suggestion__head {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
}

.meta-suggestion__title {
  overflow: hidden;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.meta-suggestion__score {
  flex: 0 0 auto;
  color: var(--color-accent-strong);
  font-size: 10.5px;
  font-weight: 600;
}

.meta-suggestion__reason {
  color: var(--color-text-muted);
  font-size: 10.5px;
  line-height: 1.55;
}

.meta-suggestion__actions {
  display: flex;
  gap: 5px;
  justify-content: flex-end;
  margin-top: 2px;
}

.meta-suggestion__button {
  height: 25px;
  padding: 0 9px;
  font-size: 10.5px;
}
</style>
