<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import type { IconName } from '@/components/common/icons'
import { coursesApi } from '@/services/courses'
import { notesApi } from '@/services/notes'
import type { Course, NoteSummary } from '@/types'
import { formatDateTime } from '@/utils/format'

interface QuickLink {
  title: string
  description: string
  path: string
  icon: IconName
}

const courses = ref<Course[]>([])
const recentNotes = ref<NoteSummary[]>([])
const isLoading = ref(true)
const errorMessage = ref('')

const quickLinks: QuickLink[] = [
  {
    title: '笔记工作区',
    description: '编辑、分屏预览与自动保存',
    path: '/notes',
    icon: 'notes',
  },
  {
    title: '课程管理',
    description: '整理学习主题与课程结构',
    path: '/courses',
    icon: 'courses',
  },
  {
    title: '知识点管理',
    description: '维护知识的层级与归属',
    path: '/knowledge-points',
    icon: 'tree',
  },
  {
    title: '笔记库',
    description: '搜索、筛选与导入导出',
    path: '/note-library',
    icon: 'library',
  },
  {
    title: '标签管理',
    description: '用标签连接不同课程的内容',
    path: '/tags',
    icon: 'tag',
  },
  {
    title: '知识图谱',
    description: '查看知识点之间的关联',
    path: '/graph',
    icon: 'graph',
  },
]

const stats = computed(() => {
  const unavailable = isLoading.value || Boolean(errorMessage.value)

  return [
    {
      label: '课程',
      value: unavailable ? '—' : courses.value.length,
      hint: '学习主题',
      path: '/courses',
    },
    {
      label: '知识点',
      value: unavailable
        ? '—'
        : courses.value.reduce((total, course) => total + course.knowledge_point_count, 0),
      hint: '结构化内容',
      path: '/knowledge-points',
    },
    {
      label: '笔记',
      value: unavailable ? '—' : recentNotes.value.length,
      hint: '持续积累',
      path: '/note-library',
    },
  ]
})

async function loadWorkspace(): Promise<void> {
  isLoading.value = true
  errorMessage.value = ''

  try {
    const [courseItems, noteItems] = await Promise.all([
      coursesApi.list(),
      notesApi.listNotes(),
    ])
    courses.value = courseItems
    recentNotes.value = noteItems
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  void loadWorkspace()
})
</script>

