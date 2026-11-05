<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import PageHeader from '@/components/common/PageHeader.vue'
import CourseFormDialog from '@/components/courses/CourseFormDialog.vue'
import { notesApi } from '@/services/notes'
import { useCoursesStore } from '@/stores/courses'
import type { Course, KnowledgePoint } from '@/types'
import { formatDate } from '@/utils/format'

const props = defineProps<{
  /** 由路由参数 courseId 注入（路由配置中开启了 props: true） */
  courseId: string
}>()

interface FlatPoint {
  point: KnowledgePoint
  depth: number
}

const router = useRouter()
const courses = useCoursesStore()

// 从课程列表进入时先使用缓存数据渲染，避免详情页整页 loading 闪烁
const cachedCourse = courses.courseById(props.courseId)
const course = ref<Course | null>(cachedCourse)
const points = ref<KnowledgePoint[]>([])
const isLoading = ref(cachedCourse === null)
const errorMessage = ref<string | null>(null)
const actionError = ref<string | null>(null)
const formOpen = ref(false)
const deleteOpen = ref(false)
const deleting = ref(false)

const deleteMessage = computed(() => {
  if (!course.value) return ''

  return (
    `确定删除「${course.value.name}」吗？该课程下的 ` +
    `${course.value.knowledge_point_count} 个知识点、${course.value.note_count} 条笔记，` +
    '以及相关的标签关联和知识点关联会一并删除，且无法恢复。'
  )
})

/** 把课程下的知识点按 parent_id 展开成带层级深度的扁平列表 */
const flatPoints = computed<FlatPoint[]>(() => {
  const ids = new Set(points.value.map((point) => point.id))
  const childrenByParent = new Map<string, KnowledgePoint[]>()

  for (const point of points.value) {
    const key = point.parent_id && ids.has(point.parent_id) ? point.parent_id : ''
    const list = childrenByParent.get(key)
    if (list) {
      list.push(point)
    } else {
      childrenByParent.set(key, [point])
    }
  }

  const result: FlatPoint[] = []
  const visited = new Set<string>()

  function walk(parentId: string, depth: number): void {
    for (const point of childrenByParent.get(parentId) ?? []) {
      if (visited.has(point.id)) continue
      visited.add(point.id)
      result.push({ point, depth })
      walk(point.id, depth + 1)
    }
  }

  walk('', 0)

  // 兜底：parent_id 异常（例如指向其它课程）的知识点也要展示出来
  for (const point of points.value) {
    if (!visited.has(point.id)) {
      result.push({ point, depth: 0 })
    }
  }

  return result
})

async function load(): Promise<void> {
  if (!course.value) isLoading.value = true
  errorMessage.value = null

  try {
    const [loadedCourse, allPoints] = await Promise.all([
      courses.getCourse(props.courseId),
      notesApi.listKnowledgePoints(),
    ])

    course.value = loadedCourse
    points.value = allPoints.filter((point) => point.course_id === loadedCourse.id)
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)

    if (course.value) {
      // 已有缓存时静默刷新失败：保留页面内容，用顶部提示告知
      actionError.value = message
    } else {
      points.value = []
      errorMessage.value = message
    }
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  void load()
})

function onCourseSaved(saved: Course): void {
  course.value = saved
  actionError.value = null
  formOpen.value = false
}

function requestDelete(): void {
  if (!course.value) return
  actionError.value = null
  deleteOpen.value = true
}

function cancelDelete(): void {
  if (deleting.value) return
  deleteOpen.value = false
}

