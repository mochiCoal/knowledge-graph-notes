<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import PageHeader from '@/components/common/PageHeader.vue'
import MarkdownImportDialog from '@/components/notes/MarkdownImportDialog.vue'
import { exportNoteToMarkdown } from '@/services/markdownFiles'
import { notesApi } from '@/services/notes'
import { tagsApi } from '@/services/tags'
import type { NoteSummary, TagSummary } from '@/types'
import { formatDateTime } from '@/utils/format'

const route = useRoute()
const router = useRouter()

const notes = ref<NoteSummary[]>([])
const tags = ref<TagSummary[]>([])
const keyword = ref('')
const activeTagId = ref<string | null>(null)
const isLoading = ref(false)
const errorMessage = ref<string | null>(null)
const exportingId = ref<string | null>(null)
const importOpen = ref(false)
const notice = ref<string | null>(null)

let searchTimer: number | null = null
/** 搜索请求序号：避免慢的旧请求覆盖新结果 */
let requestId = 0

function readTagFromQuery(): string | null {
  const value = route.query.tag
  return typeof value === 'string' && value !== '' ? value : null
}

const isSearching = computed(() => keyword.value.trim() !== '')

const activeTag = computed(
  () => tags.value.find((tag) => tag.id === activeTagId.value) ?? null,
)

/** 过滤条只展示有关联笔记的标签，当前筛选的标签始终保留 */
const filterTags = computed(() =>
  tags.value.filter((tag) => tag.note_count > 0 || tag.id === activeTagId.value),
)

const metaText = computed(() => {
  const scope = activeTag.value ? `标签「${activeTag.value.name}」` : ''

  if (isSearching.value) {
    return scope
      ? `在${scope}中找到 ${notes.value.length} 条与「${keyword.value.trim()}」相关的笔记`
      : `找到 ${notes.value.length} 条与「${keyword.value.trim()}」相关的笔记`
  }

  return scope ? `${scope}下共 ${notes.value.length} 条笔记` : `共 ${notes.value.length} 条笔记`
})

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

async function loadTags(): Promise<void> {
  try {
    tags.value = await tagsApi.overview()
  } catch {
    // 标签筛选属于增强能力，加载失败不影响笔记列表
    tags.value = []
  }
}

async function load(searchKeyword = keyword.value): Promise<void> {
  const currentRequest = ++requestId
  isLoading.value = true
  errorMessage.value = null

  try {
    const trimmed = searchKeyword.trim()
    const tagId = activeTagId.value
    const result = trimmed
      ? await notesApi.searchNotes(trimmed, tagId)
      : await notesApi.listNotes(tagId)

    if (currentRequest === requestId) {
      notes.value = result
    }
  } catch (error) {
    if (currentRequest === requestId) {
      errorMessage.value = errorText(error)
    }
  } finally {
    if (currentRequest === requestId) {
      isLoading.value = false
    }
  }
}

function selectTag(tagId: string | null): void {
  if (tagId === activeTagId.value) return

  const query = { ...route.query }
  if (tagId) {
    query.tag = tagId
  } else {
    delete query.tag
  }

  void router.replace({ query })
}

function scheduleSearch(): void {
  notice.value = null

  if (searchTimer !== null) {
    window.clearTimeout(searchTimer)
  }

  searchTimer = window.setTimeout(() => {
    searchTimer = null
    void load()
  }, 300)
}

function clearSearch(): void {
  keyword.value = ''

  if (searchTimer !== null) {
    window.clearTimeout(searchTimer)
    searchTimer = null
  }

  void load('')
}

function openNote(note: NoteSummary): void {
  void router.push({ name: 'notes', query: { point: note.knowledge_point_id } })
}

async function exportNote(note: NoteSummary): Promise<void> {
  if (exportingId.value) return

  exportingId.value = note.id
  errorMessage.value = null

  try {
    const path = await exportNoteToMarkdown(note)
    if (path) {
      notice.value = `已导出「${note.title}」到 ${path}`
    }
  } catch (error) {
    errorMessage.value = errorText(error)
  } finally {
    exportingId.value = null
  }
}

