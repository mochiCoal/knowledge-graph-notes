<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import PageHeader from '@/components/common/PageHeader.vue'
import TagFormDialog from '@/components/tags/TagFormDialog.vue'
import { useTagsStore } from '@/stores/tags'
import type { NoteSummary, Tag, TagSummary } from '@/types'
import { formatDateTime } from '@/utils/format'

const router = useRouter()
const store = useTagsStore()

const keyword = ref('')
const formOpen = ref(false)
const editingTag = ref<TagSummary | null>(null)
const deleteOpen = ref(false)
const deleting = ref(false)
const actionError = ref<string | null>(null)
const notice = ref<string | null>(null)

let searchTimer: number | null = null

const deleteMessage = computed(() => {
  const tag = store.selectedTag
  if (!tag) return ''

  return tag.note_count > 0
    ? `确定删除标签「${tag.name}」吗？${tag.note_count} 条笔记会解除与该标签的关联，笔记本身不会被删除。`
    : `确定删除标签「${tag.name}」吗？该标签还没有关联笔记。`
})

const showInitialEmpty = computed(
  () => store.initialized && store.tags.length === 0 && keyword.value.trim() === '',
)

onMounted(() => {
  keyword.value = store.keyword

  // 标签可能在笔记工作区被增删，重新进入时刷新一次
  if (store.initialized) {
    void store.refresh()
  } else {
    void store.initialize()
  }
})

onBeforeUnmount(() => {
  if (searchTimer !== null) window.clearTimeout(searchTimer)
})

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function onSearchInput(): void {
  notice.value = null

  if (searchTimer !== null) window.clearTimeout(searchTimer)

  searchTimer = window.setTimeout(() => {
    searchTimer = null
    void store.search(keyword.value)
  }, 250)
}

function clearSearch(): void {
  keyword.value = ''

  if (searchTimer !== null) {
    window.clearTimeout(searchTimer)
    searchTimer = null
  }

  void store.search('')
}

function selectTag(tagId: string): void {
  actionError.value = null
  notice.value = null
  void store.selectTag(tagId)
}

function refresh(): void {
  actionError.value = null
  notice.value = null
  void store.refresh()
}

function openCreate(): void {
  actionError.value = null
  notice.value = null
  editingTag.value = null
  formOpen.value = true
}

function openRename(): void {
  const tag = store.selectedTag
  if (!tag) return

  actionError.value = null
  notice.value = null
  editingTag.value = tag
  formOpen.value = true
}

function onSaved(tag: Tag): void {
  formOpen.value = false
  actionError.value = null
  notice.value = `已保存标签「${tag.name}」`
}

function requestDelete(): void {
  if (!store.selectedTag) return

  actionError.value = null
  notice.value = null
  deleteOpen.value = true
}

function cancelDelete(): void {
  if (deleting.value) return
  deleteOpen.value = false
}

async function confirmDelete(): Promise<void> {
  const tag = store.selectedTag
  if (!tag || deleting.value) return

  deleting.value = true
  actionError.value = null

  try {
    const detached = await store.deleteTag(tag.id)
    deleteOpen.value = false
    notice.value =
      detached > 0
        ? `已删除标签「${tag.name}」，${detached} 条笔记已解除关联`
        : `已删除标签「${tag.name}」`
  } catch (error) {
    actionError.value = errorText(error)
    deleteOpen.value = false
  } finally {
    deleting.value = false
  }
}

function openNote(note: NoteSummary): void {
  void router.push({ name: 'notes', query: { point: note.knowledge_point_id } })
}

function openInLibrary(): void {
  const tag = store.selectedTag
  if (!tag) return

  void router.push({ name: 'note-library', query: { tag: tag.id } })
}
</script>