async function confirmDelete(): Promise<void> {
  const target = course.value
  if (!target || deleting.value) return

  deleting.value = true
  actionError.value = null

  try {
    await courses.deleteCourse(target.id)
    deleteOpen.value = false
    await router.replace({ name: 'courses' })
  } catch (error) {
    deleteOpen.value = false
    actionError.value = error instanceof Error ? error.message : String(error)
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <div class="page">
    <PageHeader
      :title="course?.name ?? '课程详情'"
      description="查看课程信息、知识点结构与笔记 / 知识图谱入口。"
    >
      <template #actions>
        <RouterLink to="/courses" class="ghost-button">返回列表</RouterLink>
        <button v-if="course" type="button" class="ghost-button" @click="formOpen = true">
          <AppIcon name="pencil" :size="15" />
          编辑
        </button>
        <button v-if="course" type="button" class="danger-button" @click="requestDelete">
          <AppIcon name="trash" :size="15" />
          删除
        </button>
      </template>
    </PageHeader>

    <div v-if="actionError" class="alert-bar" role="alert">
      <AppIcon name="alert" :size="16" />
      <span>{{ actionError }}</span>
      <button type="button" class="alert-bar__dismiss" @click="actionError = null">关闭</button>
    </div>

    <div v-if="isLoading" class="detail-state">
      <span class="spinner" />
      <span>正在加载课程…</span>
    </div>

    <div v-else-if="errorMessage" class="empty-panel detail-error">
      <div class="empty-panel__icon">
        <AppIcon name="alert" :size="20" />
      </div>
      <h3>课程加载失败</h3>
      <p>{{ errorMessage }}</p>
      <div class="detail-error__actions">
        <RouterLink to="/courses" class="ghost-button">返回列表</RouterLink>
        <button type="button" class="primary-button" @click="load">重试</button>
      </div>
    </div>

    <template v-else-if="course">
      <section class="stat-grid" aria-label="课程统计">
        <article class="stat-tile">
          <span class="stat-tile__icon">
            <AppIcon name="graph" :size="18" />
          </span>
          <div class="stat-tile__body">
            <strong>{{ course.knowledge_point_count }}</strong>
            <span>知识点</span>
          </div>
        </article>

        <article class="stat-tile">
          <span class="stat-tile__icon">
            <AppIcon name="notes" :size="18" />
          </span>
          <div class="stat-tile__body">
            <strong>{{ course.note_count }}</strong>
            <span>笔记</span>
          </div>
        </article>

        <article class="stat-tile stat-tile--wide">
          <span class="stat-tile__icon">
            <AppIcon name="courses" :size="18" />
          </span>
          <div class="stat-tile__body">
            <strong class="stat-tile__date">{{ formatDate(course.updated_at) }}</strong>
            <span>最近更新</span>
          </div>
        </article>
      </section>

      <section class="detail-card">
        <header class="detail-card__head">
          <h2>课程简介</h2>
          <span>创建于 {{ formatDate(course.created_at) }}</span>
        </header>
        <div class="detail-card__body">
          <p class="course-description" :class="{ 'is-empty': !course.description }">
            {{ course.description || '暂无课程描述，点击右上角「编辑」补充课程简介。' }}
          </p>
        </div>
      </section>

      <section class="detail-card">
        <header class="detail-card__head">
          <h2>
            知识点结构
            <span class="count-badge">{{ points.length }}</span>
          </h2>
          <span>阶段三将支持在此维护知识点</span>
        </header>

        <ul v-if="flatPoints.length" class="point-list">
          <li
            v-for="{ point, depth } in flatPoints"
            :key="point.id"
            class="point-row"
            :style="{ paddingLeft: `${18 + depth * 18}px` }"
          >
            <span class="point-row__dot" aria-hidden="true" />
            <span class="point-row__title">{{ point.title }}</span>
            <span v-if="point.description" class="point-row__desc">
              {{ point.description }}
            </span>
          </li>
        </ul>

        <div v-else class="point-empty">
          <p>这门课程还没有知识点。</p>
          <RouterLink to="/notes" class="text-link">前往笔记工作区创建第一个知识点</RouterLink>
        </div>
      </section>

      <section class="quick-grid" aria-label="课程快捷入口">
        <RouterLink to="/notes" class="quick-card">
          <span class="quick-card__icon">
            <AppIcon name="notes" :size="19" />
          </span>
          <span class="quick-card__text">
            <strong>笔记工作区</strong>
            <span>在知识点下编写 Markdown 笔记</span>
          </span>
          <AppIcon name="arrow-right" :size="16" class="quick-card__arrow" />
        </RouterLink>

        <RouterLink to="/graph" class="quick-card">
          <span class="quick-card__icon">
            <AppIcon name="graph" :size="19" />
          </span>
          <span class="quick-card__text">
            <strong>知识图谱</strong>
            <span>查看知识点之间的语义关联</span>
          </span>
          <AppIcon name="arrow-right" :size="16" class="quick-card__arrow" />
        </RouterLink>
      </section>
    </template>

    <CourseFormDialog
      :open="formOpen"
      :course="course"
      @close="formOpen = false"
      @saved="onCourseSaved"
    />

    <ConfirmDialog
      :open="deleteOpen"
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
.stat-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
  gap: 14px;
}

.stat-tile {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 16px 18px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
}

.stat-tile--wide {
  grid-column: span 1;
}

.stat-tile__icon {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 36px;
  height: 36px;
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
  border-radius: 10px;
}

.stat-tile__body {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.stat-tile__body strong {
  overflow: hidden;
  font-size: 22px;
  font-weight: 600;
  line-height: 1.2;
  letter-spacing: -0.02em;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.stat-tile__body strong.stat-tile__date {
  font-size: 14px;
  letter-spacing: 0;
}

.stat-tile__body span {
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.detail-card {
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
}

.detail-card__head {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 12px;
  align-items: baseline;
  justify-content: space-between;
  padding: 15px 22px;
  border-bottom: 1px solid var(--color-border);
}

.detail-card__head h2 {
  display: flex;
  gap: 7px;
  align-items: center;
  font-size: 14px;
  font-weight: 600;
}

.detail-card__head > span {
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.count-badge {
  padding: 0 7px;
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 500;
  line-height: 18px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.detail-card__body {
  padding: 18px 22px 20px;
}

.course-description {
  color: var(--color-text-secondary);
  font-size: 13px;
  line-height: 1.8;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.course-description.is-empty {
  color: var(--color-text-muted);
}

.point-list {
  padding: 6px 0 8px;
}

.point-row {
  display: flex;
  gap: 8px;
  align-items: baseline;
  min-height: 34px;
  padding-top: 7px;
  padding-right: 18px;
  padding-bottom: 7px;
  color: var(--color-text-secondary);
  font-size: 12.5px;
  border-top: 1px solid var(--color-border);
}

.point-list .point-row:first-child {
  border-top: none;
}

.point-row__dot {
  flex: 0 0 auto;
  width: 5px;
  height: 5px;
  background: var(--color-accent);
  border-radius: 50%;
  opacity: 0.6;
  transform: translateY(-1px);
}

.point-row__title {
  color: var(--color-text);
  font-weight: 500;
  overflow-wrap: anywhere;
}

.point-row__desc {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 11.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.point-empty {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
  padding: 34px 18px;
  color: var(--color-text-muted);
  font-size: 12.5px;
  text-align: center;
}

.text-link {
  color: var(--color-accent-strong);
  font-size: 12.5px;
  font-weight: 500;
}

.text-link:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
}

.quick-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 14px;
}

.quick-card {
  display: flex;
  gap: 12px;
  align-items: center;
  padding: 16px 18px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.quick-card:hover {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-md);
}

.quick-card__icon {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 38px;
  height: 38px;
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
  border-radius: 11px;
}

.quick-card__text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.quick-card__text strong {
  font-size: 13.5px;
  font-weight: 600;
}

.quick-card__text span {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 11.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.quick-card__arrow {
  flex: 0 0 auto;
  margin-left: auto;
  color: var(--color-text-muted);
  transition:
    color var(--transition-fast),
    transform var(--transition-fast);
}

.quick-card:hover .quick-card__arrow {
  color: var(--color-accent);
  transform: translateX(2px);
}

.detail-state {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  color: var(--color-text-muted);
  font-size: 13px;
}

.detail-error {
  gap: 8px;
  padding: 48px 18px;
  background: var(--color-surface);
  border: 1px dashed var(--color-border-strong);
  border-radius: var(--radius-lg);
}

.detail-error__actions {
  display: flex;
  gap: 8px;
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
</style>