async function onImported(pointId: string, title: string): Promise<void> {
  importOpen.value = false
  notice.value = `已导入「${title}」`

  await load()
  void router.push({ name: 'notes', query: { point: pointId } })
}

onMounted(async () => {
  activeTagId.value = readTagFromQuery()
  await Promise.all([loadTags(), load('')])
})

// 标签筛选写进路由，便于从标签管理页直接带标签进入
watch(
  () => route.query.tag,
  () => {
    const next = readTagFromQuery()
    if (next === activeTagId.value) return

    activeTagId.value = next
    void load()
  },
)

onBeforeUnmount(() => {
  if (searchTimer !== null) window.clearTimeout(searchTimer)
})
</script>

<template>
  <div class="page">
    <PageHeader
      title="笔记库"
      description="集中检索所有知识点下的 Markdown 笔记，并支持从本地导入文件、导出为 .md。"
    >
      <template #actions>
        <button type="button" class="primary-button" @click="importOpen = true">
          <AppIcon name="upload" :size="16" />
          导入 Markdown
        </button>
      </template>
    </PageHeader>

    <div v-if="errorMessage" class="alert-bar" role="alert">
      <AppIcon name="alert" :size="16" />
      <span>{{ errorMessage }}</span>
      <button type="button" class="alert-bar__dismiss" @click="errorMessage = null">关闭</button>
    </div>

    <div v-if="notice" class="notice-bar" role="status">
      <AppIcon name="sparkles" :size="15" />
      <span>{{ notice }}</span>
      <button type="button" class="alert-bar__dismiss" @click="notice = null">关闭</button>
    </div>

    <section class="library-bar">
      <div class="search-field">
        <AppIcon name="search" :size="16" class="search-field__icon" />
        <input
          v-model="keyword"
          class="search-field__input"
          type="text"
          placeholder="搜索笔记标题或正文…"
          aria-label="搜索笔记"
          @input="scheduleSearch"
        />
        <button
          v-if="keyword"
          type="button"
          class="search-field__clear"
          aria-label="清空搜索"
          @click="clearSearch"
        >
          <AppIcon name="x" :size="14" />
        </button>
      </div>

      <span class="library-bar__meta">{{ metaText }}</span>
    </section>

    <div v-if="tags.length" class="tag-filters" aria-label="按标签筛选">
      <button
        type="button"
        class="filter-chip"
        :class="{ 'is-active': activeTagId === null }"
        @click="selectTag(null)"
      >
        全部
      </button>
      <button
        v-for="tag in filterTags"
        :key="tag.id"
        type="button"
        class="filter-chip"
        :class="{ 'is-active': activeTagId === tag.id }"
        @click="selectTag(tag.id)"
      >
        #{{ tag.name }}
        <span class="filter-chip__count">{{ tag.note_count }}</span>
      </button>
      <RouterLink to="/tags" class="filter-chip filter-chip--link" title="管理标签">
        <AppIcon name="tag" :size="13" />
        管理标签
      </RouterLink>
    </div>

    <div v-if="isLoading && !notes.length" class="library-state">
      <span class="spinner" />
      <span>正在加载笔记…</span>
    </div>

    <div v-else-if="errorMessage && !notes.length" class="empty-panel library-empty">
      <div class="empty-panel__icon">
        <AppIcon name="alert" :size="20" />
      </div>
      <h3>笔记加载失败</h3>
      <p>{{ errorMessage }}</p>
      <button type="button" class="primary-button" @click="load()">重新加载</button>
    </div>

    <div v-else-if="!notes.length" class="empty-panel library-empty">
      <div class="empty-panel__icon">
        <AppIcon :name="isSearching ? 'search' : 'notes'" :size="22" />
      </div>
      <h3>
        {{
          isSearching
            ? '没有找到匹配的笔记'
            : activeTag
              ? `标签「${activeTag.name}」下没有笔记`
              : '还没有笔记'
        }}
      </h3>
      <p>
        {{
          isSearching
            ? `没有笔记的标题或正文包含「${keyword.trim()}」，换个关键字试试。`
            : activeTag
              ? '给笔记添加这个标签后，它就会出现在这里。'
              : '在笔记工作区选择一个知识点开始记录，或从本地导入 Markdown 文件。'
        }}
      </p>
      <button
        v-if="isSearching"
        type="button"
        class="ghost-button"
        @click="clearSearch"
      >
        清空搜索条件
      </button>
      <button
        v-else-if="activeTag"
        type="button"
        class="ghost-button"
        @click="selectTag(null)"
      >
        查看全部笔记
      </button>
      <RouterLink v-else to="/notes" class="primary-button">
        <AppIcon name="notes" :size="16" />
        前往笔记工作区
      </RouterLink>
    </div>

    <ul v-else class="note-list" aria-label="笔记列表">
      <li v-for="note in notes" :key="note.id" class="note-item">
        <button type="button" class="note-item__main" @click="openNote(note)">
          <span class="note-item__title">{{ note.title }}</span>
          <span class="note-item__excerpt">{{ note.excerpt || '（暂无正文）' }}</span>
          <span v-if="note.tag_names.length" class="note-item__tags">
            <span v-for="name in note.tag_names" :key="name" class="note-tag">
              #{{ name }}
            </span>
          </span>
          <span class="note-item__meta">
            <span class="note-item__path">
              {{ note.course_name }} / {{ note.knowledge_point_title }}
            </span>
            <span class="note-item__dot" aria-hidden="true">·</span>
            <span>{{ note.character_count }} 字</span>
            <span class="note-item__dot" aria-hidden="true">·</span>
            <span>更新于 {{ formatDateTime(note.updated_at) }}</span>
          </span>
        </button>

        <div class="note-item__actions">
          <button
            type="button"
            class="action-button"
            :disabled="exportingId !== null"
            title="导出为 Markdown"
            @click="exportNote(note)"
          >
            <AppIcon name="download" :size="15" />
            {{ exportingId === note.id ? '导出中…' : '导出' }}
          </button>
          <button
            type="button"
            class="action-button action-button--primary"
            title="在笔记工作区打开"
            @click="openNote(note)"
          >
            打开
            <AppIcon name="arrow-right" :size="14" />
          </button>
        </div>
      </li>
    </ul>

    <MarkdownImportDialog
      :open="importOpen"
      @close="importOpen = false"
      @imported="onImported"
    />
  </div>
