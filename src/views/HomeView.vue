<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import type { IconName } from '@/components/common/icons'
import { dashboardApi } from '@/services/dashboard'
import { useAppStore } from '@/stores/app'
import type {
  DashboardOverview,
  DashboardStats,
  RecentKnowledgePoint,
  RecentNote,
} from '@/types'
import { formatDateTime, formatNumber } from '@/utils/format'

interface QuickLink {
  title: string
  description: string
  path: string
  icon: IconName
}

interface OverviewStat {
  label: string
  value: string
  hint: string
  path: string
  icon: IconName
}

const appStore = useAppStore()

const overview = ref<DashboardOverview | null>(null)
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

const stats = computed<OverviewStat[]>(() => {
  const data = overview.value
  const value = (key: keyof DashboardStats): string =>
    data ? formatNumber(data[key]) : '—'

  return [
    {
      label: '课程',
      value: value('course_count'),
      hint: '学习主题',
      path: '/courses',
      icon: 'courses',
    },
    {
      label: '知识点',
      value: value('knowledge_point_count'),
      hint: '结构化内容',
      path: '/knowledge-points',
      icon: 'tree',
    },
    {
      label: '笔记',
      value: value('note_count'),
      hint: '持续积累',
      path: '/note-library',
      icon: 'notes',
    },
    {
      label: '标签',
      value: value('tag_count'),
      hint: '交叉连接',
      path: '/tags',
      icon: 'tag',
    },
    {
      label: '关联',
      value: value('relation_count'),
      hint: '语义网络',
      path: '/relations',
      icon: 'link',
    },
  ]
})

const recentNotes = computed<RecentNote[]>(() => overview.value?.recent_notes ?? [])
const recentPoints = computed<RecentKnowledgePoint[]>(
  () => overview.value?.recent_points ?? [],
)

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function noteLink(note: RecentNote): RouteLocationRaw {
  return { name: 'notes', query: { point: note.knowledge_point_id } }
}

function pointLink(point: RecentKnowledgePoint): RouteLocationRaw {
  if (point.course_id) {
    return {
      name: 'knowledge-points',
      params: { courseId: point.course_id },
      query: { point: point.id },
    }
  }

  return { name: 'knowledge-points', query: { point: point.id } }
}

function pointPath(point: RecentKnowledgePoint): string {
  return [point.course_name, ...point.path].join(' / ')
}