<template>
  <div class="page home-page">
    <header class="home-hero">
      <div class="home-hero__copy">
        <span class="home-hero__eyebrow">个人知识工作区</span>
        <h1>继续你的思考与记录。</h1>
        <p>从课程到知识点，再到每一页笔记，让分散的内容逐渐形成清晰的知识脉络。</p>
      </div>
      <div class="home-hero__actions">
        <RouterLink to="/notes" class="primary-button">
          <AppIcon name="pencil" :size="15" />
          开始记录
        </RouterLink>
        <RouterLink to="/courses" class="ghost-button">
          <AppIcon name="plus" :size="15" />
          新建课程
        </RouterLink>
      </div>
    </header>

    <div v-if="errorMessage" class="home-error" role="alert">
      <AppIcon name="alert" :size="16" />
      <span>知识库数据加载失败：{{ errorMessage }}</span>
      <button type="button" @click="loadWorkspace">重新加载</button>
    </div>

    <section class="overview" aria-label="知识库概览" :aria-busy="isLoading">
      <RouterLink v-for="stat in stats" :key="stat.label" :to="stat.path" class="overview__item">
        <span class="overview__label">{{ stat.label }}</span>
        <strong class="overview__value">{{ stat.value }}</strong>
        <span class="overview__hint">{{ stat.hint }}</span>
      </RouterLink>
    </section>

    <div class="home-grid">
      <section class="home-section recent-section" aria-labelledby="recent-title">
        <header class="section-header">
          <div>
            <h2 id="recent-title">最近编辑</h2>
            <p>从上次的思路继续</p>
          </div>
          <RouterLink to="/note-library" class="section-link">
            全部笔记
            <AppIcon name="arrow-right" :size="14" />
          </RouterLink>
        </header>

        <div v-if="isLoading" class="recent-skeleton" role="status" aria-label="正在加载最近笔记">
          <div v-for="index in 4" :key="index" class="skeleton-row">
            <span class="skeleton-block skeleton-block--icon" />
            <span class="skeleton-lines">
              <span class="skeleton-block skeleton-block--title" />
              <span class="skeleton-block skeleton-block--text" />
            </span>
          </div>
        </div>

        <div v-else-if="errorMessage" class="recent-unavailable">
          <AppIcon name="alert" :size="20" />
          <span>暂时无法读取最近笔记</span>
        </div>

        <div v-else-if="recentNotes.length" class="recent-list">
          <RouterLink
            v-for="note in recentNotes.slice(0, 5)"
            :key="note.id"
            :to="{ name: 'notes', query: { point: note.knowledge_point_id } }"
            class="recent-note"
          >
            <span class="recent-note__icon">
              <AppIcon name="notes" :size="17" />
            </span>
            <span class="recent-note__body">
              <strong class="recent-note__title">{{ note.title }}</strong>
              <span class="recent-note__excerpt">
                {{ note.excerpt || '这页笔记还没有正文内容。' }}
              </span>
              <span class="recent-note__meta">
                {{ note.course_name }} / {{ note.knowledge_point_title }}
                <span aria-hidden="true">·</span>
                {{ formatDateTime(note.updated_at) }}
              </span>
            </span>
            <AppIcon name="chevron-right" :size="15" class="recent-note__arrow" />
          </RouterLink>
        </div>

        <div v-else class="recent-empty">
          <span class="recent-empty__icon">
            <AppIcon name="notes" :size="22" />
          </span>
          <h3>从第一页笔记开始</h3>
          <p>选择一个知识点，把今天的想法、问题与收获记录下来。</p>
          <RouterLink to="/notes" class="section-link">
            打开笔记工作区
            <AppIcon name="arrow-right" :size="14" />
          </RouterLink>
        </div>
      </section>

      <aside class="home-section quick-section" aria-labelledby="quick-title">
        <header class="section-header">
          <div>
            <h2 id="quick-title">快速访问</h2>
            <p>常用工具与视图</p>
          </div>
        </header>

        <nav class="quick-links" aria-label="快速访问">
          <RouterLink v-for="link in quickLinks" :key="link.path" :to="link.path" class="quick-link">
            <span class="quick-link__icon">
              <AppIcon :name="link.icon" :size="17" />
            </span>
            <span class="quick-link__body">
              <strong>{{ link.title }}</strong>
              <span>{{ link.description }}</span>
            </span>
            <AppIcon name="chevron-right" :size="14" class="quick-link__arrow" />
          </RouterLink>
        </nav>

        <RouterLink to="/settings" class="settings-link">
          <AppIcon name="settings" :size="15" />
          外观与应用设置
          <AppIcon name="arrow-right" :size="13" />
        </RouterLink>
      </aside>
    </div>

    <footer class="home-footer">
      <span class="home-footer__status">
        <span aria-hidden="true" />
        所有内容保存在本地
      </span>
      <span>Markdown · SQLite · 离线可用</span>
    </footer>
  </div>
</template>

<style scoped>
.home-page {
  gap: 30px;
}

.home-hero {
  display: flex;
  gap: 32px;
  align-items: flex-end;
  justify-content: space-between;
  padding: 6px 0 4px;
}