</template>

<style scoped>
.library-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 16px;
  align-items: center;
  justify-content: space-between;
}

.search-field {
  position: relative;
  display: flex;
  flex: 1 1 320px;
  gap: 8px;
  align-items: center;
  max-width: 560px;
  height: 38px;
  padding: 0 10px 0 12px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  transition:
    background-color var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.search-field:focus-within {
  background: var(--color-surface);
  border-color: var(--color-accent);
  box-shadow: 0 0 0 2px var(--color-accent-soft);
}

.search-field__icon {
  flex: 0 0 auto;
  color: var(--color-text-muted);
}

.search-field__input {
  flex: 1;
  min-width: 0;
  height: 100%;
  padding: 0;
  font-size: 13px;
  background: transparent;
  border: none;
  outline: none;
}

.search-field__input::placeholder {
  color: var(--color-text-muted);
}

.search-field__clear {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 22px;
  height: 22px;
  padding: 0;
  color: var(--color-text-muted);
  background: transparent;
  border: none;
  border-radius: 4px;
}

.search-field__clear:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.library-bar__meta {
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.tag-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.filter-chip {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  height: 26px;
  padding: 0 10px;
  color: var(--color-text-secondary);
  font-size: 11.5px;
  white-space: nowrap;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 999px;
  transition:
    color var(--transition-fast),
    border-color var(--transition-fast),
    background-color var(--transition-fast);
}

.filter-chip:hover {
  color: var(--color-accent);
  border-color: var(--color-accent);
}

.filter-chip.is-active {
  color: var(--color-accent-strong);
  font-weight: 500;
  background: var(--color-accent-soft);
  border-color: var(--color-accent);
}

.filter-chip__count {
  color: var(--color-text-muted);
  font-size: 10px;
  font-weight: 400;
}

.filter-chip.is-active .filter-chip__count {
  color: var(--color-accent-strong);
}

.filter-chip--link {
  margin-left: auto;
  color: var(--color-text-muted);
  border-style: dashed;
}

.note-item__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.note-tag {
  padding: 0 7px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  line-height: 18px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.note-list {
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--color-border);
}

.note-item {
  display: flex;
  gap: 20px;
  align-items: center;
  padding: 18px 8px;
  border-bottom: 1px solid var(--color-border);
  transition: background-color var(--transition-fast);
}

.note-item:hover,
.note-item:focus-within {
  background: var(--color-surface-hover);
}

.note-item__main {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  padding: 0;
  text-align: left;
  background: transparent;
  border: none;
}

.note-item__title {
  overflow: hidden;
  color: var(--color-text);
  font-size: 14px;
  font-weight: 600;
  line-height: 1.4;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.note-item__main:hover .note-item__title {
  color: var(--color-accent-strong);
}

.note-item__excerpt {
  display: -webkit-box;
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 12.5px;
  line-height: 1.6;
  overflow-wrap: anywhere;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.note-item__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  align-items: baseline;
  color: var(--color-text-muted);
  font-size: 11px;
}

.note-item__path {
  overflow: hidden;
  max-width: 100%;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.note-item__dot {
  color: var(--color-border-strong);
}

.note-item__actions {
  display: flex;
  flex: 0 0 auto;
  gap: 6px;
  align-items: center;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.note-item:hover .note-item__actions,
.note-item:focus-within .note-item__actions {
  opacity: 1;
}

.action-button {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  height: 30px;
  padding: 0 10px;
  color: var(--color-text-secondary);
  font-size: 12px;
  white-space: nowrap;
  background: var(--color-surface);
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  transition:
    color var(--transition-fast),
    border-color var(--transition-fast),
    background-color var(--transition-fast);
}

.action-button:hover:not(:disabled) {
  color: var(--color-accent);
  background: var(--color-accent-soft);
  border-color: var(--color-accent);
}

.action-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.action-button--primary {
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
  border-color: transparent;
}

.library-state {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: center;
  padding: 56px 0;
  color: var(--color-text-muted);
  font-size: 13px;
}

.library-empty {
  gap: 8px;
  padding: 48px 18px;
  background: var(--color-surface);
  border: 1px dashed var(--color-border-strong);
  border-radius: var(--radius-lg);
}

.library-empty .primary-button,
.library-empty .ghost-button {
  margin-top: 6px;
}

.alert-bar,
.notice-bar {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 10px 14px;
  font-size: 12.5px;
  border-radius: var(--radius-sm);
}

.alert-bar {
  color: var(--color-danger);
  background: var(--color-danger-soft);
  border: 1px solid var(--color-danger);
}

.notice-bar {
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
  border: 1px solid var(--color-accent);
}

.alert-bar__dismiss {
  margin-left: auto;
  padding: 0;
  color: inherit;
  font-size: 12px;
  background: none;
  border: none;
  text-decoration: underline;
  text-underline-offset: 3px;
}

@media (hover: none) {
  .note-item__actions {
    opacity: 1;
  }
}

@media (max-width: 760px) {
  .note-item {
    flex-direction: column;
    align-items: stretch;
  }

  .note-item__actions {
    justify-content: flex-end;
    opacity: 1;
  }

  .search-field {
    max-width: none;
  }
}
</style>
