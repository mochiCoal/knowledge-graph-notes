<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import MarkdownImportDialog from './MarkdownImportDialog.vue'
import SaveStatusIndicator from './SaveStatusIndicator.vue'
import type { IconName } from '@/components/common/icons'
import { exportNoteToMarkdown } from '@/services/markdownFiles'
import { useNotesStore } from '@/stores/notes'
import { useWorkspaceStore } from '@/stores/workspace'
import { renderMarkdown } from '@/utils/markdown'
import type { EditorMode } from '@/types'

const notes = useNotesStore()
const workspace = useWorkspaceStore()

const textareaRef = ref<HTMLTextAreaElement | null>(null)
const previewRef = ref<HTMLElement | null>(null)
const confirmDeleteOpen = ref(false)
const deleting = ref(false)
const deleteError = ref('')

const importOpen = ref(false)
const exporting = ref(false)

interface FileNotice {
  type: 'success' | 'error'
  text: string
}

const fileNotice = ref<FileNotice | null>(null)
let noticeTimer: number | null = null

const modes: { value: EditorMode; label: string; icon: IconName }[] = [
  { value: 'edit', label: '编辑', icon: 'pencil' },
  { value: 'split', label: '分屏', icon: 'split' },
  { value: 'preview', label: '预览', icon: 'eye' },
]

const renderedContent = computed(() => renderMarkdown(notes.draft.content))
const hasSelection = computed(() => notes.selectedPoint !== null)
const courseName = computed(() => notes.selectedCourse?.name ?? '')
const pointPath = computed(() => notes.selectedPointPath)
const pathTitle = computed(() =>
  [courseName.value, ...pointPath.value.map((point) => point.title)]
    .filter(Boolean)
    .join(' / '),
)

function onTitleInput(event: Event): void {
  notes.updateTitle((event.target as HTMLInputElement).value)
}

function onContentInput(event: Event): void {
  notes.updateContent((event.target as HTMLTextAreaElement).value)
}

function onEditorKeydown(event: KeyboardEvent): void {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault()
    void notes.saveNow()
  }
}

// 分屏模式下让预览跟随编辑器滚动（按比例同步，足够直观且不会互相抢占）
let scrollFrame = 0

function onEditorScroll(): void {
  if (workspace.editorMode !== 'split' || scrollFrame) return

  scrollFrame = window.requestAnimationFrame(() => {
    scrollFrame = 0

    const editor = textareaRef.value
    const preview = previewRef.value
    if (!editor || !preview) return

    const editorMax = editor.scrollHeight - editor.clientHeight
    const previewMax = preview.scrollHeight - preview.clientHeight
    if (editorMax <= 0 || previewMax <= 0) return

    preview.scrollTop = (editor.scrollTop / editorMax) * previewMax
  })
}

watch(
  () => notes.selectedPointId,
  () => {
    resetPaneScroll()
  },
)

// 笔记加载完成后回到顶部，避免复用的 textarea 保留上一条笔记的滚动位置
watch(
  () => notes.isLoadingNote,
  (loading) => {
    if (!loading) resetPaneScroll()
  },
)

function resetPaneScroll(): void {
  void nextTick(() => {
    // 等一帧，确保新的 Markdown 内容已经完成布局
    window.requestAnimationFrame(() => {
      const textarea = textareaRef.value
      if (textarea) {
        if (document.activeElement === textarea) {
          textarea.blur()
        }
        textarea.scrollTop = 0
        textarea.setSelectionRange(0, 0)
      }
      if (previewRef.value) previewRef.value.scrollTop = 0
    })
  })
}

function notify(type: FileNotice['type'], text: string): void {
  fileNotice.value = { type, text }

  if (noticeTimer !== null) {
    window.clearTimeout(noticeTimer)
    noticeTimer = null
  }

  if (type === 'success') {
    noticeTimer = window.setTimeout(() => {
      fileNotice.value = null
      noticeTimer = null
    }, 5000)
  }
}

function openImport(): void {
  if (!hasSelection.value) return

  fileNotice.value = null
  // 先把当前草稿落盘，避免导入覆盖后丢失未保存的编辑
  void notes.flushSave()
  importOpen.value = true
}