async function loadOverview(): Promise<void> {
  isLoading.value = true
  errorMessage.value = ''

  try {
    overview.value = await dashboardApi.overview()
  } catch (error) {
    errorMessage.value = errorText(error)
    overview.value = null
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  void loadOverview()
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
        <button type="button" class="ghost-button" @click="appStore.openSearch()">
          <AppIcon name="search" :size="15" />
          搜索知识库
          <kbd class="home-hero__kbd">Ctrl K</kbd>
        </button>
      </div>
    </header>

    <div v-if="errorMessage" class="home-error" role="alert">
      <AppIcon name="alert" :size="16" />
      <span>知识库数据加载失败：{{ errorMessage }}</span>
      <button type="button" @click="loadOverview">重新加载</button>
    </div>

    <section class="overview" aria-label="知识库概览" :aria-busy="isLoading">
      <RouterLink v-for="stat in stats" :key="stat.label" :to="stat.path" class="overview__item">
        <span class="overview__top">
          <AppIcon :name="stat.icon" :size="14" />
          <span class="overview__label">{{ stat.label }}</span>
        </span>
        <strong class="overview__value">{{ stat.value }}</strong>
        <span class="overview__hint">{{ stat.hint }}</span>
      </RouterLink>
    </section>

    <div class="home-grid">
      <div class="home-column">
        <section class="home-section" aria-labelledby="recent-notes-title">
          <header class="section-header">
            <div>
              <h2 id="recent-notes-title">最近编辑的笔记</h2>
              <p>从上次的思路继续</p>
            </div>
            <RouterLink to="/note-library" class="section-link">
              全部笔记
              <AppIcon name="arrow-right" :size="14" />
            </RouterLink>
          </header>

          <div
            v-if="isLoading"
            class="recent-skeleton"
            role="status"
            aria-label="正在加载最近笔记"
          >
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
              v-for="note in recentNotes"
              :key="note.id"
              :to="noteLink(note)"
              class="recent-item"
            >
              <span class="recent-item__icon">
                <AppIcon name="notes" :size="17" />
              </span>
              <span class="recent-item__body">
                <strong class="recent-item__title">{{ note.title }}</strong>
                <span class="recent-item__excerpt">
                  {{ note.excerpt || '这页笔记还没有正文内容。' }}
                </span>
                <span class="recent-item__meta">
                  {{ note.course_name }} / {{ note.knowledge_point_title }}
                  <span aria-hidden="true">·</span>
                  {{ formatDateTime(note.updated_at) }}
                  <span aria-hidden="true">·</span>
                  {{ formatNumber(note.character_count) }} 字
                </span>
              </span>
              <AppIcon name="chevron-right" :size="15" class="recent-item__arrow" />
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

        <section class="home-section" aria-labelledby="recent-points-title">
          <header class="section-header">
            <div>
              <h2 id="recent-points-title">最近创建的知识点</h2>
              <p>刚刚加入知识库的结构化内容</p>
            </div>
            <RouterLink to="/knowledge-points" class="section-link">
              知识点管理
              <AppIcon name="arrow-right" :size="14" />
            </RouterLink>
          </header>

          <div
            v-if="isLoading"
            class="recent-skeleton"
            role="status"
            aria-label="正在加载最近知识点"
          >
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
            <span>暂时无法读取最近知识点</span>
          </div>

          <div v-else-if="recentPoints.length" class="recent-list">
            <RouterLink
              v-for="point in recentPoints"
              :key="point.id"
              :to="pointLink(point)"
              class="recent-item"
            >
              <span class="recent-item__icon">
                <AppIcon name="tree" :size="17" />
              </span>
              <span class="recent-item__body">
                <strong class="recent-item__title">{{ point.title }}</strong>
                <span class="recent-item__excerpt">
                  {{ point.excerpt || '还没有填写描述。' }}
                </span>
                <span class="recent-item__meta">
                  {{ pointPath(point) }}
                  <span aria-hidden="true">·</span>
                  {{ formatDateTime(point.created_at) }} 创建
                  <span v-if="point.note_count > 0" aria-hidden="true">·</span>
                  <span v-if="point.note_count > 0">{{ point.note_count }} 篇笔记</span>
                </span>
              </span>
              <AppIcon name="chevron-right" :size="15" class="recent-item__arrow" />
            </RouterLink>
          </div>

          <div v-else class="recent-empty">
            <span class="recent-empty__icon">
              <AppIcon name="tree" :size="22" />
            </span>
            <h3>先建立课程的知识结构</h3>
            <p>把课程拆分成多级知识点，笔记和关联都会围绕它们展开。</p>
            <RouterLink to="/knowledge-points" class="section-link">
              管理知识点
              <AppIcon name="arrow-right" :size="14" />
            </RouterLink>
          </div>
        </section>
      </div>

      <aside class="home-aside">
        <section class="aside-card" aria-labelledby="search-title">
          <header class="aside-card__head">
            <div>
              <h2 id="search-title">全局搜索</h2>
              <p>跨模块查找内容</p>
            </div>
            <kbd class="aside-card__kbd">Ctrl K</kbd>
          </header>

          <button type="button" class="home-search" @click="appStore.openSearch()">
            <AppIcon name="search" :size="15" />
            <span>搜索课程、知识点、笔记…</span>
            <AppIcon name="arrow-right" :size="14" class="home-search__arrow" />
          </button>
          <p class="aside-card__hint">
            支持课程、知识点、笔记、标签与知识点关联，结果可直接跳转到对应页面。
          </p>
        </section>

        <section class="aside-card" aria-labelledby="quick-title">
          <header class="aside-card__head">
            <div>
              <h2 id="quick-title">快速访问</h2>
              <p>常用工具与视图</p>
            </div>
          </header>

          <nav class="quick-links" aria-label="快速访问">
            <RouterLink
              v-for="link in quickLinks"
              :key="link.path"
              :to="link.path"
              class="quick-link"
            >
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
        </section>

        <section class="aside-card aside-card--data" aria-labelledby="data-title">
          <header class="aside-card__head">
            <div>
              <h2 id="data-title">数据安全</h2>
              <p>本地 SQLite · 离线可用</p>
            </div>
            <span class="aside-card__icon">
              <AppIcon name="database" :size="16" />
            </span>
          </header>
          <p class="aside-card__hint">
            所有内容只保存在本机。可以随时在设置中备份数据库，或从备份文件恢复。
          </p>
          <RouterLink to="/settings" class="aside-card__link">
            前往数据与备份
            <AppIcon name="arrow-right" :size="13" />
          </RouterLink>
        </section>
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
  gap: 26px;
}

.home-hero {
  display: flex;
  gap: 32px;
  align-items: flex-end;
  justify-content: space-between;
  padding: 6px 0 2px;
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

.home-hero__kbd {
  margin-left: 2px;
  padding: 0 4px;
  color: var(--color-text-muted);
  font-family: var(--font-sans);
  font-size: 9.5px;
  line-height: 16px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 4px;
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
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 8px;
}

.overview__item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 13px 15px 12px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  transition:
    border-color var(--transition-fast),
    background-color var(--transition-fast);
}

.overview__item:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-strong);
}

.overview__top {
  display: flex;
  gap: 6px;
  align-items: center;
  color: var(--color-text-muted);
}

.overview__label {
  font-size: 11.5px;
  font-weight: 500;
}