.home-hero__copy {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.home-hero__eyebrow {
  margin-bottom: 10px;
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
}

.home-hero h1 {
  font-size: clamp(28px, 3vw, 36px);
  font-weight: 700;
  line-height: 1.25;
  letter-spacing: -0.035em;
}

.home-hero p {
  max-width: 620px;
  margin-top: 12px;
  color: var(--color-text-secondary);
  font-size: 13.5px;
  line-height: 1.75;
}

.home-hero__actions {
  display: flex;
  flex: 0 0 auto;
  gap: 8px;
  padding-bottom: 3px;
}

.home-error {
  display: flex;
  gap: 9px;
  align-items: center;
  padding: 10px 12px;
  color: var(--color-danger);
  font-size: 12px;
  background: var(--color-danger-soft);
  border-radius: var(--radius-sm);
}

.home-error span {
  flex: 1;
  min-width: 0;
}

.home-error button {
  flex: 0 0 auto;
  padding: 0;
  color: inherit;
  font-size: 12px;
  text-decoration: underline;
  text-underline-offset: 3px;
  background: transparent;
  border: 0;
}

.overview {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  padding: 18px 0;
  border-top: 1px solid var(--color-border);
  border-bottom: 1px solid var(--color-border);
}

.overview__item {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 2px 16px;
  align-items: end;
  padding: 0 28px;
  border-right: 1px solid var(--color-border);
  transition: color var(--transition-fast);
}

.overview__item:first-child {
  padding-left: 0;
}

.overview__item:last-child {
  padding-right: 0;
  border-right: 0;
}

.overview__label {
  align-self: center;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
}

.overview__value {
  grid-row: 1 / 3;
  grid-column: 2;
  color: var(--color-text);
  font-size: 28px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  line-height: 1.2;
  letter-spacing: -0.03em;
  transition: color var(--transition-fast);
}

.overview__hint {
  color: var(--color-text-muted);
  font-size: 11px;
}

.overview__item:hover .overview__value {
  color: var(--color-accent);
}

.home-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.7fr) minmax(280px, 0.9fr);
  gap: 48px;
  align-items: start;
}

.home-section {
  min-width: 0;
}

.section-header {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
  min-height: 48px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--color-border);
}

