<script setup lang="ts">
import AppIcon from '@/components/common/AppIcon.vue'
import PageHeader from '@/components/common/PageHeader.vue'
import type { IconName } from '@/components/common/icons'

interface StatCard {
  label: string
  value: string
  hint: string
  icon: IconName
}

interface EntryCard {
  title: string
  description: string
  path: string
  icon: IconName
}

const stats: StatCard[] = [
  { label: '课程', value: '—', hint: '阶段二接入', icon: 'courses' },
  { label: '知识点', value: '—', hint: '阶段三接入', icon: 'graph' },
  { label: '笔记', value: '—', hint: '阶段四接入', icon: 'notes' },
]

const entries: EntryCard[] = [
  {
    title: '课程管理',
    description: '创建课程，作为知识点与笔记的组织入口。',
    path: '/courses',
    icon: 'courses',
  },
  {
    title: '笔记编辑',
    description: '在知识点下编写 Markdown 笔记，支持实时预览。',
    path: '/notes',
    icon: 'notes',
  },
  {
    title: '知识图谱',
    description: '以交互图谱查看知识点之间的语义关联。',
    path: '/graph',
    icon: 'graph',
  },
  {
    title: '应用设置',
    description: '管理数据备份、恢复与应用偏好。',
    path: '/settings',
    icon: 'settings',
  },
]
</script>

<template>
  <div class="page">
    <PageHeader
      title="首页"
      description="课程、知识点、笔记与知识图谱的统一入口。"
    />

    <section class="stat-grid">
      <article v-for="stat in stats" :key="stat.label" class="stat-card">
        <div class="stat-card__top">
          <span class="stat-card__icon">
            <AppIcon :name="stat.icon" :size="18" />
          </span>
          <span class="stat-card__hint">{{ stat.hint }}</span>
        </div>
        <div class="stat-card__body">
          <span class="stat-card__value">{{ stat.value }}</span>
          <span class="stat-card__label">{{ stat.label }}</span>
        </div>
      </article>
    </section>

    <section class="section-block">
      <div class="section-heading">
        <h2>快速入口</h2>
        <span>主要功能模块</span>
      </div>

      <div class="entry-grid">
        <RouterLink
          v-for="entry in entries"
          :key="entry.path"
          :to="entry.path"
          class="entry-card"
        >
          <span class="entry-card__icon">
            <AppIcon :name="entry.icon" :size="20" />
          </span>
          <span class="entry-card__title">{{ entry.title }}</span>
          <span class="entry-card__description">{{ entry.description }}</span>
          <span class="entry-card__arrow">
            <AppIcon name="arrow-right" :size="16" />
          </span>
        </RouterLink>
      </div>
    </section>

    <section class="notice">
      <strong>笔记工作区已可用</strong>
      <span>
        课程与知识点目录、Markdown 编辑与分屏预览、自动保存、标签以及三档主题切换均已接入；其余模块将在后续阶段完成。
      </span>
    </section>
  </div>
</template>

<style scoped>
.stat-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
  gap: 16px;
}

.stat-card {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 18px 20px 20px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-sm);
}

.stat-card__top {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: space-between;
}

.stat-card__icon {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 34px;
  height: 34px;
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
  border-radius: 10px;
}

.stat-card__hint {
  padding: 2px 8px;
  color: var(--color-text-muted);
  font-size: 11px;
  white-space: nowrap;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.stat-card__body {
  display: flex;
  gap: 8px;
  align-items: baseline;
}

.stat-card__value {
  font-size: 26px;
  font-weight: 600;
  line-height: 1;
  letter-spacing: -0.02em;
}

.stat-card__label {
  color: var(--color-text-secondary);
  font-size: 12.5px;
}

.section-block {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-heading {
  display: flex;
  gap: 12px;
  align-items: baseline;
  justify-content: space-between;
}

.section-heading h2 {
  font-size: 15px;
  font-weight: 600;
}

.section-heading span {
  color: var(--color-text-muted);
  font-size: 12px;
}

.entry-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
  gap: 16px;
}

.entry-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 18px 20px 20px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  transition:
    transform var(--transition-fast),
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.entry-card:hover {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-md);
  transform: translateY(-2px);
}

.entry-card__icon {
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  margin-bottom: 6px;
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
  border-radius: 11px;
}

.entry-card__title {
  font-size: 14.5px;
  font-weight: 600;
}

.entry-card__description {
  color: var(--color-text-secondary);
  font-size: 12.5px;
  line-height: 1.55;
}

.entry-card__arrow {
  position: absolute;
  top: 22px;
  right: 20px;
  color: var(--color-text-muted);
  transition:
    color var(--transition-fast),
    transform var(--transition-fast);
}

.entry-card:hover .entry-card__arrow {
  color: var(--color-accent);
  transform: translateX(2px);
}

.notice {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 16px 20px;
  background: var(--color-accent-soft);
  border-radius: var(--radius-md);
}

.notice strong {
  color: var(--color-accent-strong);
  font-size: 13px;
  font-weight: 600;
}

.notice span {
  color: var(--color-text-secondary);
  font-size: 12.5px;
}
</style>
