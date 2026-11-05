<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import CourseFormDialog from '@/components/courses/CourseFormDialog.vue'
import KnowledgePointFormDialog from './KnowledgePointFormDialog.vue'
import KnowledgeTreeNode from './KnowledgeTreeNode.vue'
import { useNotesStore } from '@/stores/notes'
import { useWorkspaceStore } from '@/stores/workspace'
import type { Course } from '@/types'

const notes = useNotesStore()
const workspace = useWorkspaceStore()

// 默认展开全部课程，只记录用户手动收起的课程
const collapsedCourses = ref<Set<string>>(new Set())
const courseDialogOpen = ref(false)
const pointDialogOpen = ref(false)
const pointDialogCourseId = ref<string | null>(null)
const pointDialogParentId = ref<string | null>(null)

const sidebarStyle = computed(() => ({
  width: workspace.sidebarCollapsed ? '0px' : `${workspace.sidebarWidth}px`,
}))

function isCourseExpanded(courseId: string): boolean {
  return !collapsedCourses.value.has(courseId)
}

function toggleCourse(courseId: string): void {
  const next = new Set(collapsedCourses.value)
  if (next.has(courseId)) {
    next.delete(courseId)
  } else {
    next.add(courseId)
  }
  collapsedCourses.value = next
}

function openCourseDialog(): void {
  courseDialogOpen.value = true
}

/** 课程表单由课程管理模块共享，创建成功后同步到笔记目录 */
function onCourseSaved(course: Course): void {
  notes.addCourse(course)
}

function openPointDialog(courseId?: string, parentId?: string): void {
  if (!notes.courses.length) {
    openCourseDialog()
    return
  }

  pointDialogCourseId.value = courseId ?? notes.selectedPoint?.course_id ?? null
  pointDialogParentId.value = parentId ?? null
  pointDialogOpen.value = true
}

// 侧边栏宽度拖拽
const resizeStartX = ref(0)
const resizeStartWidth = ref(0)

function startResize(event: PointerEvent): void {
  if (workspace.sidebarCollapsed) return

  event.preventDefault()
  resizeStartX.value = event.clientX
  resizeStartWidth.value = workspace.sidebarWidth
  workspace.beginResize()

  window.addEventListener('pointermove', onResizeMove)
  window.addEventListener('pointerup', stopResize, { once: true })
}

function onResizeMove(event: PointerEvent): void {
  if (!workspace.resizing) return

  const next = resizeStartWidth.value + (event.clientX - resizeStartX.value)
  workspace.setSidebarWidth(next)
}

function stopResize(): void {
  workspace.endResize()
  window.removeEventListener('pointermove', onResizeMove)
}

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', onResizeMove)
  window.removeEventListener('pointerup', stopResize)
})
</script>

<template>
  <aside
    class="notes-sidebar"
    :class="{
      'is-collapsed': workspace.sidebarCollapsed,
      'is-resizing': workspace.resizing,
    }"
    :style="sidebarStyle"
    aria-label="课程与知识点目录"
  >
    <div class="notes-sidebar__inner" :style="{ width: `${workspace.sidebarWidth}px` }">
      <header class="notes-sidebar__header">
        <span>课程与知识点</span>
        <div class="notes-sidebar__actions">
          <button
            type="button"
            class="icon-button"
            title="新建课程"
            aria-label="新建课程"
            @click="openCourseDialog"
          >
            <AppIcon name="plus" :size="16" />
          </button>
          <button
            type="button"
            class="icon-button"
            title="新建知识点"
            aria-label="新建知识点"
            :disabled="!notes.courses.length"
            @click="openPointDialog()"
          >
            <AppIcon name="notes" :size="16" />
          </button>
        </div>
      </header>

      <div class="notes-sidebar__body">
        <div v-if="notes.isLoading" class="sidebar-loading">
          <span class="spinner" />
          <span>正在加载课程…</span>
        </div>

        <div v-else-if="notes.errorMessage && !notes.courses.length" class="empty-panel">
          <div class="empty-panel__icon">
            <AppIcon name="alert" :size="20" />
          </div>
          <h3>目录加载失败</h3>
          <p>{{ notes.errorMessage }}</p>
          <button type="button" class="ghost-button" @click="notes.initialize(true)">重试</button>
        </div>

        <div v-else-if="!notes.courses.length" class="empty-panel">
          <div class="empty-panel__icon">
            <AppIcon name="courses" :size="20" />
          </div>
          <h3>还没有课程</h3>
          <p>创建课程后，就可以添加知识点并开始记笔记。</p>
          <button type="button" class="ghost-button" @click="openCourseDialog">新建课程</button>
        </div>

        <ul v-else class="course-list">
          <li v-for="course in notes.courses" :key="course.id" class="course-item">
            <div
              class="course-row"
              :class="{ 'is-open': isCourseExpanded(course.id) }"
              @click="toggleCourse(course.id)"
            >
              <AppIcon name="chevron-right" :size="13" class="course-row__chevron" />
              <span class="course-row__name" :title="course.name">{{ course.name }}</span>
              <span class="course-row__count">{{ notes.pointsOfCourse(course.id).length }}</span>
              <button
                type="button"
                class="row-action"
                title="在此课程下新建知识点"
                aria-label="在此课程下新建知识点"
                @click.stop="openPointDialog(course.id)"
              >
                <AppIcon name="plus" :size="14" />
              </button>
            </div>

            <ul v-if="isCourseExpanded(course.id)" class="course-points">
              <KnowledgeTreeNode
                v-for="point in notes.rootPointsOf(course.id)"
                :key="point.id"
                :point="point"
                :depth="0"
              />
              <li v-if="!notes.rootPointsOf(course.id).length" class="course-empty">
                暂无知识点
                <button type="button" class="text-button" @click="openPointDialog(course.id)">
                  添加
                </button>
              </li>
            </ul>
          </li>
        </ul>
      </div>

      <footer v-if="notes.courses.length" class="notes-sidebar__footer">
        {{ notes.courses.length }} 门课程 · {{ notes.knowledgePoints.length }} 个知识点
      </footer>
    </div>

    <div
      v-if="!workspace.sidebarCollapsed"
      class="notes-sidebar__resizer"
      title="拖动调整目录宽度"
      @pointerdown="startResize"
    />

    <CourseFormDialog
      :open="courseDialogOpen"
      @close="courseDialogOpen = false"
      @saved="onCourseSaved"
    />
    <KnowledgePointFormDialog
      :open="pointDialogOpen"
      :course-id="pointDialogCourseId"
      :parent-id="pointDialogParentId"
      @close="pointDialogOpen = false"
    />
  </aside>