<template>
  <div class="page">
    <PageHeader
      title="标签管理"
      description="创建、重命名与删除标签，并查看每个标签下的笔记，用标签对笔记进行分类和检索。"
    >
      <template #actions>
        <button
          type="button"
          class="ghost-button"
          :disabled="store.isLoading"
          @click="refresh"
        >
          <AppIcon name="refresh" :size="15" />
          刷新
        </button>
        <button type="button" class="primary-button" @click="openCreate">
          <AppIcon name="plus" :size="16" />
          新建标签
        </button>
      </template>
    </PageHeader>

    <div
      v-if="actionError || (store.errorMessage && store.tags.length)"
      class="alert-bar"
      role="alert"
    >
      <AppIcon name="alert" :size="16" />
      <span>{{ actionError || store.errorMessage }}</span>
      <button
        type="button"
        class="alert-bar__dismiss"
        @click="actionError = null; store.dismissError()"
      >
        关闭
      </button>
    </div>

    <div v-if="notice" class="notice-bar" role="status">
      <AppIcon name="sparkles" :size="15" />
      <span>{{ notice }}</span>
      <button type="button" class="alert-bar__dismiss" @click="notice = null">关闭</button>
    </div>

    <div v-if="store.isLoading && !store.initialized" class="page-state">
      <span class="spinner" />
      <span>正在加载标签…</span>
    </div>

    <div v-else-if="store.errorMessage && !store.tags.length" class="empty-panel tags-empty">
      <div class="empty-panel__icon">
        <AppIcon name="alert" :size="20" />
      </div>
      <h3>标签加载失败</h3>
      <p>{{ store.errorMessage }}</p>
      <button type="button" class="primary-button" @click="refresh">重新加载</button>
    </div>

    <div v-else-if="showInitialEmpty" class="empty-panel tags-empty">
      <div class="empty-panel__icon">
        <AppIcon name="tag" :size="22" />
      </div>
      <h3>还没有标签</h3>
      <p>创建标签后，就能在笔记里给内容分类，并按标签快速检索笔记。</p>
      <button type="button" class="primary-button" @click="openCreate">
        <AppIcon name="plus" :size="16" />
        新建标签
      </button>
    </div>

    <div v-else class="tags-layout">
      <aside class="pane pane--tags" aria-label="标签列表">
        <header class="pane__head">
          <h2>全部标签</h2>
          <span class="count-badge">{{ store.tags.length }}</span>
        </header>

        <div class="pane__body pane__body--tags">
          <div class="search-field">
            <AppIcon name="search" :size="15" class="search-field__icon" />
            <input
              v-model="keyword"
              class="search-field__input"
              type="text"
              placeholder="搜索标签…"
              aria-label="搜索标签"
              @input="onSearchInput"
            />
            <button
              v-if="keyword"
              type="button"
              class="search-field__clear"
              aria-label="清空搜索"
              @click="clearSearch"
            >
              <AppIcon name="x" :size="13" />
            </button>
          </div>

          <div v-if="store.isLoading" class="pane-state">
            <span class="spinner" />
            <span>正在搜索…</span>
          </div>

          <ul v-else-if="store.tags.length" class="tag-list">
            <li v-for="tag in store.tags" :key="tag.id">
              <button
                type="button"
                class="tag-option"
                :class="{ 'is-active': tag.id === store.selectedTagId }"
                @click="selectTag(tag.id)"
              >
                <span class="tag-option__name">#{{ tag.name }}</span>
                <span class="tag-option__count">{{ tag.note_count }}</span>
              </button>
            </li>
          </ul>

          <p v-else class="pane-hint">没有找到匹配的标签。</p>
        </div>

        <footer class="pane__foot">
          {{ store.tags.length }} 个标签 · {{ store.totalTaggedNotes }} 条笔记关联
        </footer>
      </aside>

      <section class="pane pane--notes" aria-label="标签下的笔记">
        <header class="pane__head">
          <h2 :title="store.selectedTag?.name">
            {{ store.selectedTag ? `#${store.selectedTag.name}` : '标签笔记' }}
          </h2>
          <span class="count-badge">{{ store.selectedTag?.note_count ?? 0 }}</span>

          <div v-if="store.selectedTag" class="pane__head-actions">
            <button
              type="button"
              class="ghost-button pane__head-button"
              @click="openRename"
            >
              <AppIcon name="pencil" :size="14" />
              重命名
            </button>
            <button
              type="button"
              class="ghost-button pane__head-button pane__head-button--danger"
              @click="requestDelete"
            >
              <AppIcon name="trash" :size="14" />
              删除
            </button>
          </div>
        </header>

        <div class="pane__body pane__body--notes">
          <div v-if="store.isLoadingNotes" class="pane-state">
            <span class="spinner" />
            <span>正在加载笔记…</span>
          </div>

          <div v-else-if="store.notesError" class="pane-state pane-state--error">
            <AppIcon name="alert" :size="16" />
            <span>{{ store.notesError }}</span>
            <button
              type="button"
              class="ghost-button"
              @click="store.selectedTagId && store.selectTag(store.selectedTagId)"
            >
              重试
            </button>
          </div>

          <div v-else-if="!store.selectedTag" class="empty-panel">
            <div class="empty-panel__icon">
              <AppIcon name="tag" :size="20" />
            </div>
            <h3>选择一个标签</h3>
            <p>在左侧选择标签，查看使用该标签的笔记。</p>
          </div>

          <template v-else>
            <ul v-if="store.notes.length" class="tag-note-list">
              <li v-for="note in store.notes" :key="note.id" class="tag-note">
                <button type="button" class="tag-note__main" @click="openNote(note)">
                  <span class="tag-note__title">{{ note.title }}</span>
                  <span class="tag-note__path">
                    {{ note.course_name }} / {{ note.knowledge_point_title }}
                  </span>
                  <span v-if="note.tag_names.length" class="tag-note__tags">
                    <span v-for="name in note.tag_names" :key="name" class="tag-chip">
                      #{{ name }}
                    </span>
                  </span>
                  <span class="tag-note__meta">
                    {{ note.character_count }} 字 · 更新于 {{ formatDateTime(note.updated_at) }}
                  </span>
                </button>

                <button
                  type="button"
                  class="ghost-button tag-note__open"
                  @click="openNote(note)"
                >
                  打开
                  <AppIcon name="arrow-right" :size="13" />
                </button>
              </li>
            </ul>

            <div v-else class="empty-panel">
              <div class="empty-panel__icon">
                <AppIcon name="notes" :size="20" />
              </div>
              <h3>该标签下还没有笔记</h3>
              <p>在笔记工作区编辑笔记时，可以在右侧面板为它添加这个标签。</p>
              <RouterLink to="/notes" class="ghost-button">前往笔记工作区</RouterLink>
            </div>
          </template>
        </div>

        <footer v-if="store.selectedTag" class="pane__foot pane__foot--split">
          <span>共 {{ store.notes.length }} 条笔记</span>
          <button type="button" class="text-button" @click="openInLibrary">
            在笔记库中筛选
            <AppIcon name="arrow-right" :size="13" />
          </button>
        </footer>
      </section>
    </div>

    <TagFormDialog
      :open="formOpen"
      :tag="editingTag"
      @close="formOpen = false"
      @saved="onSaved"
    />

    <ConfirmDialog
      :open="deleteOpen"
      title="删除标签"
      :message="deleteMessage"
      confirm-text="删除标签"
      :busy="deleting"
      @cancel="cancelDelete"
      @confirm="confirmDelete"
    />
  </div>
