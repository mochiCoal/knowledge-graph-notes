<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import PageHeader from '@/components/common/PageHeader.vue'
import RelationFormDialog from '@/components/relations/RelationFormDialog.vue'
import RelationTypeDialog from '@/components/relations/RelationTypeDialog.vue'
import { useRelationsStore } from '@/stores/relations'
import type { RelationDetail, RelationSuggestion } from '@/types'
import { formatDateTime } from '@/utils/format'

const route = useRoute()
const store = useRelationsStore()

const formOpen = ref(false)
const editingRelation = ref<RelationDetail | null>(null)
const defaultSourceId = ref<string | null>(null)
const typeManagerOpen = ref(false)

const pendingDelete = ref<RelationDetail | null>(null)
const deleting = ref(false)
const suggestionBusyId = ref<string | null>(null)

const actionError = ref<string | null>(null)
const notice = ref<string | null>(null)

const queryCourseId =
  typeof route.query.course === 'string' && route.query.course
    ? route.query.course
    : null
const queryPointId =
  typeof route.query.point === 'string' && route.query.point ? route.query.point : null

const deleteMessage = computed(() => {
  const relation = pendingDelete.value
  if (!relation) return ''

  const arrow = relation.directed ? '→' : '↔'
  return (
    `确定删除「${relation.source_title}」${arrow}「${relation.target_title}」的` +
    `「${relation.relation_label}」关联吗？此操作不可恢复。`
  )
})

const typeFilterLabel = computed(() => {
  if (!store.typeFilter) return ''
  return store.relationLabel(store.typeFilter)
})

const hasCourses = computed(() => store.courses.length > 0)
const hasEnoughPoints = computed(() => store.allPoints.length >= 2)
const activeTypeUsage = computed(
  () => store.types.find((type) => type.id === store.typeFilter)?.usage_count ?? 0,
)

onMounted(() => {
  void store.initialize(queryCourseId, queryPointId)
})

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function clearFeedback(): void {
  actionError.value = null
  notice.value = null
}

function selectCourse(courseId: string): void {
  clearFeedback()
  void store.selectCourse(courseId)
}

function onCourseChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value
  if (value) selectCourse(value)
}

function selectPoint(pointId: string): void {
  clearFeedback()
  void store.selectPoint(pointId)
}

async function openPoint(pointId: string): Promise<void> {
  const point = store.allPoints.find((item) => item.id === pointId)
  if (!point) return

  clearFeedback()

  if (point.course_id !== store.selectedCourseId) {
    await store.selectCourse(point.course_id, point.id)
  } else {
    await store.selectPoint(point.id)
  }
}

function refresh(): void {
  clearFeedback()
  void store.refresh()
}

function onTypeFilterChange(event: Event): void {
  store.setTypeFilter((event.target as HTMLSelectElement).value)
}

function openCreate(): void {
  if (!store.selectedPointId) return

  clearFeedback()
  editingRelation.value = null
  defaultSourceId.value = store.selectedPointId
  formOpen.value = true
}

function openEdit(relation: RelationDetail): void {
  clearFeedback()
  editingRelation.value = relation
  defaultSourceId.value = relation.source_id
  formOpen.value = true
}

function onRelationSaved(relation: RelationDetail): void {
  formOpen.value = false
  editingRelation.value = null
  actionError.value = null
  notice.value = `已保存「${relation.source_title}」与「${relation.target_title}」的${relation.relation_label}关联`
}

function requestDelete(relation: RelationDetail): void {
  clearFeedback()
  pendingDelete.value = relation
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
    await store.deleteRelation(target.id)
    pendingDelete.value = null
    notice.value = `已删除「${target.source_title}」与「${target.target_title}」的关联`
  } catch (error) {
    pendingDelete.value = null
    actionError.value = errorText(error)
  } finally {
    deleting.value = false
  }
}

