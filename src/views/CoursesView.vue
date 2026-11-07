<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import PageHeader from '@/components/common/PageHeader.vue'
import CourseFormDialog from '@/components/courses/CourseFormDialog.vue'
import { useCoursesStore } from '@/stores/courses'
import type { Course } from '@/types'
import { formatDateTime } from '@/utils/format'

const courses = useCoursesStore()

const formOpen = ref(false)
const editingCourse = ref<Course | null>(null)
const pendingDelete = ref<Course | null>(null)
const deleting = ref(false)
const actionError = ref<string | null>(null)

const deleteMessage = computed(() => {
  const target = pendingDelete.value
  if (!target) return ''

  return (
    `确定删除「${target.name}」吗？该课程下的 ` +
    `${target.knowledge_point_count} 个知识点、${target.note_count} 条笔记，` +
    '以及相关的标签关联和知识点关联会一并删除，且无法恢复。'
  )
})

onMounted(() => {
  void courses.fetchCourses(true)
})

function openCreate(): void {
  editingCourse.value = null
  formOpen.value = true
}

function openEdit(course: Course): void {
  editingCourse.value = course
  formOpen.value = true
}

function closeForm(): void {
  formOpen.value = false
}

function onCourseSaved(): void {
  actionError.value = null
  formOpen.value = false
}

/** 关闭错误提示；若提示来自列表刷新失败，同时清理 store 中的错误 */
function clearError(): void {
  actionError.value = null
  courses.dismissError()
}

function requestDelete(course: Course): void {
  actionError.value = null
  pendingDelete.value = course
}

function cancelDelete(): void {
  if (deleting.value) return
  pendingDelete.value = null
}

async function confirmDelete(): Promise<void> {
  const target = pendingDelete.value
  if (!target || deleting.value) return

  deleting.value = true
  actionError.value = null

  try {
    await courses.deleteCourse(target.id)
    pendingDelete.value = null
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
    pendingDelete.value = null
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <div class="page">
    <PageHeader
      title="课程管理"
      description="课程是知识点与笔记的组织入口，在这里完成课程的增删改查。"
    >
      <template #actions>
        <button type="button" class="primary-button" @click="openCreate">
          <AppIcon name="plus" :size="16" />
          新建课程
        </button>
      </template>
    </PageHeader>

    <div
      v-if="actionError || (courses.errorMessage && courses.items.length)"
      class="alert-bar"
      role="alert"
    >
      <AppIcon name="alert" :size="16" />
      <span>{{ actionError || courses.errorMessage }}</span>
      <button type="button" class="alert-bar__dismiss" @click="clearError">关闭</button>
    </div>

    <div v-if="courses.isLoading && !courses.items.length" class="list-state">
      <span class="spinner" />
      <span>正在加载课程…</span>
    </div>

    <div
      v-else-if="courses.errorMessage && !courses.items.length"
      class="empty-panel list-state--error"
    >
      <div class="empty-panel__icon">
        <AppIcon name="alert" :size="20" />
      </div>
      <h3>课程加载失败</h3>
      <p>{{ courses.errorMessage }}</p>
      <button type="button" class="primary-button" @click="courses.fetchCourses(true)">
        重新加载
      </button>
    </div>

    <div v-else-if="!courses.items.length" class="empty-panel course-empty">
      <div class="empty-panel__icon">
        <AppIcon name="courses" :size="22" />
      </div>
      <h3>还没有课程</h3>
      <p>创建第一门课程，用它来组织知识点与 Markdown 笔记。</p>
      <button type="button" class="primary-button" @click="openCreate">
        <AppIcon name="plus" :size="16" />
        新建课程
      </button>
    </div>

    <template v-else>
      <div class="list-meta">
        <span>共 {{ courses.items.length }} 门课程</span>
        <span>数据保存在本地 SQLite，重启应用后仍然存在</span>
      </div>

      <section class="course-grid" aria-label="课程列表">
        <article v-for="course in courses.items" :key="course.id" class="course-card">
          <header class="course-card__head">
            <span class="course-card__icon">
              <AppIcon name="courses" :size="18" />
            </span>

            <div class="course-card__titles">
              <RouterLink :to="`/courses/${course.id}`" class="course-card__name">
                {{ course.name }}
              </RouterLink>
              <span class="course-card__updated">
                更新于 {{ formatDateTime(course.updated_at) }}
              </span>
            </div>

            <div class="course-card__actions">
              <button
                type="button"
                class="card-action"
                title="编辑课程"
                aria-label="编辑课程"
                @click="openEdit(course)"
              >
                <AppIcon name="pencil" :size="15" />
              </button>
              <button
                type="button"
                class="card-action card-action--danger"
                title="删除课程"
                aria-label="删除课程"
                @click="requestDelete(course)"
              >
                <AppIcon name="trash" :size="15" />
              </button>
            </div>
          </header>

          <p class="course-card__description" :class="{ 'is-empty': !course.description }">
            {{ course.description || '暂无课程描述' }}
          </p>

          <footer class="course-card__foot">
            <div class="course-card__stats">
              <span><strong>{{ course.knowledge_point_count }}</strong> 知识点</span>
              <span class="course-card__dot" aria-hidden="true">·</span>
              <span><strong>{{ course.note_count }}</strong> 笔记</span>
            </div>

            <RouterLink :to="`/courses/${course.id}`" class="course-card__link">
              进入课程
              <AppIcon name="arrow-right" :size="14" />
            </RouterLink>
          </footer>
        </article>
      </section>
    </template>

    <CourseFormDialog
      :open="formOpen"
      :course="editingCourse"
      @close="closeForm"
      @saved="onCourseSaved"
    />

    <ConfirmDialog
      :open="pendingDelete !== null"
      title="删除课程"
      :message="deleteMessage"
      confirm-text="删除课程"
      :busy="deleting"
      @cancel="cancelDelete"
      @confirm="confirmDelete"
    />
  </div>
</template>

<style scoped>
.list-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 16px;
  align-items: baseline;
  justify-content: space-between;
  color: var(--color-text-muted);
  font-size: 12px;
}

.course-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.course-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 178px;
  padding: 18px 20px 16px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  transition:
    background-color var(--transition-fast),
    border-color var(--transition-fast);
}

