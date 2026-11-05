<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import PageHeader from '@/components/common/PageHeader.vue'
import KnowledgePointDetailPanel from '@/components/knowledge/KnowledgePointDetailPanel.vue'
import KnowledgePointFormDialog from '@/components/knowledge/KnowledgePointFormDialog.vue'
import KnowledgePointTree from '@/components/knowledge/KnowledgePointTree.vue'
import { knowledgePointsApi } from '@/services/knowledgePoints'
import { useKnowledgePointsStore } from '@/stores/knowledgePoints'
import type { KnowledgePoint, KnowledgePointDetail, KnowledgePointNode } from '@/types'

const props = defineProps<{
  /** 由路由参数 courseId 注入（可选，用于从课程详情页直接进入某门课程） */
  courseId?: string
}>()

const route = useRoute()
const router = useRouter()
const store = useKnowledgePointsStore()

const formOpen = ref(false)
const editingPoint = ref<KnowledgePoint | null>(null)
const formParentId = ref<string | null>(null)

const pendingDelete = ref<KnowledgePointNode | null>(null)
const deleteDetail = ref<KnowledgePointDetail | null>(null)
const deleting = ref(false)
const hierarchyBusy = ref(false)
const courseBusy = ref(false)

const actionError = ref<string | null>(null)
const notice = ref<string | null>(null)

const deleteMessage = computed(() => {
  const node = pendingDelete.value
  if (!node) return ''

  const detail = deleteDetail.value
  const extras: string[] = []

  if (detail) {
    if (detail.descendant_count > 0) {
      extras.push(`${detail.descendant_count} 个后代知识点`)
    }
    if (detail.subtree_note_count > 0) {
      extras.push(`${detail.subtree_note_count} 条笔记`)
    }
  } else if (node.child_count > 0) {
    extras.push(`${node.child_count} 个子知识点`)
  }

  const body = extras.length
    ? `其下的 ${extras.join('、')}以及相关知识点关联会一并删除，`
    : '相关知识点关联会一并删除，'

  return `确定删除「${node.title}」吗？${body}且无法恢复。`
})

onMounted(async () => {
  await store.initialize(props.courseId || null)

  // 从课程详情页 / 笔记页带过来的知识点：直接选中它
  const pointId = typeof route.query.point === 'string' ? route.query.point : ''
  if (pointId) {
    await store.selectPoint(pointId)
  }
})

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function selectCourse(courseId: string): void {
  actionError.value = null
  notice.value = null
  void store.selectCourse(courseId)
}

function refresh(): void {
  actionError.value = null
  notice.value = null
  void store.refresh()
}

/** 关闭错误提示；若提示来自列表刷新失败，同时清理 store 中的错误 */
function clearError(): void {
  actionError.value = null
  store.dismissError()
}

function openCreate(parentId: string | null = null): void {
  if (!store.courses.length) return

  actionError.value = null
  notice.value = null
  editingPoint.value = null
  formParentId.value = parentId
  formOpen.value = true
}

function openEdit(point: KnowledgePoint): void {
  actionError.value = null
  notice.value = null
  editingPoint.value = point
  formParentId.value = point.parent_id
  formOpen.value = true
}

function onSaved(point: KnowledgePoint): void {
  formOpen.value = false
  actionError.value = null
  notice.value = `已保存「${point.title}」`
}

async function requestDelete(node: KnowledgePointNode): Promise<void> {
  actionError.value = null
  notice.value = null
  pendingDelete.value = node
  deleteDetail.value = null

  try {
    deleteDetail.value = await knowledgePointsApi.detail(node.id)
  } catch {
    // 详情获取失败时使用节点自身统计生成简化提示
  }
}

function cancelDelete(): void {
  if (deleting.value) return
  pendingDelete.value = null
  deleteDetail.value = null
}

async function confirmDelete(): Promise<void> {
  const target = pendingDelete.value
  if (!target || deleting.value) return

  deleting.value = true
  actionError.value = null

  try {
    const result = await store.deletePoint(target.id)
    actionError.value = null
    notice.value =
      result.deleted_points > 1
        ? `已删除「${target.title}」及其下 ${result.deleted_points - 1} 个后代知识点`
        : `已删除「${target.title}」`
  } catch (error) {
    actionError.value = errorText(error)
  } finally {
    deleting.value = false
    pendingDelete.value = null
    deleteDetail.value = null
  }
}