.section-header h2 {
  font-size: 15px;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.section-header p {
  margin-top: 2px;
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.section-link {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 5px;
  align-items: center;
  color: var(--color-text-secondary);
  font-size: 12px;
  transition:
    gap var(--transition-fast),
    color var(--transition-fast);
}

.section-link:hover {
  gap: 8px;
  color: var(--color-accent);
}

.recent-list {
  display: flex;
  flex-direction: column;
}

.recent-note {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  padding: 16px 10px;
  border-bottom: 1px solid var(--color-border);
  border-radius: 0 0 6px 6px;
  transition: background-color var(--transition-fast);
}

.recent-note:hover {
  background: var(--color-surface-hover);
}

.recent-note__icon {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 30px;
  height: 30px;
  margin-top: 1px;
  color: var(--color-text-secondary);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 6px;
}

.recent-note__body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.recent-note__title {
  overflow: hidden;
  font-size: 13.5px;
  font-weight: 600;
  line-height: 1.5;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.recent-note__excerpt {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.55;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.recent-note__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  margin-top: 3px;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 10.5px;
  line-height: 1.5;
}

.recent-note__arrow {
  flex: 0 0 auto;
  margin-top: 8px;
  color: var(--color-text-muted);
  opacity: 0;
  transform: translateX(-4px);
  transition:
    opacity var(--transition-fast),
    transform var(--transition-fast);
}

.recent-note:hover .recent-note__arrow,
.recent-note:focus-visible .recent-note__arrow {
  opacity: 1;
  transform: translateX(0);
}

.quick-section {
  padding-left: 4px;
}

.quick-links {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-top: 8px;
}

.quick-link {
  display: flex;
  gap: 10px;
  align-items: center;
  min-height: 52px;
  padding: 7px 9px;
  border-radius: 6px;
  transition: background-color var(--transition-fast);
}

.quick-link:hover {
  background: var(--color-surface-hover);
}

.quick-link__icon {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 30px;
  height: 30px;
  color: var(--color-text-secondary);
}

.quick-link__body {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
}

.quick-link__body strong {
  font-size: 12.5px;
  font-weight: 500;
}

.quick-link__body span {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 10.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.quick-link__arrow {
  flex: 0 0 auto;
  color: var(--color-text-muted);
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.quick-link:hover .quick-link__arrow,
.quick-link:focus-visible .quick-link__arrow {
  opacity: 1;
}

.settings-link {
  display: flex;
  gap: 7px;
  align-items: center;
  margin: 12px 9px 0;
  padding-top: 14px;
  color: var(--color-text-muted);
  font-size: 11.5px;
  border-top: 1px solid var(--color-border);
  transition: color var(--transition-fast);
}

.settings-link svg:last-child {
  margin-left: auto;
}

.settings-link:hover {
  color: var(--color-text);
}

.recent-empty,
.recent-unavailable {
  display: flex;
  flex-direction: column;
  gap: 7px;
  align-items: flex-start;
  padding: 40px 10px;
  color: var(--color-text-muted);
}

.recent-empty__icon {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  margin-bottom: 4px;
  background: var(--color-surface-hover);
  border-radius: 8px;
}

.recent-empty h3 {
  color: var(--color-text);
  font-size: 14px;
  font-weight: 600;
}

.recent-empty p {
  max-width: 360px;
  margin-bottom: 6px;
  font-size: 12px;
  line-height: 1.7;
}

.recent-unavailable {
  flex-direction: row;
  align-items: center;
  justify-content: center;
  min-height: 180px;
  font-size: 12px;
}

.recent-skeleton {
  display: flex;
  flex-direction: column;
}

.skeleton-row {
  display: flex;
  gap: 12px;
  align-items: center;
  min-height: 91px;
  padding: 16px 10px;
  border-bottom: 1px solid var(--color-border);
  animation: skeleton-pulse 1.4s ease-in-out infinite alternate;
}

.skeleton-lines {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 9px;
}

.skeleton-block {
  display: block;
  background: var(--color-surface-hover);
  border-radius: 4px;
}

.skeleton-block--icon {
  width: 30px;
  height: 30px;
  border-radius: 6px;
}

.skeleton-block--title {
  width: min(46%, 220px);
  height: 12px;
}

.skeleton-block--text {
  width: min(82%, 420px);
  height: 9px;
}

.home-footer {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 20px;
  align-items: center;
  justify-content: space-between;
  padding-top: 16px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  border-top: 1px solid var(--color-border);
}

.home-footer__status {
  display: inline-flex;
  gap: 7px;
  align-items: center;
}

.home-footer__status > span {
  width: 6px;
  height: 6px;
  background: var(--color-success);
  border-radius: 50%;
}

@keyframes skeleton-pulse {
  from {
    opacity: 0.48;
  }
  to {
    opacity: 1;
  }
}

@media (max-width: 980px) {
  .home-grid {
    grid-template-columns: minmax(0, 1fr);
    gap: 36px;
  }

  .quick-section {
    padding-left: 0;
  }

  .quick-links {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 680px) {
  .home-hero {
    flex-direction: column;
    gap: 20px;
    align-items: flex-start;
  }

  .home-hero__actions {
    width: 100%;
  }

  .home-hero__actions > a {
    flex: 1;
  }

  .overview__item {
    padding: 0 14px;
  }

  .overview__value {
    font-size: 23px;
  }

  .overview__hint {
    display: none;
  }

  .home-footer {
    align-items: flex-start;
    flex-direction: column;
  }
}

@media (max-width: 480px) {
  .quick-links {
    grid-template-columns: minmax(0, 1fr);
  }

  .overview__item {
    display: flex;
    flex-direction: column-reverse;
    gap: 5px;
    align-items: flex-start;
  }

  .recent-note__meta {
    display: block;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .recent-note__meta span {
    margin: 0 5px;
  }
}
</style>