</template>

<style scoped>
.notes-sidebar {
  position: relative;
  display: flex;
  flex: 0 0 auto;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--color-surface);
  border-right: 1px solid var(--color-border);
  transition:
    width 180ms ease,
    background-color var(--transition-base),
    border-color var(--transition-base);
}

.notes-sidebar.is-collapsed {
  border-right-color: transparent;
}

.notes-sidebar.is-resizing {
  transition: none;
}

.notes-sidebar__inner {
  display: flex;
  flex: 0 0 auto;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.notes-sidebar__header {
  display: flex;
  flex: 0 0 auto;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
  height: 50px;
  padding: 0 8px 0 14px;
  border-bottom: 1px solid var(--color-border);
}

.notes-sidebar__header > span {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.02em;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.notes-sidebar__actions {
  display: flex;
  flex: 0 0 auto;
  gap: 2px;
}

.notes-sidebar__header .icon-button:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}

.notes-sidebar__body {
  flex: 1;
  padding: 8px;
  overflow-x: hidden;
  overflow-y: auto;
}

.course-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.course-row {
  display: flex;
  gap: 5px;
  align-items: center;
  height: 32px;
  padding: 0 4px 0 6px;
  color: var(--color-text-secondary);
  border-radius: 6px;
  cursor: pointer;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.course-row:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.course-row__chevron {
  flex: 0 0 auto;
  color: var(--color-text-muted);
  transition: transform 150ms ease;
}

.course-row.is-open .course-row__chevron {
  transform: rotate(90deg);
}

.course-row__name {
  flex: 1;
  overflow: hidden;
  font-size: 12.5px;
  font-weight: 500;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.course-row__count {
  flex: 0 0 auto;
  color: var(--color-text-muted);
  font-size: 10.5px;
}

.row-action {
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
  opacity: 0;
  transition:
    opacity var(--transition-fast),
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.course-row:hover .row-action,
.row-action:focus-visible {
  opacity: 1;
}

.row-action:hover {
  color: var(--color-accent);
  background: var(--color-accent-soft);
}

.course-points {
  padding: 2px 0 4px;
}

.course-empty {
  padding: 4px 12px 6px 28px;
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.text-button {
  padding: 0;
  color: var(--color-accent);
  font-size: 11.5px;
  background: none;
  border: none;
}

.text-button:hover {
  text-decoration: underline;
}

.sidebar-loading {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 12px 8px;
  color: var(--color-text-muted);
  font-size: 12px;
}

.notes-sidebar__footer {
  flex: 0 0 auto;
  padding: 8px 14px 12px;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 11px;
  white-space: nowrap;
  text-overflow: ellipsis;
  border-top: 1px solid var(--color-border);
}

.notes-sidebar__resizer {
  position: absolute;
  top: 0;
  right: -3px;
  z-index: 3;
  width: 6px;
  height: 100%;
  cursor: col-resize;
}

.notes-sidebar__resizer:hover::after,
.notes-sidebar.is-resizing .notes-sidebar__resizer::after {
  content: '';
  position: absolute;
  top: 0;
  right: 2px;
  width: 2px;
  height: 100%;
  background: var(--color-accent);
  opacity: 0.45;
}
</style>
