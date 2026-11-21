<script setup lang="ts">
import { computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import GraphCanvas from '@/components/graph/GraphCanvas.vue'
import GraphDetailPanel from '@/components/graph/GraphDetailPanel.vue'
import { useGraphStore } from '@/stores/graph'
import type { GraphLayoutMode } from '@/types'

const route = useRoute()
const store = useGraphStore()

const queryCourseId =
  typeof route.query.course === 'string' && route.query.course
    ? route.query.course
    : null

const depthOptions = computed(() => {
  const options: { value: number | null; label: string }[] = [
    { value: null, label: '全部层级' },
  ]

  for (let depth = 0; depth <= store.maxTreeDepth; depth += 1) {
    options.push({
      value: depth,
      label: depth === 0 ? '仅顶层' : `前 ${depth + 1} 层`,
    })
  }

  return options
})

const allTypesActive = computed(
  () => store.types.length > 0 && store.activeTypeIds.length === store.types.length,
)

const courseDescription = computed(
  () =>
    store.activeCourse?.description ||
    '以课程知识点为节点、语义关联为连线，查看知识结构并发现知识点之间的联系。',
)

const layoutOptions: { value: GraphLayoutMode; label: string; icon: 'tree' | 'graph' }[] = [
  { value: 'hierarchy', label: '层次布局', icon: 'tree' },
  { value: 'force', label: '关系网络', icon: 'graph' },
]

onMounted(async () => {
  if (!store.initialized) {
    await store.initialize(queryCourseId)
    return
  }

  if (queryCourseId && queryCourseId !== store.activeCourseId) {
    await store.selectCourse(queryCourseId)
  } else {
    await store.refresh()
  }
})

function onCourseChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value
  if (value) void store.selectCourse(value)
}

function onDepthChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value
  store.setMaxDepth(value === '' ? null : Number(value))
}

function onSelect(pointId: string | null): void {
  store.selectPoint(pointId)
}

// 同一页面内只改变 ?course= 时也要切换课程（路由复用组件，不触发 onMounted）
watch(
  () => route.query.course,
  (value) => {
    const courseId = typeof value === 'string' && value ? value : null
    if (courseId && courseId !== store.activeCourseId) {
      void store.selectCourse(courseId)
    }
  },
)
</script>