async function indentPoint(): Promise<void> {
  const current = store.detail?.point
  if (!current || hierarchyBusy.value) return

  const previous = store.previousSibling(current.id)
  if (!previous) return

  hierarchyBusy.value = true
  actionError.value = null

  try {
    await store.movePoint(current.id, previous.id)
    notice.value = `「${current.title}」已成为「${previous.title}」的子知识点`
  } catch (error) {
    actionError.value = errorText(error)
  } finally {
    hierarchyBusy.value = false
  }
}

async function outdentPoint(): Promise<void> {
  const current = store.detail?.point
  if (!current || hierarchyBusy.value) return

  const parent = store.parentOf(current.id)
  if (!parent) return

  hierarchyBusy.value = true
  actionError.value = null

  try {
    await store.movePoint(current.id, parent.parent_id)
    notice.value = `「${current.title}」已提升一级`
  } catch (error) {
    actionError.value = errorText(error)
  } finally {
    hierarchyBusy.value = false
  }
}

async function moveCourse(courseId: string): Promise<void> {
  const current = store.detail?.point
  if (!current || courseBusy.value) return

  const target = store.courses.find((course) => course.id === courseId)
  if (!target) return

  courseBusy.value = true
  actionError.value = null

  try {
    await store.movePointToCourse(current.id, target.id)
    notice.value = `「${current.title}」已迁移到「${target.name}」课程`
  } catch (error) {
    actionError.value = errorText(error)
  } finally {
    courseBusy.value = false
  }
}

function editSelected(): void {
  const current = store.detail?.point
  if (current) openEdit(current)
}

function deleteSelected(): void {
  const node = store.selectedNode
  if (node) void requestDelete(node)
}

function addChildToSelected(): void {
  const current = store.detail?.point
  openCreate(current?.id ?? null)
}

function retryDetail(): void {
  if (store.selectedId) void store.selectPoint(store.selectedId)
}

function openNote(pointId: string): void {
  void router.push({ name: 'notes', query: { point: pointId } })
}
</script>

