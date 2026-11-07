<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import { useKnowledgePointsStore } from '@/stores/knowledgePoints'
import type { Course } from '@/types'
import { formatDate } from '@/utils/format'

const props = withDefaults(
  defineProps<{
    /** 层级调整进行中 */
    hierarchyBusy?: boolean
    /** 课程迁移进行中 */
    courseBusy?: boolean
  }>(),
  {
    hierarchyBusy: false,
    courseBusy: false,
  },
)

const emit = defineEmits<{
  edit: []
  delete: []
  addChild: []
  openNote: [pointId: string]
  openRelations: [pointId: string]
  indent: []
  outdent: []
  moveCourse: [courseId: string]
  retry: []
}>()

const store = useKnowledgePointsStore()

const targetCourseId = ref('')

const detail = computed(() => store.detail)
const point = computed(() => detail.value?.point ?? null)

const canIndent = computed(() => {
  const current = point.value
  return current ? store.previousSibling(current.id) !== null : false
})

const canOutdent = computed(() => {
  const current = point.value
  return current ? store.parentOf(current.id) !== null : false
})

const indentHint = computed(() =>
  canIndent.value
    ? '成为同级中上一个知识点的子知识点'
    : '同级中前面没有知识点，无法再降为子级',
)

const outdentHint = computed(() =>
  canOutdent.value ? '提升一级，与父知识点同级' : '已经是顶层知识点',
)

/** 可迁移的目标课程（排除当前所属课程） */
const otherCourses = computed<Course[]>(() =>
  store.courses.filter((course) => course.id !== point.value?.course_id),
)

const canMoveCourse = computed(
  () => targetCourseId.value !== '' && !props.courseBusy && point.value !== null,
)

watch(
  () => point.value?.id,
  () => {
    targetCourseId.value = ''
  },
)

function submitMoveCourse(): void {
  if (!canMoveCourse.value) return
  emit('moveCourse', targetCourseId.value)
}
</script>

<template>
  <div class="detail-panel">
    <div v-if="store.isLoadingDetail" class="detail-state">
      <span class="spinner" />
      <span>正在加载知识点详情…</span>
    </div>

    <div v-else-if="store.detailError" class="detail-state detail-state--error">
      <AppIcon name="alert" :size="18" />
      <span>{{ store.detailError }}</span>
      <button type="button" class="ghost-button" @click="emit('retry')">重试</button>
    </div>

    <div v-else-if="!detail || !point" class="empty-panel detail-empty">
      <div class="empty-panel__icon">
        <AppIcon name="tree" :size="20" />
      </div>
      <h3>选择一个知识点</h3>
      <p>在左侧知识点树中点击任意节点，这里会显示它的详情与调整入口。</p>
    </div>

    <template v-else>
      <header class="detail-head">
        <div class="detail-head__titles">
          <h3>{{ point.title }}</h3>
          <span class="level-badge">
            {{ detail.depth === 0 ? '顶层知识点' : `第 ${detail.depth} 级` }}
          </span>
        </div>

        <div class="detail-head__actions">
          <button type="button" class="icon-button" title="编辑知识点" @click="emit('edit')">
            <AppIcon name="pencil" :size="15" />
          </button>
          <button
            type="button"
            class="icon-button icon-button--danger"
            title="删除知识点"
            @click="emit('delete')"
          >
            <AppIcon name="trash" :size="15" />
          </button>
        </div>
      </header>

      <nav class="breadcrumb" aria-label="知识点路径">
        <template v-for="(item, index) in detail.path" :key="item.id">
          <span v-if="index > 0" class="breadcrumb__sep" aria-hidden="true">/</span>
          <button
            v-if="item.id !== point.id"
            type="button"
            class="breadcrumb__link"
            @click="store.selectPoint(item.id)"
          >
            {{ item.title }}
          </button>
          <span v-else class="breadcrumb__current">{{ item.title }}</span>
        </template>
      </nav>

      <div class="detail-stats">
        <div class="detail-stat">
          <strong>{{ detail.child_count }}</strong>
          <span>直接子项</span>
        </div>
        <div class="detail-stat">
          <strong>{{ detail.descendant_count }}</strong>
          <span>全部后代</span>
        </div>
        <div class="detail-stat">
          <strong>{{ detail.note_count }}</strong>
          <span>直接笔记</span>
        </div>
        <div class="detail-stat">
          <strong>{{ detail.relation_count }}</strong>
          <span>语义关联</span>
        </div>
      </div>

      <section class="detail-section">
        <h4>知识点描述</h4>
        <p class="detail-description" :class="{ 'is-empty': !point.description }">
          {{ point.description || '暂无描述，点击右上角编辑补充。' }}
        </p>
      </section>

      <section class="detail-section">
        <h4>
          层级调整
          <span class="detail-section__hint">通过调整父知识点改变层级</span>
        </h4>
        <div class="button-row">
          <button
            type="button"
            class="ghost-button button-row__item"
            :title="indentHint"
            :disabled="!canIndent || hierarchyBusy"
            @click="emit('indent')"
          >
            <AppIcon name="indent" :size="15" />
            降为子级
          </button>
          <button
            type="button"
            class="ghost-button button-row__item"
            :title="outdentHint"
            :disabled="!canOutdent || hierarchyBusy"
            @click="emit('outdent')"
          >
            <AppIcon name="outdent" :size="15" />
            提升一级
          </button>
        </div>
        <p class="detail-hint">{{ indentHint }}</p>
        <p class="detail-hint">{{ outdentHint }}</p>
      </section>

      <section v-if="otherCourses.length" class="detail-section">
        <h4>
          所属课程
          <span class="detail-section__hint">迁移时子知识点会一起移动</span>
        </h4>
        <p class="detail-course">
          当前：<strong>{{ detail.course_name }}</strong>
          <span v-if="detail.child_count > 0">（含 {{ detail.child_count }} 个子项）</span>
        </p>
        <div class="move-row">
          <select v-model="targetCourseId" class="select" :disabled="courseBusy">
            <option value="">选择目标课程</option>
            <option v-for="course in otherCourses" :key="course.id" :value="course.id">
              {{ course.name }}
            </option>
          </select>
          <button
            type="button"
            class="ghost-button"
            :disabled="!canMoveCourse"
            @click="submitMoveCourse"
          >
            {{ courseBusy ? '迁移中…' : '迁移' }}
          </button>
        </div>
      </section>

      <section class="detail-section">
        <h4>
          知识关联
          <span class="detail-section__hint">
            {{ detail.outgoing_relation_count }} 条出边 · {{ detail.incoming_relation_count }} 条入边
          </span>
        </h4>
        <div class="button-row">
          <button
            type="button"
            class="ghost-button button-row__item"
            @click="emit('openRelations', point.id)"
          >
            <AppIcon name="link" :size="15" />
            查看 / 管理关联
          </button>
        </div>
        <p class="detail-hint">
          语义关联与父子层级相互独立，可在知识关联页查看推荐并确认新的关系。
        </p>
      </section>

      <section class="detail-section">
        <h4>快捷操作</h4>
        <div class="button-row">
          <button type="button" class="ghost-button button-row__item" @click="emit('addChild')">
            <AppIcon name="plus" :size="15" />
            新建子知识点
          </button>
          <button
            type="button"
            class="ghost-button button-row__item"
            @click="emit('openNote', point.id)"
          >
            <AppIcon name="notes" :size="15" />
            在笔记中打开
          </button>
        </div>
      </section>

      <footer class="detail-meta">
        <span>创建于 {{ formatDate(point.created_at) }}</span>
        <span>更新于 {{ formatDate(point.updated_at) }}</span>
      </footer>
    </template>
  </div>
