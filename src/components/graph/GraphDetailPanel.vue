<script setup lang="ts">
import { computed } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import { useGraphStore } from '@/stores/graph'

const store = useGraphStore()

const selected = computed(() => store.selectedPoint)
const path = computed(() => store.selectedPointPath)
const relations = computed(() => store.selectedRelations)

const depthLabel = computed(() => {
  const point = selected.value
  if (!point) return ''
  return point.depth === 0 ? '顶层知识点' : `第 ${point.depth} 级`
})

function directionLabel(outgoing: boolean, directed: boolean): string {
  if (!directed) return '↔'
  return outgoing ? '→' : '←'
}
</script>

<template>
  <aside class="graph-detail" aria-label="图谱详情">
    <template v-if="selected">
      <header class="graph-detail__head">
        <div class="graph-detail__title">
          <span class="graph-detail__eyebrow">{{ depthLabel }}</span>
          <h2>{{ selected.title }}</h2>
        </div>
        <button
          type="button"
          class="icon-button"
          title="取消选择"
          aria-label="取消选择"
          @click="store.selectPoint(null)"
        >
          <AppIcon name="x" :size="15" />
        </button>
      </header>

      <div class="graph-detail__body">
        <nav class="graph-path" aria-label="知识点路径">
          <template v-for="(item, index) in path" :key="item.id">
            <span v-if="index > 0" class="graph-path__sep" aria-hidden="true">/</span>
            <button
              type="button"
              class="graph-path__item"
              :class="{ 'is-current': item.id === selected.id }"
              @click="store.revealPoint(item.id)"
            >
              {{ item.title }}
            </button>
          </template>
        </nav>

        <div class="graph-stats">
          <div class="graph-stat">
            <strong>{{ selected.child_count }}</strong>
            <span>直接子项</span>
          </div>
          <div class="graph-stat">
            <strong>{{ selected.note_count }}</strong>
            <span>直接笔记</span>
          </div>
          <div class="graph-stat">
            <strong>{{ store.selectedRelationCount }}</strong>
            <span>语义关联</span>
          </div>
          <div class="graph-stat">
            <strong>{{ selected.depth === 0 ? '顶层' : selected.depth }}</strong>
            <span>目录层级</span>
          </div>
        </div>

        <section class="graph-section">
          <h3>知识点描述</h3>
          <p class="graph-description" :class="{ 'is-empty': !selected.description }">
            {{ selected.description || '暂无描述，可到知识点管理中补充。' }}
          </p>
        </section>

        <section class="graph-section">
          <h3>
            关联知识点
            <span class="graph-section__count">{{ relations.length }}</span>
          </h3>

          <ul v-if="relations.length" class="graph-relations">
            <li v-for="item in relations" :key="item.relation.id">
              <button
                type="button"
                class="graph-relation"
                :title="item.relation.description || item.otherPoint.title"
                @click="store.revealPoint(item.otherPoint.id)"
              >
                <span
                  class="graph-relation__dot"
                  :style="{ background: item.relation.relation_color }"
                />
                <span class="graph-relation__body">
                  <span class="graph-relation__title">{{ item.otherPoint.title }}</span>
                  <span class="graph-relation__meta">
                    {{ item.relation.relation_label }}
                    <span class="graph-relation__direction">
                      {{ directionLabel(item.outgoing, item.relation.directed) }}
                    </span>
                  </span>
                </span>
                <AppIcon name="chevron-right" :size="13" class="graph-relation__arrow" />
              </button>
            </li>
          </ul>
          <p v-else class="graph-hint">
            暂无语义关联。可以到知识关联页手动创建，或接受系统推荐的关联建议。
          </p>
        </section>

        <section class="graph-section">
          <h3>快捷操作</h3>
          <div class="graph-actions">
            <RouterLink
              :to="{ name: 'notes', query: { point: selected.id } }"
              class="ghost-button graph-actions__button"
            >
              <AppIcon name="notes" :size="14" />
              在笔记中打开
            </RouterLink>
            <RouterLink
              :to="{
                name: 'knowledge-points',
                params: { courseId: store.activeCourseId ?? undefined },
                query: { point: selected.id },
              }"
              class="ghost-button graph-actions__button"
            >
              <AppIcon name="tree" :size="14" />
              在知识点管理中打开
            </RouterLink>
            <RouterLink
              :to="{ name: 'relations', query: { point: selected.id } }"
              class="ghost-button graph-actions__button"
            >
              <AppIcon name="link" :size="14" />
              管理知识点关联
            </RouterLink>
          </div>
        </section>
      </div>
    </template>

    <template v-else>
      <header class="graph-detail__head">
        <div class="graph-detail__title">
          <span class="graph-detail__eyebrow">当前课程</span>
          <h2>{{ store.activeCourse?.name ?? '图谱概览' }}</h2>
        </div>
      </header>

      <div class="graph-detail__body">
        <p v-if="store.activeCourse?.description" class="graph-description">
          {{ store.activeCourse.description }}
        </p>
        <p v-else class="graph-hint">点击图中的任意知识点节点，查看详情与关联。</p>

        <div class="graph-stats">
          <div class="graph-stat">
            <strong>{{ store.visiblePoints.length }}</strong>
            <span>当前节点</span>
          </div>
          <div class="graph-stat">
            <strong>{{ store.hierarchyEdgeCount }}</strong>
            <span>层级连线</span>
          </div>
          <div class="graph-stat">
            <strong>{{ store.visibleRelations.length }}</strong>
            <span>语义连线</span>
          </div>
          <div class="graph-stat">
            <strong>{{ store.activeTypeIds.length }}</strong>
            <span>启用类型</span>
          </div>
        </div>

        <section class="graph-section">
          <h3>读图说明</h3>
          <ul class="graph-guide">
            <li>
              <span class="graph-guide__line graph-guide__line--hierarchy" />
              虚线表示知识点之间的父子层级，由目录结构自动生成。
            </li>
            <li>
              <span class="graph-guide__dot" />
              实线表示语义关联，颜色对应关联类型，箭头表示有向关系。
            </li>
            <li>
              <AppIcon name="graph" :size="13" />
              滚轮缩放、拖拽空白平移、拖动节点调整位置；「重新布局」可恢复自动布局。
            </li>
            <li>
              <AppIcon name="link" :size="13" />
              选中节点会高亮一跳关联并淡出其余节点；「全部标签」可显示所有关系标签。
            </li>
            <li>
              <AppIcon name="sparkles" :size="13" />
              关联类型与知识点内容来自本地 SQLite，筛选只影响当前视图。
            </li>
          </ul>
        </section>

        <section class="graph-section">
          <h3>关联类型</h3>
          <div v-if="store.types.length" class="graph-type-list">
            <span
              v-for="type in store.types"
              :key="type.id"
              class="graph-type"
              :class="{ 'is-off': !store.isTypeActive(type.id) }"
            >
              <span class="graph-type__dot" :style="{ background: type.color }" />
              {{ type.label }}
              <span class="graph-type__count">
                {{ store.typeUsageOf(type.id) }}
              </span>
            </span>
          </div>
          <p v-else class="graph-hint">还没有关联类型，请先到知识关联页创建。</p>
        </section>

        <RouterLink
          v-if="store.activeCourseId"
          :to="{ name: 'course-detail', params: { courseId: store.activeCourseId } }"
          class="graph-course-link"
        >
          查看课程详情
          <AppIcon name="arrow-right" :size="13" />
        </RouterLink>
      </div>
    </template>
  </aside>