async function refreshSuggestions(): Promise<void> {
  actionError.value = null
  await store.generateSuggestions()
}

async function acceptSuggestion(suggestion: RelationSuggestion): Promise<void> {
  if (suggestionBusyId.value) return

  suggestionBusyId.value = suggestion.id
  actionError.value = null

  try {
    const relation = await store.acceptSuggestion(suggestion.id)
    notice.value = `已接受建议，创建「${relation.source_title}」与「${relation.target_title}」的${relation.relation_label}关联`
  } catch (error) {
    actionError.value = errorText(error)
  } finally {
    suggestionBusyId.value = null
  }
}

async function rejectSuggestion(suggestion: RelationSuggestion): Promise<void> {
  if (suggestionBusyId.value) return

  suggestionBusyId.value = suggestion.id
  actionError.value = null

  try {
    await store.rejectSuggestion(suggestion.id)
    notice.value = '已忽略该建议，之后不会再推荐同一知识点对'
  } catch (error) {
    actionError.value = errorText(error)
  } finally {
    suggestionBusyId.value = null
  }
}

function onTypeChanged(message: string): void {
  actionError.value = null
  notice.value = message
}

function relationArrow(relation: RelationDetail): string {
  return relation.directed ? '→' : '↔'
}

function scorePercent(score: number): number {
  return Math.max(1, Math.min(100, Math.round(score * 100)))
}
</script>