<template>
  <div class="page knowledge-page">
    <PageHeader
      title="知识点管理"
      description="为课程维护多级知识点目录：新建、编辑、删除知识点，并随时调整层级与归属课程。"
    >
      <template #actions>
        <button
          type="button"
          class="ghost-button"
          :disabled="!store.activeCourseId || store.isLoading"
          @click="refresh"
        >
          <AppIcon name="refresh" :size="15" />
          刷新
        </button>
        <button
          type="button"
          class="primary-button"
          :disabled="!store.courses.length"
          @click="openCreate()"
        >
          <AppIcon name="plus" :size="16" />
          新建知识点
        </button>
      </template>
    </PageHeader>

    <div
      v-if="actionError || (store.errorMessage && store.tree.length)"
      class="alert-bar"
      role="alert"
    >
      <AppIcon name="alert" :size="16" />
      <span>{{ actionError || store.errorMessage }}</span>
      <button type="button" class="alert-bar__dismiss" @click="clearError">关闭</button>
    </div>

    <div v-if="notice" class="notice-bar" role="status">
      <AppIcon name="sparkles" :size="15" />
      <span>{{ notice }}</span>
      <button type="button" class="alert-bar__dismiss" @click="notice = null">关闭</button>
    </div>

    <div v-if="store.isLoading && !store.initialized" class="page-state">
      <span class="spinner" />
      <span>正在加载课程与知识点…</span>
    </div>

    <div
      v-else-if="!store.courses.length"
      class="empty-panel knowledge-empty"
    >
      <div class="empty-panel__icon">
        <AppIcon name="tree" :size="22" />
      </div>
      <h3>还没有课程</h3>
      <p>知识点必须挂在课程下，先创建一门课程再来维护知识点目录。</p>
      <RouterLink to="/courses" class="primary-button">
        <AppIcon name="courses" :size="16" />
        前往课程管理
      </RouterLink>
    </div>

    <div v-else class="knowledge-layout">
      <aside class="pane pane--courses" aria-label="课程列表">
        <header class="pane__head">
          <h2>课程</h2>
          <span class="count-badge">{{ store.courses.length }}</span>
        </header>

        <div class="pane__body pane__body--courses">
          <button
            v-for="course in store.courses"
            :key="course.id"
            type="button"
            class="course-option"
            :class="{ 'is-active': course.id === store.activeCourseId }"
            @click="selectCourse(course.id)"
          >
            <span class="course-option__name" :title="course.name">{{ course.name }}</span>
            <span class="course-option__count">{{ store.pointCountOf(course.id) }}</span>
          </button>
        </div>

        <footer class="pane__foot">
          {{ store.courses.length }} 门课程 · {{ store.allPoints.length }} 个知识点
        </footer>
      </aside>

      <section class="pane pane--tree" aria-label="知识点树">
        <header class="pane__head">
          <h2 :title="store.activeCourse?.name">
            {{ store.activeCourse?.name ?? '知识点树' }}
          </h2>
          <span class="count-badge">{{ store.totalCount }}</span>
          <button
            type="button"
            class="icon-button pane__head-action"
            title="新建顶层知识点"
            aria-label="新建顶层知识点"
            :disabled="!store.activeCourseId"
            @click="openCreate()"
          >
            <AppIcon name="plus" :size="15" />
          </button>
        </header>

        <div class="pane__body pane__body--tree">
          <div v-if="store.isLoading" class="pane-state">
            <span class="spinner" />
            <span>正在加载知识点…</span>
          </div>

          <div
            v-else-if="store.errorMessage && !store.tree.length"
            class="pane-state pane-state--error"
          >
            <AppIcon name="alert" :size="16" />
            <span>{{ store.errorMessage }}</span>
            <button type="button" class="ghost-button" @click="refresh">重试</button>
          </div>

          <KnowledgePointTree
            v-else-if="store.tree.length"
            :nodes="store.tree"
            @add-child="openCreate($event.id)"
            @edit="openEdit"
            @delete="requestDelete"
          />

          <div v-else class="empty-panel">
            <div class="empty-panel__icon">
              <AppIcon name="tree" :size="20" />
            </div>
            <h3>这门课程还没有知识点</h3>
            <p>从顶层知识点开始，逐步拆分出多级目录。</p>
            <button type="button" class="primary-button" @click="openCreate()">
              <AppIcon name="plus" :size="16" />
              新建知识点
            </button>
          </div>
        </div>

        <footer v-if="store.tree.length" class="pane__foot">
          悬停行可新建子知识点 / 编辑 / 删除，详情面板可调整层级
        </footer>
      </section>

      <aside class="pane pane--detail" aria-label="知识点详情">
        <KnowledgePointDetailPanel
          :hierarchy-busy="hierarchyBusy"
          :course-busy="courseBusy"
          @edit="editSelected"
          @delete="deleteSelected"
          @add-child="addChildToSelected"
          @open-note="openNote"
          @indent="indentPoint"
          @outdent="outdentPoint"
          @move-course="moveCourse"
          @retry="retryDetail"
        />
      </aside>
    </div>

    <KnowledgePointFormDialog
      :open="formOpen"
      :course-name="store.activeCourse?.name ?? ''"
      :tree="store.tree"
      :point="editingPoint"
      :default-parent-id="formParentId"
      @close="formOpen = false"
      @saved="onSaved"
    />

    <ConfirmDialog
      :open="pendingDelete !== null"
      title="删除知识点"
      :message="deleteMessage"
      confirm-text="删除知识点"
      :busy="deleting"
      @cancel="cancelDelete"
      @confirm="confirmDelete"
    />
  </div>
</template>

<style scoped>
.knowledge-layout {
  display: grid;
  grid-template-columns: 210px minmax(0, 1fr) 320px;
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
  min-height: 42px;
  padding: 0 10px 0 14px;
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
  margin-left: auto;
  padding: 0 7px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  font-weight: 500;
  line-height: 18px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.pane__head-action {
  flex: 0 0 auto;
  width: 26px;
  height: 26px;
}

.pane__head-action:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}

.pane__body {
  min-height: 120px;
  padding: 8px;
  overflow-y: auto;
}

.pane__body--courses {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: min(58vh, 560px);
}

.pane__body--tree {
  max-height: min(62vh, 640px);
}

.pane__foot {
  flex: 0 0 auto;
  padding: 9px 14px;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 10.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
  border-top: 1px solid var(--color-border);
}

.course-option {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
  padding: 7px 10px;
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

.course-option:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.course-option.is-active {
  color: var(--color-accent-strong);
  font-weight: 500;
  background: var(--color-accent-soft);
}

.course-option__name {
  flex: 1;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.course-option__count {
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

.knowledge-empty {
  gap: 8px;
  padding: 56px 18px;
  background: var(--color-surface);
  border: 1px dashed var(--color-border-strong);
  border-radius: var(--radius-lg);
}

.knowledge-empty .primary-button {
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

@media (max-width: 1180px) {
  .knowledge-layout {
    grid-template-columns: 200px minmax(0, 1fr);
  }

  .pane--detail {
    grid-column: 1 / -1;
  }
}

@media (max-width: 760px) {
  .knowledge-layout {
    grid-template-columns: minmax(0, 1fr);
  }

  .pane--detail {
    grid-column: auto;
  }
}
</style>