<template>
  <div class="graph-page">
    <header class="graph-header">
      <div class="graph-header__main">
        <span class="graph-header__eyebrow">
          <AppIcon name="graph" :size="13" />
          课程知识图谱
        </span>
        <h1>{{ store.activeCourse?.name ?? '知识图谱' }}</h1>
        <p>{{ courseDescription }}</p>
      </div>

      <div class="graph-header__side">
        <div v-if="store.activeCourse" class="graph-header__stats">
          <span class="graph-header__stat">
            <strong>{{ store.visiblePoints.length }}</strong>
            知识点
          </span>
          <span class="graph-header__stat">
            <strong>{{ store.visibleRelations.length }}</strong>
            语义关联
          </span>
          <span class="graph-header__stat">
            <strong>{{ store.hierarchyEdgeCount }}</strong>
            层级连线
          </span>
        </div>

        <button
          type="button"
          class="ghost-button"
          :disabled="store.isLoading || store.isLoadingCourse"
          @click="store.refresh()"
        >
          <AppIcon name="refresh" :size="15" />
          刷新
        </button>
      </div>
    </header>

    <div
      v-if="store.errorMessage && store.initialized"
      class="graph-alert"
      role="alert"
    >
      <AppIcon name="alert" :size="15" />
      <span>{{ store.errorMessage }}</span>
      <button type="button" class="graph-alert__close" @click="store.dismissError()">
        关闭
      </button>
    </div>

    <div v-if="store.isLoading && !store.initialized" class="graph-state">
      <span class="spinner" />
      <span>正在加载知识图谱…</span>
    </div>

    <div
      v-else-if="store.errorMessage && !store.initialized"
      class="graph-state graph-state--error"
    >
      <AppIcon name="alert" :size="20" />
      <span>{{ store.errorMessage }}</span>
      <button type="button" class="primary-button" @click="store.refresh()">重新加载</button>
    </div>

    <div v-else-if="!store.courses.length" class="graph-empty">
      <div class="graph-empty__icon">
        <AppIcon name="courses" :size="24" />
      </div>
      <h2>还没有课程</h2>
      <p>知识图谱以课程为单位组织知识点，先创建课程并补充知识点。</p>
      <RouterLink to="/courses" class="primary-button">
        <AppIcon name="courses" :size="16" />
        前往课程管理
      </RouterLink>
    </div>

    <template v-else>
      <section class="graph-toolbar" aria-label="图谱筛选与布局">
        <label class="toolbar-field">
          <span class="toolbar-field__label">课程</span>
          <select
            class="select toolbar-field__select"
            :value="store.activeCourseId ?? ''"
            @change="onCourseChange"
          >
            <option v-for="course in store.courses" :key="course.id" :value="course.id">
              {{ course.name }}
            </option>
          </select>
        </label>

        <div class="graph-segmented" role="group" aria-label="布局方式">
          <button
            v-for="option in layoutOptions"
            :key="option.value"
            type="button"
            class="graph-segmented__item"
            :class="{ 'is-active': store.layout === option.value }"
            :aria-pressed="store.layout === option.value"
            @click="store.setLayout(option.value)"
          >
            <AppIcon :name="option.icon" :size="13" />
            {{ option.label }}
          </button>
        </div>

        <label class="toolbar-field">
          <span class="toolbar-field__label">层级</span>
          <select
            class="select toolbar-field__select toolbar-field__select--depth"
            :value="store.maxDepth ?? ''"
            @change="onDepthChange"
          >
            <option
              v-for="option in depthOptions"
              :key="option.value ?? 'all'"
              :value="option.value ?? ''"
            >
              {{ option.label }}
            </option>
          </select>
        </label>

        <div class="type-filter" role="group" aria-label="按关联类型筛选">
          <button
            type="button"
            class="type-chip type-chip--all"
            :class="{ 'is-active': allTypesActive }"
            :aria-pressed="allTypesActive"
            @click="store.selectAllTypes()"
          >
            全部
          </button>
          <button
            v-for="type in store.types"
            :key="type.id"
            type="button"
            class="type-chip"
            :class="{ 'is-active': store.isTypeActive(type.id) }"
            :style="{ '--type-color': type.color }"
            :aria-pressed="store.isTypeActive(type.id)"
            :title="`${type.directed ? '有向' : '无向'}关联：${type.label}`"
            @click="store.toggleType(type.id)"
          >
            <span class="type-chip__dot" />
            {{ type.label }}
            <span class="type-chip__count">{{ store.typeUsageOf(type.id) }}</span>
          </button>
          <span v-if="!store.types.length" class="type-filter__empty">
            暂无关联类型
          </span>
        </div>

        <label class="toolbar-toggle">
          <input
            type="checkbox"
            :checked="store.showHierarchyEdges"
            @change="store.setShowHierarchyEdges(($event.target as HTMLInputElement).checked)"
          />
          层级连线
        </label>

        <label
          class="toolbar-toggle"
          title="关闭时只显示选中节点的一跳关系标签，勾选后显示全部标签"
        >
          <input
            type="checkbox"
            :checked="store.showEdgeLabels"
            @change="store.setShowEdgeLabels(($event.target as HTMLInputElement).checked)"
          />
          全部标签
        </label>
      </section>

      <div class="graph-body">
        <div class="graph-stage">
          <GraphCanvas
            :points="store.visiblePoints"
            :relations="store.visibleRelations"
            :layout="store.layout"
            :show-hierarchy-edges="store.showHierarchyEdges"
            :show-edge-labels="store.showEdgeLabels"
            :selected-id="store.selectedPointId"
            @select="onSelect"
          />

          <div v-if="store.isLoadingCourse" class="graph-stage__state">
            <span class="spinner" />
            <span>正在加载课程图谱…</span>
          </div>

          <div v-else-if="!store.visiblePoints.length" class="graph-stage__empty">
            <div class="graph-stage__empty-icon">
              <AppIcon name="tree" :size="22" />
            </div>
            <h3>这门课程还没有知识点</h3>
            <p>知识点管理支持多级目录；创建知识点后即可在这里查看结构。</p>
            <RouterLink
              :to="{ name: 'knowledge-points', params: { courseId: store.activeCourseId ?? undefined } }"
              class="primary-button"
            >
              <AppIcon name="plus" :size="16" />
              去创建知识点
            </RouterLink>
          </div>

          <div
            v-else-if="!store.activeTypeIds.length && store.types.length"
            class="graph-stage__notice"
          >
            当前已关闭全部语义关联类型，仅显示层级结构。
            <button type="button" @click="store.selectAllTypes()">恢复全部</button>
          </div>
        </div>

        <GraphDetailPanel />
      </div>
    </template>
  </div>