</template>

<style scoped>
.tags-layout {
  display: grid;
  grid-template-columns: 260px minmax(0, 1fr);
  gap: 14px;
  align-items: start;
}

.pane {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
}

.pane__head {
  display: flex;
  flex: 0 0 auto;
  gap: 7px;
  align-items: center;
  min-height: 44px;
  padding: 0 10px 0 16px;
  border-bottom: 1px solid var(--color-border);
}

.pane__head h2 {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.01em;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.count-badge {
  flex: 0 0 auto;
  padding: 0 7px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  font-weight: 500;
  line-height: 18px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.pane__head-actions {
  display: flex;
  flex: 0 0 auto;
  gap: 4px;
  margin-left: auto;
}

.pane__head-button {
  height: 28px;
  padding: 0 9px;
  font-size: 11.5px;
}

.pane__head-button--danger:hover {
  color: var(--color-danger);
  background: var(--color-danger-soft);
  border-color: var(--color-danger);
}

.pane__body {
  padding: 8px;
  overflow-y: auto;
}

.pane__body--tags {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: min(62vh, 640px);
}

.pane__body--notes {
  min-height: 220px;
  max-height: min(64vh, 680px);
  padding: 10px;
}

.pane__foot {
  flex: 0 0 auto;
  padding: 9px 16px;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 10.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
  border-top: 1px solid var(--color-border);
}

.pane__foot--split {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: space-between;
}

.search-field {
  display: flex;
  flex: 0 0 auto;
  gap: 7px;
  align-items: center;
  height: 32px;
  padding: 0 8px 0 10px;
  background: var(--color-surface);
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.search-field:focus-within {
  border-color: var(--color-accent);
  box-shadow: 0 0 0 3px var(--color-accent-soft);
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
  font-size: 12.5px;
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
  width: 20px;
  height: 20px;
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

.tag-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.tag-option {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
  padding: 8px 10px;
  color: var(--color-text-secondary);
  font-size: 12.5px;
  text-align: left;
  background: transparent;
  border: none;
  border-radius: 7px;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.tag-option:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.tag-option.is-active {
  color: var(--color-accent-strong);
  font-weight: 500;
  background: var(--color-accent-soft);
}

.tag-option__name {
  flex: 1;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.tag-option__count {
  flex: 0 0 auto;
  color: var(--color-text-muted);
  font-size: 10.5px;
  font-weight: 400;
}

.pane-state {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
  padding: 36px 12px;
  color: var(--color-text-muted);
  font-size: 12px;
  text-align: center;
}

.pane-state--error {
  color: var(--color-danger);
}

.pane-state .ghost-button {
  height: 28px;
  padding: 0 12px;
  font-size: 12px;
}

.tag-note-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.tag-note {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 11px 12px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.tag-note:hover {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-sm);
}

.tag-note__main {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
  padding: 0;
  text-align: left;
  background: transparent;
  border: none;
}

.tag-note__title {
  overflow: hidden;
  color: var(--color-text);
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.tag-note__main:hover .tag-note__title {
  color: var(--color-accent-strong);
}

.tag-note__path,
.tag-note__meta {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 11px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.tag-note__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin: 1px 0;
}

.tag-chip {
  padding: 0 7px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  line-height: 18px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.tag-note__open {
  flex: 0 0 auto;
  height: 28px;
  padding: 0 9px;
  font-size: 11.5px;
}

.text-button {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  padding: 0;
  color: var(--color-accent-strong);
  font-size: 10.5px;
  background: none;
  border: none;
}

.text-button:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
}

.tags-empty {
  gap: 8px;
  padding: 56px 18px;
  background: var(--color-surface);
  border: 1px dashed var(--color-border-strong);
  border-radius: var(--radius-lg);
}

.tags-empty .primary-button {
  margin-top: 6px;
}

.page-state {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  color: var(--color-text-muted);
  font-size: 13px;
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

@media (max-width: 900px) {
  .tags-layout {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