<template>
  <div class="page relations-page">
    <PageHeader
      title="知识点关联"
      description="为知识点建立前置、相关、包含、对比等语义关联，并依据名称、描述与笔记内容获取可解释的本地推荐。"
    >
      <template #actions>
        <button type="button" class="ghost-button" :disabled="store.isLoading" @click="refresh">
          <AppIcon name="refresh" :size="15" />
          刷新
        </button>
        <button type="button" class="ghost-button" @click="typeManagerOpen = true">
          <AppIcon name="settings" :size="15" />
          关联类型
        </button>
        <button
          type="button"
          class="primary-button"
          :disabled="!store.selectedPointId || !hasEnoughPoints"
          @click="openCreate"
        >
          <AppIcon name="plus" :size="16" />
          新建关联
        </button>
      </template>
    </PageHeader>

    <div
      v-if="actionError || (store.errorMessage && store.initialized)"
      class="alert-bar"
      role="alert"
    >
      <AppIcon name="alert" :size="16" />
      <span>{{ actionError || store.errorMessage }}</span>
      <button
        type="button"
        class="alert-bar__dismiss"
        @click="actionError = null; store.dismissError()"
      >
        关闭
      </button>
    </div>

    <div v-if="notice" class="notice-bar" role="status">
      <AppIcon name="sparkles" :size="15" />
      <span>{{ notice }}</span>
      <button type="button" class="alert-bar__dismiss" @click="notice = null">关闭</button>
    </div>

    <div v-if="store.isLoading && !store.initialized" class="page-state">
      <span class="spinner" />
      <span>正在加载知识点与关联…</span>
    </div>

    <div v-else-if="store.errorMessage && !store.initialized" class="empty-panel relations-empty">
      <div class="empty-panel__icon">
        <AppIcon name="alert" :size="20" />
      </div>
      <h3>关联数据加载失败</h3>
      <p>{{ store.errorMessage }}</p>
      <button type="button" class="primary-button" @click="refresh">重新加载</button>
    </div>

    <div v-else-if="!hasCourses" class="empty-panel relations-empty">
      <div class="empty-panel__icon">
        <AppIcon name="courses" :size="22" />
      </div>
      <h3>还没有课程</h3>
      <p>知识点关联建立在课程的知识点之上，先创建课程和知识点再来建立关联。</p>
      <RouterLink to="/courses" class="primary-button">
        <AppIcon name="courses" :size="16" />
        前往课程管理
      </RouterLink>
    </div>

    <div v-else class="relations-layout">
      <aside class="pane pane--points" aria-label="课程与知识点">
        <header class="pane__head">
          <h2>课程与知识点</h2>
          <span class="count-badge">{{ store.coursePoints.length }}</span>
        </header>

        <div class="pane__body pane__body--points">
          <select
            class="select course-select"
            :value="store.selectedCourseId ?? ''"
            aria-label="选择课程"
            @change="onCourseChange"
          >
            <option v-for="course in store.courses" :key="course.id" :value="course.id">
              {{ course.name }}
            </option>
          </select>

          <div class="search-field">
            <AppIcon name="search" :size="15" class="search-field__icon" />
            <input
              v-model="store.keyword"
              class="search-field__input"
              type="text"
              placeholder="搜索知识点…"
              aria-label="搜索知识点"
            />
            <button
              v-if="store.keyword"
              type="button"
              class="search-field__clear"
              aria-label="清空搜索"
              @click="store.keyword = ''"
            >
              <AppIcon name="x" :size="13" />
            </button>
          </div>

          <ul v-if="store.filteredPoints.length" class="point-list">
            <li v-for="point in store.filteredPoints" :key="point.id">
              <button
                type="button"
                class="point-option"
                :class="{ 'is-active': point.id === store.selectedPointId }"
                :title="point.title"
                @click="selectPoint(point.id)"
              >
                <span class="point-option__title">{{ point.title }}</span>
                <span class="point-option__count">
                  {{ store.relationCountOf(point.id) }}
                </span>
              </button>
            </li>
          </ul>

          <p v-else class="pane-hint">没有匹配的知识点。</p>
        </div>

        <footer class="pane__foot">
          {{ store.coursePoints.length }} 个知识点 · {{ store.totalRelations }} 条关联
        </footer>
      </aside>

      <section class="pane pane--relations" aria-label="当前知识点的关联">
        <header class="pane__head pane__head--wrap">
          <h2 :title="store.selectedPoint?.title">
            {{ store.selectedPoint?.title ?? '知识点关联' }}
          </h2>
          <span class="count-badge">{{ store.selectedPointRelations.length }}</span>

          <div class="pane__head-actions">
            <select
              class="select type-filter"
              :value="store.typeFilter"
              aria-label="按关联类型筛选"
              @change="onTypeFilterChange"
            >
              <option value="">全部类型</option>
              <option v-for="type in store.types" :key="type.id" :value="type.id">
                {{ type.label }}（{{ type.usage_count }}）
              </option>
            </select>
            <button
              type="button"
              class="ghost-button pane__head-button"
              :disabled="!store.selectedPointId || !hasEnoughPoints"
              @click="openCreate"
            >
              <AppIcon name="plus" :size="14" />
              新建
            </button>
          </div>
        </header>

        <div class="pane__body pane__body--relations">
          <div v-if="store.isLoadingRelations" class="pane-state">
            <span class="spinner" />
            <span>正在加载关联…</span>
          </div>

          <div v-else-if="store.relationsError" class="pane-state pane-state--error">
            <AppIcon name="alert" :size="16" />
            <span>{{ store.relationsError }}</span>
            <button type="button" class="ghost-button" @click="refresh">重试</button>
          </div>

          <div v-else-if="!store.selectedPoint" class="empty-panel">
            <div class="empty-panel__icon">
              <AppIcon name="link" :size="20" />
            </div>
            <h3>选择一个知识点</h3>
            <p>在左侧选择知识点，查看它已经建立的语义关联。</p>
          </div>

          <template v-else>
            <ul v-if="store.selectedPointRelations.length" class="relation-list">
              <li
                v-for="relation in store.selectedPointRelations"
                :key="relation.id"
                class="relation-card"
              >
                <div class="relation-card__head">
                  <span
                    class="type-chip"
                    :style="{ color: relation.relation_color, borderColor: relation.relation_color }"
                  >
                    <span
                      class="type-chip__dot"
                      :style="{ background: relation.relation_color }"
                    />
                    {{ relation.relation_label }}
                    <span class="type-chip__direction">
                      {{ relation.directed ? '有向' : '无向' }}
                    </span>
                  </span>
                  <span class="relation-card__time">
                    {{ formatDateTime(relation.created_at) }}
                  </span>
                </div>

                <div class="relation-endpoints">
                  <button
                    type="button"
                    class="endpoint"
                    :class="{ 'is-current': relation.source_id === store.selectedPointId }"
                    :title="`查看「${relation.source_title}」`"
                    @click="openPoint(relation.source_id)"
                  >
                    {{ relation.source_title }}
                  </button>
                  <span class="endpoint__arrow" :title="relation.directed ? '有向关联' : '无向关联'">
                    {{ relationArrow(relation) }}
                  </span>
                  <button
                    type="button"
                    class="endpoint"
                    :class="{ 'is-current': relation.target_id === store.selectedPointId }"
                    :title="`查看「${relation.target_title}」`"
                    @click="openPoint(relation.target_id)"
                  >
                    {{ relation.target_title }}
                  </button>
                </div>

                <p v-if="relation.description" class="relation-card__desc">
                  {{ relation.description }}
                </p>
                <p v-else class="relation-card__desc is-empty">暂无关联描述</p>

                <div class="relation-card__foot">
                  <span class="relation-card__path">
                    {{ relation.source_course_name }} / {{ relation.target_course_name }}
                  </span>
                  <div class="relation-card__actions">
                    <button type="button" class="ghost-button card-button" @click="openEdit(relation)">
                      <AppIcon name="pencil" :size="13" />
                      编辑
                    </button>
                    <button
                      type="button"
                      class="ghost-button card-button card-button--danger"
                      @click="requestDelete(relation)"
                    >
                      <AppIcon name="trash" :size="13" />
                      删除
                    </button>
                  </div>
                </div>
              </li>
            </ul>

            <div v-else class="empty-panel">
              <div class="empty-panel__icon">
                <AppIcon name="link" :size="20" />
              </div>
              <h3>{{ store.typeFilter ? '该类型下没有关联' : '这个知识点还没有关联' }}</h3>
              <p>
                {{
                  store.typeFilter
                    ? '切换到「全部类型」，或为它建立一条新的关联。'
                    : '可以先手动创建关联，也可以查看右侧的推荐建议。'
                }}
              </p>
              <button
                type="button"
                class="primary-button"
                :disabled="!hasEnoughPoints"
                @click="openCreate"
              >
                <AppIcon name="plus" :size="16" />
                新建关联
              </button>
            </div>
          </template>
        </div>

        <footer class="pane__foot pane__foot--split">
          <span>
            共 {{ store.selectedPointRelations.length }} 条
            <template v-if="typeFilterLabel">（类型：{{ typeFilterLabel }}）</template>
          </span>
          <span v-if="activeTypeUsage > 0">该类型共被 {{ activeTypeUsage }} 条关联使用</span>
          <button
            v-else
            type="button"
            class="text-button"
            :disabled="!store.types.length"
            @click="typeManagerOpen = true"
          >
            管理关联类型
            <AppIcon name="arrow-right" :size="13" />
          </button>
        </footer>
      </section>

      <aside class="pane pane--suggestions" aria-label="关联建议">
        <header class="pane__head">
          <h2>关联建议</h2>
          <span class="count-badge">{{ store.suggestions.length }}</span>
          <button
            type="button"
            class="icon-button pane__head-action"
            title="重新生成建议"
            aria-label="重新生成建议"
            :disabled="!store.selectedPointId || store.isLoadingSuggestions"
            @click="refreshSuggestions"
          >
            <AppIcon name="refresh" :size="15" />
          </button>
        </header>

        <div class="pane__body pane__body--suggestions">
          <p class="suggestion-intro">
            <AppIcon name="sparkles" :size="14" />
            根据名称、描述与笔记内容在本地计算，接受后会写入正式关联。
          </p>

          <div v-if="store.isLoadingSuggestions" class="pane-state">
            <span class="spinner" />
            <span>正在分析文本相似度…</span>
          </div>

          <div v-else-if="store.suggestionsError" class="pane-state pane-state--error">
            <AppIcon name="alert" :size="16" />
            <span>{{ store.suggestionsError }}</span>
            <button type="button" class="ghost-button" @click="refreshSuggestions">重试</button>
          </div>

          <div v-else-if="!store.selectedPoint" class="empty-panel">
            <div class="empty-panel__icon">
              <AppIcon name="sparkles" :size="18" />
            </div>
            <h3>选择知识点后生成建议</h3>
            <p>建议只比较同一课程内的知识点，避免跨课程噪声。</p>
          </div>

          <template v-else>
            <ul v-if="store.suggestions.length" class="suggestion-list">
              <li
                v-for="suggestion in store.suggestions"
                :key="suggestion.id"
                class="suggestion-card"
              >
                <div class="suggestion-card__head">
                  <span class="suggestion-card__title" :title="store.suggestionOtherTitle(suggestion)">
                    {{ store.suggestionOtherTitle(suggestion) }}
                  </span>
                  <span class="suggestion-card__score">{{ scorePercent(suggestion.score) }}%</span>
                </div>

                <div class="score-bar" :aria-label="`综合相似度 ${scorePercent(suggestion.score)}%`">
                  <span
                    class="score-bar__fill"
                    :style="{ width: `${scorePercent(suggestion.score)}%` }"
                  />
                </div>

                <ul class="reason-list">
                  <li v-for="reason in suggestion.reasons" :key="reason" class="reason-item">
                    {{ reason }}
                  </li>
                </ul>

                <p class="suggestion-card__meta">
                  建议类型：{{ suggestion.relation_type_label }} · 同课程知识点
                </p>

                <div class="suggestion-card__actions">
                  <button
                    type="button"
                    class="primary-button card-button"
                    :disabled="suggestionBusyId !== null"
                    @click="acceptSuggestion(suggestion)"
                  >
                    {{ suggestionBusyId === suggestion.id ? '处理中…' : '接受' }}
                  </button>
                  <button
                    type="button"
                    class="ghost-button card-button"
                    :disabled="suggestionBusyId !== null"
                    @click="rejectSuggestion(suggestion)"
                  >
                    忽略
                  </button>
                </div>
              </li>
            </ul>

            <div v-else class="empty-panel suggestion-empty">
              <div class="empty-panel__icon">
                <AppIcon name="sparkles" :size="18" />
              </div>
              <h3>暂未发现合适的关联</h3>
              <p>
                补充知识点描述或笔记内容后再试；已接受与已忽略的知识点对不会重复推荐。
              </p>
              <button type="button" class="ghost-button" @click="refreshSuggestions">
                重新生成
              </button>
            </div>
          </template>
        </div>

        <footer class="pane__foot">
          建议是辅助结果，最终关联由你确认；忽略后不再推荐同一知识点对
        </footer>
      </aside>
    </div>

    <RelationFormDialog
      :open="formOpen"
      :relation="editingRelation"
      :default-source-id="defaultSourceId"
      @close="formOpen = false"
      @saved="onRelationSaved"
    />

    <RelationTypeDialog
      :open="typeManagerOpen"
      @close="typeManagerOpen = false"
      @changed="onTypeChanged"
    />

    <ConfirmDialog
      :open="pendingDelete !== null"
      title="删除知识点关联"
      :message="deleteMessage"
      confirm-text="删除关联"
      :busy="deleting"
      @cancel="cancelDelete"
      @confirm="confirmDelete"
    />
  </div>