async function exportNote(): Promise<void> {
  if (exporting.value) return
  if (!notes.note && !notes.draft.content.trim()) return

  exporting.value = true

  try {
    const saved = await notes.flushSave()
    if (!saved) {
      notify('error', '当前笔记保存失败，请先重试保存再导出。')
      return
    }

    const current = notes.note
    if (!current) {
      notify('error', '笔记尚未保存，无法导出。')
      return
    }

    const path = await exportNoteToMarkdown(current)
    if (path) {
      notify('success', `已导出到 ${path}`)
    }
  } catch (error) {
    notify('error', error instanceof Error ? error.message : String(error))
  } finally {
    exporting.value = false
  }
}

async function onImported(pointId: string, title: string): Promise<void> {
  importOpen.value = false

  try {
    // 目标知识点可能是在其它页面新建的，先刷新目录再切换
    if (!notes.knowledgePoints.some((point) => point.id === pointId)) {
      await notes.refreshCatalog()
    }

    if (pointId !== notes.selectedPointId) {
      await notes.selectPoint(pointId)
    } else {
      await notes.reloadSelectedNote()
    }

    notify('success', `已导入「${title}」`)
  } catch (error) {
    notify('error', error instanceof Error ? error.message : String(error))
  }
}

onBeforeUnmount(() => {
  if (scrollFrame) window.cancelAnimationFrame(scrollFrame)
  if (noticeTimer !== null) window.clearTimeout(noticeTimer)
})