</template>

<style scoped>
.graph-detail {
  display: flex;
  flex: 0 0 326px;
  flex-direction: column;
  width: 326px;
  min-width: 0;
  overflow: hidden;
  background: var(--color-surface);
  border-left: 1px solid var(--color-border);
}

.graph-detail__head {
  display: flex;
  flex: 0 0 auto;
  gap: 10px;
  align-items: flex-start;
  justify-content: space-between;
  padding: 18px 14px 14px 18px;
  border-bottom: 1px solid var(--color-border);
}

.graph-detail__title {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.graph-detail__eyebrow {
  color: var(--color-text-muted);
  font-size: 10px;
  font-weight: 500;
  letter-spacing: 0.06em;
}

.graph-detail__title h2 {
  overflow: hidden;
  color: var(--color-text);
  font-size: 16px;
  font-weight: 600;
  line-height: 1.4;
  overflow-wrap: anywhere;
}

.graph-detail__body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 18px;
  padding: 16px 18px 28px;
  overflow-y: auto;
}

.graph-path {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 4px;
  align-items: center;
  color: var(--color-text-muted);
  font-size: 11px;
}

.graph-path__item {
  max-width: 150px;
  padding: 0;
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 11px;
  white-space: nowrap;
  text-overflow: ellipsis;
  background: none;
  border: none;
}