</template>

<style scoped>
.detail-panel {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px 18px 18px;
}

.detail-state {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
  padding: 40px 12px;
  color: var(--color-text-muted);
  font-size: 12.5px;
  text-align: center;
}

.detail-state--error {
  color: var(--color-danger);
}

.detail-empty {
  padding: 32px 10px;
}

.detail-head {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  justify-content: space-between;
}

.detail-head__titles {
  display: flex;
  flex-direction: column;
  gap: 5px;
  min-width: 0;
}

.detail-head__titles h3 {
  color: var(--color-text);
  font-size: 15px;
  font-weight: 600;
  line-height: 1.4;
  overflow-wrap: anywhere;
}

.level-badge {
  align-self: flex-start;
  padding: 0 7px;
  color: var(--color-accent-strong);
  font-size: 10.5px;
  line-height: 18px;
  background: var(--color-accent-soft);
  border-radius: 999px;
}

.detail-head__actions {
  display: flex;
  flex: 0 0 auto;
  gap: 2px;
}

.icon-button--danger:hover {
  color: var(--color-danger);
  background: var(--color-danger-soft);
}

.breadcrumb {
  display: flex;
  flex-wrap: wrap;
  gap: 2px 4px;
  align-items: center;
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.breadcrumb__link {
  padding: 0;
  color: var(--color-text-secondary);
  font-size: 11.5px;
  background: none;
  border: none;
}

.breadcrumb__link:hover {
  color: var(--color-accent);
  text-decoration: underline;
  text-underline-offset: 3px;
}

.breadcrumb__current {
  color: var(--color-text);
  font-weight: 500;
}

.breadcrumb__sep {
  color: var(--color-border-strong);
}

.detail-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}

.detail-stat {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 8px 10px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}

.detail-stat strong {
  font-size: 16px;
  font-weight: 600;
  line-height: 1.3;
}

.detail-stat span {
  color: var(--color-text-muted);
  font-size: 10.5px;
  white-space: nowrap;
}

.detail-section {
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding-top: 12px;
  border-top: 1px solid var(--color-border);
}

.detail-section h4 {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: baseline;
  color: var(--color-text-secondary);
  font-size: 11.5px;
  font-weight: 600;
}

.detail-section__hint {
  color: var(--color-text-muted);
  font-size: 10.5px;
  font-weight: 400;
}

.detail-description {
  color: var(--color-text-secondary);
  font-size: 12.5px;
  line-height: 1.75;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.detail-description.is-empty {
  color: var(--color-text-muted);
}

.button-row {
  display: flex;
  gap: 8px;
}

.button-row__item {
  flex: 1;
  padding: 0 10px;
}

.detail-hint {
  color: var(--color-text-muted);
  font-size: 10.5px;
  line-height: 1.5;
}

.detail-course {
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.detail-course strong {
  color: var(--color-text);
  font-weight: 600;
}

.move-row {
  display: flex;
  gap: 8px;
}

.move-row .select {
  flex: 1;
  min-width: 0;
}

.detail-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-top: 10px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  border-top: 1px solid var(--color-border);
}
</style>