.overview__value {
  color: var(--color-text);
  font-size: 26px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  line-height: 1.25;
  letter-spacing: -0.03em;
  transition: color var(--transition-fast);
}

.overview__item:hover .overview__value {
  color: var(--color-accent);
}

.overview__hint {
  color: var(--color-text-muted);
  font-size: 10.5px;
}

.home-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.65fr) minmax(280px, 0.9fr);
  gap: 40px;
  align-items: start;
}

.home-column {
  display: flex;
  flex-direction: column;
  gap: 34px;
  min-width: 0;
}

.home-section {
  min-width: 0;
}

.home-aside {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.section-header {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
  min-height: 48px;
  padding-bottom: 10px;
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

.recent-item {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  padding: 13px 10px;
  border-bottom: 1px solid var(--color-border);
  border-radius: 0 0 6px 6px;
  transition: background-color var(--transition-fast);
}

.recent-item:hover {
  background: var(--color-surface-hover);
}

.recent-item__icon {
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

.recent-item__body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.recent-item__title {
  overflow: hidden;
  font-size: 13.5px;
  font-weight: 600;
  line-height: 1.5;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.recent-item__excerpt {
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.55;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.recent-item__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  margin-top: 2px;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 10.5px;
  line-height: 1.5;
}

.recent-item__arrow {
  flex: 0 0 auto;
  margin-top: 8px;
  color: var(--color-text-muted);
  opacity: 0;
  transform: translateX(-4px);
  transition:
    opacity var(--transition-fast),
    transform var(--transition-fast);
}

.recent-item:hover .recent-item__arrow,
.recent-item:focus-visible .recent-item__arrow {
  opacity: 1;
  transform: translateX(0);
}

.aside-card {
  padding: 14px 16px 15px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
}

.aside-card__head {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  justify-content: space-between;
}

.aside-card__head h2 {
  font-size: 13.5px;
  font-weight: 600;
}

.aside-card__head p {
  margin-top: 1px;
  color: var(--color-text-muted);
  font-size: 11px;
}

.aside-card__kbd {
  padding: 0 5px;
  color: var(--color-text-muted);
  font-family: var(--font-sans);
  font-size: 9.5px;
  line-height: 18px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 4px;
}

.aside-card__icon {
  display: grid;
  place-items: center;
  width: 26px;
  height: 26px;
  color: var(--color-text-muted);
  background: var(--color-surface-soft);
  border-radius: 6px;
}

.aside-card__hint {
  margin-top: 10px;
  color: var(--color-text-muted);
  font-size: 11.5px;
  line-height: 1.65;
}

.aside-card__link {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  margin-top: 10px;
  color: var(--color-text-secondary);
  font-size: 11.5px;
  transition:
    gap var(--transition-fast),
    color var(--transition-fast);
}

.aside-card__link:hover {
  gap: 8px;
  color: var(--color-accent);
}

.aside-card--data {
  background: var(--color-surface-soft);
}

.home-search {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
  height: 34px;
  margin-top: 11px;
  padding: 0 10px;
  color: var(--color-text-muted);
  font-size: 12px;
  text-align: left;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  transition:
    color var(--transition-fast),
    border-color var(--transition-fast),
    background-color var(--transition-fast);
}

.home-search:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
  border-color: var(--color-border-strong);
}

.home-search span {
  flex: 1;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.home-search__arrow {
  flex: 0 0 auto;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.home-search:hover .home-search__arrow {
  opacity: 1;
}

.quick-links {
  display: flex;
  flex-direction: column;
  gap: 1px;
  margin-top: 6px;
}

.quick-link {
  display: flex;
  gap: 10px;
  align-items: center;
  min-height: 42px;
  padding: 5px 6px;
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
  width: 28px;
  height: 28px;
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

.recent-empty,
.recent-unavailable {
  display: flex;
  flex-direction: column;
  gap: 7px;
  align-items: flex-start;
  padding: 32px 10px;
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
  min-height: 140px;
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
  min-height: 78px;
  padding: 13px 10px;
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

@media (max-width: 1080px) {
  .overview {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

@media (max-width: 980px) {
  .home-grid {
    grid-template-columns: minmax(0, 1fr);
    gap: 32px;
  }

  .home-aside {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    align-items: start;
  }

  .aside-card--data {
    grid-column: span 2;
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

  .home-hero__actions > a,
  .home-hero__actions > button {
    flex: 1;
  }

  .overview {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .overview__value {
    font-size: 23px;
  }

  .home-aside {
    grid-template-columns: minmax(0, 1fr);
  }

  .aside-card--data {
    grid-column: auto;
  }

  .home-footer {
    align-items: flex-start;
    flex-direction: column;
  }
}

@media (max-width: 480px) {
  .home-hero__kbd {
    display: none;
  }

  .recent-item__meta {
    display: block;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .recent-item__meta span {
    margin: 0 5px;
  }
}
</style>