.graph-path__item:hover {
  color: var(--color-accent-strong);
  text-decoration: underline;
  text-underline-offset: 3px;
}

.graph-path__item.is-current {
  color: var(--color-text);
  font-weight: 500;
}

.graph-path__sep {
  color: var(--color-border-strong);
}

.graph-stats {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 7px;
}

.graph-stat {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 9px 11px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.graph-stat strong {
  overflow: hidden;
  font-size: 16px;
  font-weight: 600;
  line-height: 1.3;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.graph-stat span {
  color: var(--color-text-muted);
  font-size: 10.5px;
}

.graph-section {
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding-top: 14px;
  border-top: 1px solid var(--color-border);
}

.graph-section h3 {
  display: flex;
  gap: 6px;
  align-items: center;
  color: var(--color-text-secondary);
  font-size: 11.5px;
  font-weight: 600;
}

.graph-section__count {
  padding: 0 6px;
  color: var(--color-text-muted);
  font-size: 10px;
  font-weight: 500;
  line-height: 16px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.graph-description {
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.7;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.graph-description.is-empty {
  color: var(--color-text-muted);
}

.graph-hint {
  color: var(--color-text-muted);
  font-size: 11.5px;
  line-height: 1.7;
}

.graph-relations {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.graph-relation {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
  padding: 7px 8px;
  text-align: left;
  background: transparent;
  border: none;
  border-radius: 6px;
  transition: background-color var(--transition-fast);
}

.graph-relation:hover {
  background: var(--color-surface-hover);
}

.graph-relation__dot {
  flex: 0 0 auto;
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.graph-relation__body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.graph-relation__title {
  overflow: hidden;
  color: var(--color-text);
  font-size: 12px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.graph-relation__meta {
  color: var(--color-text-muted);
  font-size: 10px;
}

.graph-relation__direction {
  margin-left: 3px;
  color: var(--color-accent-strong);
}

.graph-relation__arrow {
  flex: 0 0 auto;
  color: var(--color-text-muted);
}

.graph-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.graph-actions__button {
  justify-content: flex-start;
  height: 30px;
  padding: 0 10px;
  font-size: 11.5px;
}

.graph-guide {
  display: flex;
  flex-direction: column;
  gap: 8px;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.65;
}

.graph-guide li {
  display: flex;
  gap: 7px;
  align-items: flex-start;
}

.graph-guide svg {
  flex: 0 0 auto;
  margin-top: 1px;
  color: var(--color-text-muted);
}

.graph-guide__line {
  flex: 0 0 auto;
  width: 18px;
  margin-top: 8px;
  border-top: 1.5px solid var(--color-border-strong);
}

.graph-guide__line--hierarchy {
  border-top-style: dashed;
}

.graph-guide__dot {
  flex: 0 0 auto;
  width: 7px;
  height: 7px;
  margin-top: 5px;
  background: var(--color-accent);
  border-radius: 50%;
}

.graph-type-list {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.graph-type {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  padding: 0 7px;
  color: var(--color-text-secondary);
  font-size: 10.5px;
  line-height: 20px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.graph-type.is-off {
  opacity: 0.42;
}

.graph-type__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.graph-type__count {
  color: var(--color-text-muted);
  font-size: 9.5px;
}

.graph-course-link {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  margin-top: auto;
  padding-top: 8px;
  color: var(--color-accent-strong);
  font-size: 11px;
}

.graph-course-link:hover {
  text-decoration: underline;
  text-underline-offset: 3px;
}

@media (max-width: 1120px) {
  .graph-detail {
    flex-basis: 292px;
    width: 292px;
  }
}
</style>