.course-card:hover {
  background: var(--color-surface-soft);
  border-color: var(--color-border-strong);
}

.course-card__head {
  display: flex;
  gap: 11px;
  align-items: flex-start;
}

.course-card__icon {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 34px;
  height: 34px;
  color: var(--color-text-secondary);
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 7px;
}

.course-card__titles {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.course-card__name {
  display: -webkit-box;
  overflow: hidden;
  color: var(--color-text);
  font-size: 14.5px;
  font-weight: 600;
  line-height: 1.4;
  overflow-wrap: anywhere;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  transition: color var(--transition-fast);
}

.course-card__name:hover {
  color: var(--color-accent-strong);
}

.course-card__updated {
  color: var(--color-text-muted);
  font-size: 11px;
}

.course-card__actions {
  display: flex;
  flex: 0 0 auto;
  gap: 2px;
  margin-left: auto;
}

.card-action {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  padding: 0;
  color: var(--color-text-muted);
  background: transparent;
  border: none;
  border-radius: 6px;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.card-action:hover {
  color: var(--color-accent);
  background: var(--color-accent-soft);
}

.card-action--danger:hover {
  color: var(--color-danger);
  background: var(--color-danger-soft);
}

.course-card__description {
  display: -webkit-box;
  flex: 1;
  min-height: 38px;
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 12.5px;
  line-height: 1.6;
  overflow-wrap: anywhere;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
}

.course-card__description.is-empty {
  color: var(--color-text-muted);
}

.course-card__foot {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 12px;
  align-items: center;
  justify-content: space-between;
  padding-top: 12px;
  border-top: 1px solid var(--color-border);
}

.course-card__stats {
  display: flex;
  gap: 5px;
  align-items: baseline;
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.course-card__stats strong {
  color: var(--color-text);
  font-size: 12.5px;
  font-weight: 600;
}

.course-card__dot {
  color: var(--color-border-strong);
}

.course-card__link {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  color: var(--color-accent-strong);
  font-size: 12px;
  font-weight: 500;
  transition: gap var(--transition-fast);
}

.course-card__link:hover {
  gap: 7px;
}

.list-state {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: center;
  padding: 56px 0;
  color: var(--color-text-muted);
  font-size: 13px;
}

.list-state--error {
  padding: 48px 18px;
}

.course-empty {
  gap: 8px;
  padding: 56px 18px;
  background: var(--color-surface);
  border: 1px dashed var(--color-border-strong);
  border-radius: var(--radius-lg);
}

.course-empty .primary-button,
.list-state--error .primary-button {
  margin-top: 6px;
}

.alert-bar {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 10px 14px;
  color: var(--color-danger);
  font-size: 12.5px;
  background: var(--color-danger-soft);
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-sm);
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

@media (max-width: 860px) {
  .course-grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