</template>

<style scoped>
.relations-layout {
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr) 330px;
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
  min-height: 44px;
  padding: 0 10px 0 16px;
  border-bottom: 1px solid var(--color-border);
}

.pane__head--wrap {
  flex-wrap: wrap;
  padding-top: 6px;
  padding-bottom: 6px;
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

.pane__head-actions {
  display: flex;
  flex: 0 0 auto;
  gap: 6px;
  align-items: center;
  margin-left: auto;
}

.pane__head-button {
  height: 28px;
  padding: 0 9px;
  font-size: 11.5px;
}

.pane__head-action {
  flex: 0 0 auto;
  width: 26px;
  height: 26px;
  margin-left: auto;
}

.pane__head-action:disabled {
  cursor: not-allowed;
  opacity: 0.4;
}

.count-badge {
  flex: 0 0 auto;
  padding: 0 7px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  font-weight: 500;
  line-height: 18px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.pane__body {
  padding: 8px;
  overflow-y: auto;
}

.pane__body--points {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: min(62vh, 640px);
}

.pane__body--relations {
  min-height: 240px;
  max-height: min(64vh, 680px);
  padding: 10px;
}

.pane__body--suggestions {
  min-height: 220px;
  max-height: min(64vh, 680px);
  padding: 10px;
}

.pane__foot {
  flex: 0 0 auto;
  padding: 9px 16px;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 10.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
  border-top: 1px solid var(--color-border);
}

.pane__foot--split {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: space-between;
}

.course-select {
  flex: 0 0 auto;
  height: 32px;
  padding: 0 8px;
  font-size: 12px;
}

.type-filter {
  width: auto;
  max-width: 150px;
  height: 28px;
  padding: 0 6px;
  font-size: 11.5px;
}

.search-field {
  display: flex;
  flex: 0 0 auto;
  gap: 7px;
  align-items: center;
  height: 32px;
  padding: 0 8px 0 10px;
  background: var(--color-surface);
  border: 1px solid var(--color-border-strong);
  border-radius: var(--radius-sm);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.search-field:focus-within {
  border-color: var(--color-accent);
  box-shadow: 0 0 0 3px var(--color-accent-soft);
}

.search-field__icon {
  flex: 0 0 auto;
  color: var(--color-text-muted);
}

.search-field__input {
  flex: 1;
  min-width: 0;
  height: 100%;
  padding: 0;
  font-size: 12.5px;
  background: transparent;
  border: none;
  outline: none;
}

.search-field__input::placeholder {
  color: var(--color-text-muted);
}

.search-field__clear {
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
}

.search-field__clear:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.point-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.point-option {
  display: flex;
  gap: 8px;
  align-items: center;
  width: 100%;
  padding: 8px 10px;
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

.point-option:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.point-option.is-active {
  color: var(--color-accent-strong);
  font-weight: 500;
  background: var(--color-accent-soft);
}

.point-option__title {
  flex: 1;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.point-option__count {
  flex: 0 0 auto;
  padding: 0 6px;
  color: var(--color-text-muted);
  font-size: 10px;
  font-weight: 400;
  line-height: 16px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.point-option.is-active .point-option__count {
  color: var(--color-accent-strong);
  background: var(--color-surface);
}

.pane-hint {
  padding: 10px 4px;
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.relation-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.relation-card {
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding: 12px 13px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast);
}

.relation-card:hover {
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-sm);
}

.relation-card__head {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
}

.type-chip {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  padding: 0 8px;
  font-size: 10.5px;
  line-height: 20px;
  background: var(--color-surface);
  border: 1px solid currentcolor;
  border-radius: 999px;
}

.type-chip__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.type-chip__direction {
  color: var(--color-text-muted);
  font-size: 9.5px;
}

.relation-card__time {
  flex: 0 0 auto;
  color: var(--color-text-muted);
  font-size: 10.5px;
}

.relation-endpoints {
  display: flex;
  gap: 8px;
  align-items: center;
  min-width: 0;
}

.endpoint {
  max-width: 45%;
  padding: 2px 6px;
  overflow: hidden;
  color: var(--color-text);
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  text-overflow: ellipsis;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 5px;
}

.endpoint:hover {
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
}

.endpoint.is-current {
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
}

.endpoint__arrow {
  flex: 0 0 auto;
  color: var(--color-text-muted);
  font-size: 13px;
}

.relation-card__desc {
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.65;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.relation-card__desc.is-empty {
  color: var(--color-text-muted);
  font-style: italic;
}

.relation-card__foot {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: space-between;
  padding-top: 2px;
}

.relation-card__path {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 10.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.relation-card__actions {
  display: flex;
  flex: 0 0 auto;
  gap: 4px;
}

.card-button {
  height: 26px;
  padding: 0 8px;
  font-size: 11px;
}

.card-button--danger:hover:not(:disabled) {
  color: var(--color-danger);
  background: var(--color-danger-soft);
  border-color: var(--color-danger);
}

.relation-card__actions .primary-button.card-button {
  height: 28px;
  padding: 0 12px;
}

.suggestion-intro {
  display: flex;
  gap: 6px;
  align-items: flex-start;
  padding: 0 2px 8px;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.6;
}

.suggestion-intro svg {
  flex: 0 0 auto;
  margin-top: 1px;
  color: var(--color-accent);
}

.suggestion-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.suggestion-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 11px 12px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.suggestion-card__head {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: space-between;
}

.suggestion-card__title {
  overflow: hidden;
  font-size: 12.5px;
  font-weight: 600;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.suggestion-card__score {
  flex: 0 0 auto;
  color: var(--color-accent-strong);
  font-size: 11px;
  font-weight: 600;
}

.score-bar {
  height: 3px;
  overflow: hidden;
  background: var(--color-surface-hover);
  border-radius: 999px;
}

.score-bar__fill {
  display: block;
  height: 100%;
  background: var(--color-accent);
  border-radius: 999px;
}

.reason-list {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.reason-item {
  position: relative;
  padding-left: 11px;
  color: var(--color-text-secondary);
  font-size: 11px;
  line-height: 1.6;
}

.reason-item::before {
  position: absolute;
  top: 7px;
  left: 2px;
  width: 3px;
  height: 3px;
  content: '';
  background: var(--color-text-muted);
  border-radius: 50%;
}

.suggestion-card__meta {
  color: var(--color-text-muted);
  font-size: 10px;
}

.suggestion-card__actions {
  display: flex;
  gap: 6px;
  justify-content: flex-end;
  padding-top: 2px;
}

.suggestion-card__actions .card-button {
  height: 27px;
  padding: 0 11px;
}

.suggestion-empty {
  padding: 28px 8px;
}

.suggestion-empty .ghost-button {
  margin-top: 4px;
  height: 28px;
  padding: 0 12px;
  font-size: 11.5px;
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

.text-button {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  padding: 0;
  color: var(--color-accent-strong);
  font-size: 10.5px;
  background: none;
  border: none;
}

.text-button:hover:not(:disabled) {
  text-decoration: underline;
  text-underline-offset: 3px;
}

.relations-empty {
  gap: 8px;
  padding: 56px 18px;
  background: var(--color-surface);
  border: 1px dashed var(--color-border-strong);
  border-radius: var(--radius-lg);
}

.relations-empty .primary-button {
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
  .relations-layout {
    grid-template-columns: 210px minmax(0, 1fr);
  }

  .pane--suggestions {
    grid-column: 1 / -1;
  }

  .pane__body--suggestions {
    max-height: none;
  }

  .suggestion-list {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  }
}

@media (max-width: 760px) {
  .relations-layout {
    grid-template-columns: minmax(0, 1fr);
  }

  .pane--suggestions {
    grid-column: auto;
  }
}
</style>