</template>

<style scoped>
.graph-page {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--color-bg);
}

.graph-header {
  display: flex;
  flex: 0 0 auto;
  gap: 24px;
  align-items: flex-end;
  justify-content: space-between;
  padding: 16px 22px 13px;
}

.graph-header__main {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.graph-header__eyebrow {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  color: var(--color-accent-strong);
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.05em;
}

.graph-header h1 {
  overflow: hidden;
  font-size: 22px;
  font-weight: 650;
  line-height: 1.3;
  letter-spacing: -0.02em;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.graph-header p {
  max-width: 720px;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 11.5px;
  line-height: 1.55;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.graph-header__side {
  display: flex;
  flex: 0 0 auto;
  gap: 14px;
  align-items: center;
}

.graph-header__stats {
  display: flex;
  gap: 15px;
  align-items: center;
}

.graph-header__stat {
  display: inline-flex;
  gap: 4px;
  align-items: baseline;
  color: var(--color-text-muted);
  font-size: 10.5px;
  white-space: nowrap;
}

.graph-header__stat strong {
  color: var(--color-text);
  font-size: 14px;
  font-weight: 600;
}

.graph-alert {
  display: flex;
  flex: 0 0 auto;
  gap: 7px;
  align-items: center;
  margin: 0 18px 8px;
  padding: 8px 12px;
  color: var(--color-danger);
  font-size: 12px;
  background: var(--color-danger-soft);
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-sm);
}

.graph-alert__close {
  margin-left: auto;
  padding: 0;
  color: inherit;
  font-size: 11px;
  background: none;
  border: none;
  text-decoration: underline;
  text-underline-offset: 3px;
}

.graph-state {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 10px;
  align-items: center;
  justify-content: center;
  color: var(--color-text-muted);
  font-size: 12.5px;
}

.graph-state--error {
  color: var(--color-danger);
}

.graph-empty {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 7px;
  align-items: center;
  justify-content: center;
  padding: 32px;
  margin: 0 18px 18px;
  text-align: center;
  background: var(--color-surface);
  border: 1px dashed var(--color-border-strong);
  border-radius: var(--radius-lg);
}

.graph-empty__icon {
  display: grid;
  place-items: center;
  width: 48px;
  height: 48px;
  margin-bottom: 4px;
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
  border-radius: 14px;
}

.graph-empty h2 {
  font-size: 16px;
  font-weight: 600;
}

.graph-empty p {
  max-width: 380px;
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1.65;
}

.graph-empty .primary-button {
  margin-top: 6px;
}

.graph-toolbar {
  display: flex;
  flex: 0 0 auto;
  flex-wrap: wrap;
  gap: 8px 12px;
  align-items: center;
  padding: 7px 18px;
  margin: 0 18px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-bottom: none;
  border-radius: var(--radius-md) var(--radius-md) 0 0;
}

.toolbar-field {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 5px;
  align-items: center;
}

.toolbar-field__label {
  color: var(--color-text-muted);
  font-size: 10px;
  white-space: nowrap;
}

.toolbar-field__select {
  width: 142px;
  height: 29px;
  padding: 0 6px;
  font-size: 11.5px;
}

.toolbar-field__select--depth {
  width: 105px;
}

.graph-segmented {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 1px;
  padding: 2px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 7px;
}

.graph-segmented__item {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  height: 25px;
  padding: 0 8px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  background: transparent;
  border: none;
  border-radius: 5px;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.graph-segmented__item:hover {
  color: var(--color-text);
}

.graph-segmented__item.is-active {
  color: var(--color-text);
  background: var(--color-surface);
  box-shadow: var(--shadow-sm);
}

.type-filter {
  display: flex;
  flex: 1;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
  min-width: 220px;
  padding: 1px 0;
}

.type-filter__empty {
  color: var(--color-text-muted);
  font-size: 10.5px;
  white-space: nowrap;
}

.type-chip {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 4px;
  align-items: center;
  height: 24px;
  padding: 0 7px;
  color: var(--color-text-muted);
  font-size: 10px;
  white-space: nowrap;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 999px;
  opacity: 0.55;
  transition:
    opacity var(--transition-fast),
    border-color var(--transition-fast),
    color var(--transition-fast);
}

.type-chip:hover {
  opacity: 0.85;
}

.type-chip.is-active {
  color: var(--color-text-secondary);
  border-color: var(--type-color, var(--color-accent));
  opacity: 1;
}

.type-chip--all {
  padding: 0 9px;
}

.type-chip__dot {
  width: 6px;
  height: 6px;
  background: var(--type-color, var(--color-accent));
  border-radius: 50%;
}

.type-chip__count {
  color: var(--color-text-muted);
  font-size: 9px;
}

.toolbar-toggle {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 4px;
  align-items: center;
  color: var(--color-text-muted);
  font-size: 10.5px;
  white-space: nowrap;
  cursor: pointer;
}

.toolbar-toggle input {
  width: 13px;
  height: 13px;
  accent-color: var(--color-accent);
}

.graph-body {
  display: flex;
  flex: 1;
  min-height: 0;
  margin: 0 18px 18px;
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 0 0 var(--radius-md) var(--radius-md);
  box-shadow: var(--shadow-sm);
}

.graph-stage {
  position: relative;
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.graph-stage__state,
.graph-stage__empty {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  gap: 9px;
  align-items: center;
  justify-content: center;
  padding: 24px;
  color: var(--color-text-muted);
  font-size: 12px;
  text-align: center;
  background: color-mix(in srgb, var(--color-bg) 82%, transparent);
}

.graph-stage__empty-icon {
  display: grid;
  place-items: center;
  width: 46px;
  height: 46px;
  margin-bottom: 2px;
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
  border-radius: 13px;
}

.graph-stage__empty h3 {
  color: var(--color-text);
  font-size: 14px;
  font-weight: 600;
}

.graph-stage__empty p {
  max-width: 360px;
  line-height: 1.65;
}

.graph-stage__empty .primary-button {
  margin-top: 4px;
}

.graph-stage__notice {
  position: absolute;
  bottom: 54px;
  left: 50%;
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 7px 12px;
  color: var(--color-text-secondary);
  font-size: 11px;
  white-space: nowrap;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 999px;
  box-shadow: var(--shadow-sm);
  transform: translateX(-50%);
}

.graph-stage__notice button {
  padding: 0;
  color: var(--color-accent-strong);
  font-size: 11px;
  background: none;
  border: none;
  text-decoration: underline;
  text-underline-offset: 3px;
}

@media (max-width: 1120px) {
  .graph-header__stats {
    display: none;
  }

  .toolbar-field__select {
    width: 124px;
  }
}

@media (max-width: 900px) {
  .graph-header {
    padding: 12px 14px 10px;
  }

  .graph-header p {
    display: none;
  }

  .graph-toolbar {
    margin: 0 12px;
  }

  .graph-body {
    margin: 0 12px 12px;
  }
}
</style>