async function confirmDelete(): Promise<void> {
  if (deleting.value) return

  deleting.value = true
  deleteError.value = ''

  try {
    await notes.deleteCurrentNote()
    confirmDeleteOpen.value = false
  } catch (error) {
    deleteError.value = error instanceof Error ? error.message : String(error)
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <section class="note-workspace" aria-label="笔记编辑区">
    <header class="note-toolbar">
      <button
        type="button"
        class="icon-button"
        :class="{ 'is-active': !workspace.sidebarCollapsed }"
        :title="workspace.sidebarCollapsed ? '展开目录' : '收起目录'"
        :aria-label="workspace.sidebarCollapsed ? '展开目录' : '收起目录'"
        @click="workspace.toggleSidebar()"
      >
        <AppIcon name="panel-left" :size="17" />
      </button>

      <div class="note-toolbar__right">
        <SaveStatusIndicator
          v-if="hasSelection"
          :status="notes.saveStatus"
          :saved-at="notes.lastSavedAt"
          @retry="notes.saveNow()"
        />

        <div class="mode-switch" role="group" aria-label="视图模式">
          <button
            v-for="mode in modes"
            :key="mode.value"
            type="button"
            class="mode-switch__option"
            :class="{ 'is-active': workspace.editorMode === mode.value }"
            :aria-pressed="workspace.editorMode === mode.value"
            :title="mode.label"
            @click="workspace.setEditorMode(mode.value)"
          >
            <AppIcon :name="mode.icon" :size="14" />
            <span class="mode-switch__label">{{ mode.label }}</span>
          </button>
        </div>

        <button
          type="button"
          class="icon-button"
          title="导入 Markdown"
          aria-label="导入 Markdown"
          :disabled="!hasSelection"
          @click="openImport"
        >
          <AppIcon name="upload" :size="16" />
        </button>

        <button
          type="button"
          class="icon-button"
          title="导出 Markdown"
          aria-label="导出 Markdown"
          :disabled="(!notes.note && !notes.draft.content.trim()) || exporting"
          @click="exportNote"
        >
          <AppIcon name="download" :size="16" />
        </button>

        <button
          v-if="notes.note"
          type="button"
          class="icon-button"
          title="删除笔记"
          aria-label="删除笔记"
          @click="confirmDeleteOpen = true"
        >
          <AppIcon name="trash" :size="16" />
        </button>

        <button
          type="button"
          class="icon-button"
          :class="{ 'is-active': workspace.metaOpen }"
          :title="workspace.metaOpen ? '隐藏辅助面板' : '显示辅助面板'"
          :aria-label="workspace.metaOpen ? '隐藏辅助面板' : '显示辅助面板'"
          @click="workspace.toggleMeta()"
        >
          <AppIcon name="panel-right" :size="16" />
        </button>
      </div>
    </header>

    <div v-if="notes.errorMessage" class="workspace-banner" role="alert">
      <AppIcon name="alert" :size="14" />
      <span class="workspace-banner__text">{{ notes.errorMessage }}</span>
      <button type="button" class="workspace-banner__close" @click="notes.dismissError()">
        关闭
      </button>
    </div>

    <div
      v-if="fileNotice"
      class="workspace-banner"
      :class="{ 'workspace-banner--success': fileNotice.type === 'success' }"
      :role="fileNotice.type === 'error' ? 'alert' : 'status'"
    >
      <AppIcon :name="fileNotice.type === 'success' ? 'sparkles' : 'alert'" :size="14" />
      <span class="workspace-banner__text" :title="fileNotice.text">{{ fileNotice.text }}</span>
      <button type="button" class="workspace-banner__close" @click="fileNotice = null">
        关闭
      </button>
    </div>

    <div v-if="!hasSelection" class="workspace-empty">
      <div class="workspace-empty__icon">
        <AppIcon name="notes" :size="26" />
      </div>
      <h2>选择知识点开始记录</h2>
      <p>在左侧展开课程，选择一个知识点，即可编辑对应的 Markdown 笔记。</p>
      <button
        v-if="workspace.sidebarCollapsed"
        type="button"
        class="ghost-button"
        @click="workspace.toggleSidebar()"
      >
        展开目录
      </button>
    </div>

    <template v-else>
      <div class="note-heading">
        <div class="note-path" :title="pathTitle">
          <span class="note-path__course">{{ courseName }}</span>
          <template v-for="(point, index) in pointPath" :key="point.id">
            <AppIcon name="chevron-right" :size="11" class="note-path__sep" />
            <span
              class="note-path__node"
              :class="{ 'is-current': index === pointPath.length - 1 }"
            >
              {{ point.title }}
            </span>
          </template>
        </div>

        <div class="note-title-row">
          <input
            class="note-title"
            type="text"
            :value="notes.draft.title"
            maxlength="200"
            placeholder="笔记标题"
            aria-label="笔记标题"
            @input="onTitleInput"
          />
          <span v-if="notes.isLoadingNote" class="note-title__loading">
            <span class="spinner" />
          </span>
        </div>
      </div>

      <div class="note-panes" :class="`note-panes--${workspace.editorMode}`">
        <div v-show="workspace.editorMode !== 'preview'" class="note-pane note-pane--editor">
          <div class="note-pane__inner">
            <textarea
              ref="textareaRef"
              class="note-textarea"
              :value="notes.draft.content"
              spellcheck="false"
              aria-label="Markdown 编辑区"
              placeholder="开始记录…&#10;&#10;支持 Markdown 标题、列表、引用、代码块与表格。"
              @input="onContentInput"
              @keydown="onEditorKeydown"
              @scroll="onEditorScroll"
            />
          </div>
        </div>

        <div
          v-show="workspace.editorMode !== 'edit'"
          ref="previewRef"
          class="note-pane note-pane--preview"
        >
          <div class="note-pane__inner">
            <article
              v-if="notes.draft.content.trim()"
              class="markdown-body preview-body"
              v-html="renderedContent"
            />
            <p v-else class="preview-placeholder">预览会显示在这里</p>
          </div>
        </div>
      </div>
    </template>

    <MarkdownImportDialog
      :open="importOpen"
      :default-point-id="notes.selectedPointId"
      @close="importOpen = false"
      @imported="onImported"
    />

    <ConfirmDialog
      :open="confirmDeleteOpen"
      title="删除笔记"
      :message="
        deleteError
          ? `删除失败：${deleteError}`
          : '删除后无法恢复，确定要删除这条笔记吗？'
      "
      :busy="deleting"
      confirm-text="删除笔记"
      @cancel="confirmDeleteOpen = false"
      @confirm="confirmDelete"
    />
  </section>
</template>

<style scoped>
.note-workspace {
  position: relative;
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-width: 0;
  height: 100%;
  overflow: hidden;
  background: var(--color-editor);
  container-type: inline-size;
  transition: background-color var(--transition-base);
}

.note-toolbar {
  display: flex;
  flex: 0 0 auto;
  gap: 8px;
  align-items: center;
  height: 50px;
  padding: 0 10px 0 10px;
  border-bottom: 1px solid var(--color-border);
}

.note-path {
  display: flex;
  gap: 4px;
  align-items: center;
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 11.5px;
  white-space: nowrap;
}

.note-path--muted {
  color: var(--color-text-muted);
}

.note-path__course {
  flex: 0 1 auto;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.note-path__node {
  flex: 0 1 auto;
  overflow: hidden;
  text-overflow: ellipsis;
}

.note-path__node.is-current {
  color: var(--color-text);
  font-weight: 500;
}

.note-path__sep {
  flex: 0 0 auto;
  color: var(--color-border-strong);
}

.note-toolbar__right {
  display: flex;
  flex: 0 0 auto;
  gap: 7px;
  align-items: center;
  margin-left: auto;
}

.mode-switch {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.mode-switch__option {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  height: 24px;
  padding: 0 7px;
  color: var(--color-text-secondary);
  font-size: 11.5px;
  background: transparent;
  border: none;
  border-radius: 5px;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.mode-switch__option:hover {
  color: var(--color-text);
}

.mode-switch__option.is-active {
  color: var(--color-text);
  background: var(--color-surface);
  box-shadow: var(--shadow-sm);
}

.workspace-banner {
  display: flex;
  flex: 0 0 auto;
  gap: 8px;
  align-items: center;
  padding: 8px 14px;
  color: var(--color-danger);
  font-size: 12px;
  background: var(--color-danger-soft);
}

.workspace-banner__text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.workspace-banner__close {
  padding: 0;
  color: var(--color-danger);
  font-size: 11.5px;
  text-decoration: underline;
  background: none;
  border: none;
}

.workspace-banner--success {
  color: var(--color-success);
  background: var(--color-success-soft);
}

.workspace-banner--success .workspace-banner__close {
  color: var(--color-success);
}

.note-toolbar .icon-button:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}

.workspace-empty {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  justify-content: center;
  padding: 40px 24px;
  text-align: center;
}

.workspace-empty__icon {
  display: grid;
  place-items: center;
  width: 54px;
  height: 54px;
  margin-bottom: 4px;
  color: var(--color-text-muted);
  background: var(--color-surface-soft);
  border-radius: 14px;
}

.workspace-empty h2 {
  font-size: 15px;
  font-weight: 600;
}

.workspace-empty p {
  max-width: 320px;
  color: var(--color-text-muted);
  font-size: 12.5px;
  line-height: 1.7;
}

.workspace-empty .ghost-button {
  margin-top: 8px;
}

.note-heading {
  display: flex;
  flex: 0 0 auto;
  flex-direction: column;
  gap: 4px;
  padding: 14px 26px 8px 28px;
}

.note-title-row {
  display: flex;
  flex: 0 0 auto;
  gap: 12px;
  align-items: center;
}

.note-title {
  flex: 1;
  min-width: 0;
  padding: 0;
  color: var(--color-text);
  font-size: 21px;
  font-weight: 600;
  line-height: 1.4;
  letter-spacing: -0.01em;
  background: transparent;
  border: none;
}

.note-title:focus {
  outline: none;
}

.note-title::placeholder {
  color: var(--color-text-muted);
  font-weight: 500;
}

.note-title__loading {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
}

.note-panes {
  display: flex;
  flex: 1;
  min-height: 0;
}

.note-pane {
  display: flex;
  flex: 1 1 0;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.note-panes--split .note-pane--preview {
  border-left: 1px solid var(--color-border);
}

.note-pane__inner {
  flex: 1;
  min-height: 0;
}

.note-pane--editor .note-pane__inner {
  display: flex;
  justify-content: center;
}

.note-textarea {
  flex: 1;
  width: 100%;
  max-width: 880px;
  height: 100%;
  padding: 8px 28px 72px;
  overflow-y: auto;
  color: var(--color-text);
  font-family: var(--font-mono);
  font-size: 13.5px;
  line-height: 1.8;
  tab-size: 2;
  background: transparent;
  border: none;
  outline: none;
  resize: none;
}

.note-textarea::placeholder {
  color: var(--color-text-muted);
}

.note-pane--preview .note-pane__inner {
  padding: 18px 30px 72px;
  overflow-y: auto;
}

.preview-body {
  max-width: 780px;
  margin: 0 auto;
}

.preview-placeholder {
  padding: 32px 0;
  color: var(--color-text-muted);
  font-size: 12.5px;
  text-align: center;
}

@media (max-width: 1180px) {
  .mode-switch__label {
    display: none;
  }
}

/* 编辑区本身较窄时（例如窗口被压缩或辅助面板展开），分屏改为上下排列 */
@container (max-width: 620px) {
  .note-heading {
    padding: 12px 18px 6px 20px;
  }

  .note-panes--split {
    flex-direction: column;
  }

  .note-panes--split .note-pane--preview {
    border-top: 1px solid var(--color-border);
    border-left: none;
  }

  .note-textarea {
    padding: 6px 20px 48px;
  }

  .note-pane--preview .note-pane__inner {
    padding: 16px 20px 40px;
  }
}
</style>
