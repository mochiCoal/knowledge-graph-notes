<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import type { IconName } from '@/components/common/icons'
import { searchApi } from '@/services/search'
import type { GlobalSearchResult, SearchHit, SearchHitKind } from '@/types'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{ close: [] }>()

const router = useRouter()

const keyword = ref('')
const result = ref<GlobalSearchResult | null>(null)
const isLoading = ref(false)
const errorMessage = ref('')
const activeIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

let searchTimer: number | null = null
/** 请求序号：丢弃过期请求的结果，避免慢请求覆盖新关键字 */
let requestId = 0

interface GroupDefinition {
  label: string
  icon: IconName
}

const GROUP_META: Record<SearchHitKind, GroupDefinition> = {
  course: { label: '课程', icon: 'courses' },
  knowledge_point: { label: '知识点', icon: 'tree' },
  note: { label: '笔记', icon: 'notes' },
  tag: { label: '标签', icon: 'tag' },
  relation: { label: '知识关联', icon: 'link' },
}

const groups = computed(() => {
  const data = result.value
  if (!data) return []

  const definitions: Array<[SearchHitKind, SearchHit[]]> = [
    ['course', data.courses],
    ['knowledge_point', data.knowledge_points],
    ['note', data.notes],
    ['tag', data.tags],
    ['relation', data.relations],
  ]

  return definitions
    .filter(([, hits]) => hits.length > 0)
    .map(([kind, hits]) => ({ kind, ...GROUP_META[kind], hits }))
})

/** 带全局序号的搜索结果，用于 ↑ / ↓ 在分组之间连续移动 */
const groupedRows = computed(() => {
  let index = 0

  return groups.value.map((group) => ({
    ...group,
    rows: group.hits.map((hit) => ({ hit, index: index++ })),
  }))
})

const flatHits = computed<SearchHit[]>(() => groups.value.flatMap((group) => group.hits))

const hasKeyword = computed(() => keyword.value.trim() !== '')
const resultCount = computed(() => result.value?.total ?? 0)

function resetState(): void {
  keyword.value = ''
  result.value = null
  isLoading.value = false
  errorMessage.value = ''
  activeIndex.value = 0
  requestId += 1

  if (searchTimer !== null) {
    window.clearTimeout(searchTimer)
    searchTimer = null
  }
}

watch(
  () => props.open,
  async (open) => {
    if (!open) {
      // 关闭时让仍在进行的请求失效，避免结果在重新打开前被旧请求写回
      requestId += 1
      if (searchTimer !== null) {
        window.clearTimeout(searchTimer)
        searchTimer = null
      }
      return
    }

    resetState()
    await nextTick()
    inputRef.value?.focus()
  },
)

async function runSearch(query: string, currentRequest: number): Promise<void> {
  try {
    const data = await searchApi.global(query)
    if (currentRequest !== requestId) return

    result.value = data
    activeIndex.value = 0
  } catch (error) {
    if (currentRequest !== requestId) return

    result.value = null
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    if (currentRequest === requestId) {
      isLoading.value = false
    }
  }
}

function scheduleSearch(): void {
  errorMessage.value = ''

  if (searchTimer !== null) {
    window.clearTimeout(searchTimer)
    searchTimer = null
  }

  const query = keyword.value.trim()
  if (!query) {
    requestId += 1
    result.value = null
    isLoading.value = false
    activeIndex.value = 0
    return
  }

  const currentRequest = ++requestId
  isLoading.value = true
  searchTimer = window.setTimeout(() => {
    searchTimer = null
    void runSearch(query, currentRequest)
  }, 220)
}

function clearSearch(): void {
  keyword.value = ''
  scheduleSearch()
  inputRef.value?.focus()
}

async function scrollActiveIntoView(): Promise<void> {
  await nextTick()
  document
    .getElementById(`global-search-hit-${activeIndex.value}`)
    ?.scrollIntoView({ block: 'nearest' })
}

function moveActive(delta: number): void {
  const total = flatHits.value.length
  if (total === 0) return

  activeIndex.value = (activeIndex.value + delta + total) % total
  void scrollActiveIntoView()
}

function navigateTo(hit: SearchHit): void {
  if (hit.kind === 'course') {
    void router.push({ name: 'course-detail', params: { courseId: hit.id } })
  } else if (hit.kind === 'knowledge_point') {
    if (hit.course_id) {
      void router.push({
        name: 'knowledge-points',
        params: { courseId: hit.course_id },
        query: { point: hit.id },
      })
    } else {
      void router.push({ name: 'knowledge-points', query: { point: hit.id } })
    }
  } else if (hit.kind === 'note') {
    if (hit.knowledge_point_id) {
      void router.push({ name: 'notes', query: { point: hit.knowledge_point_id } })
    } else {
      void router.push({ name: 'note-library' })
    }
  } else if (hit.kind === 'tag') {
    void router.push({ name: 'note-library', query: { tag: hit.id } })
  } else {
    const query: Record<string, string> = {}
    if (hit.course_id) query.course = hit.course_id
    if (hit.knowledge_point_id) query.point = hit.knowledge_point_id

    void router.push({ name: 'relations', query })
  }

  emit('close')
}

function openActive(): void {
  const hit = flatHits.value[activeIndex.value]
  if (hit) navigateTo(hit)
}

function selectHit(index: number): void {
  activeIndex.value = index
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.preventDefault()
    emit('close')
    return
  }

  if (event.key === 'ArrowDown') {
    event.preventDefault()
    moveActive(1)
    return
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault()
    moveActive(-1)
    return
  }

  if (event.key === 'Enter') {
    event.preventDefault()
    openActive()
  }
}

onBeforeUnmount(() => {
  if (searchTimer !== null) window.clearTimeout(searchTimer)
})
</script>

<template>
  <Teleport to="body">
    <Transition name="palette">
      <div
        v-if="open"
        class="search-overlay"
        role="presentation"
        @mousedown.self="emit('close')"
      >
        <section
          class="search-panel"
          role="dialog"
          aria-modal="true"
          aria-label="全局搜索"
        >
          <header class="search-head">
            <AppIcon name="search" :size="18" class="search-head__icon" />
            <input
              ref="inputRef"
              v-model="keyword"
              class="search-head__input"
              type="text"
              name="global-search"
              autocomplete="off"
              spellcheck="false"
              placeholder="搜索课程、知识点、笔记、标签…"
              aria-label="搜索关键字"
              @input="scheduleSearch"
              @keydown="onKeydown"
            />
            <span v-if="isLoading" class="spinner search-head__spinner" />
            <button
              v-else-if="keyword"
              type="button"
              class="search-head__clear"
              aria-label="清空搜索"
              @click="clearSearch"
            >
              <AppIcon name="x" :size="14" />
            </button>
            <kbd class="search-head__esc">Esc</kbd>
          </header>

          <div class="search-body">
            <div v-if="errorMessage" class="search-state search-state--error" role="alert">
              <AppIcon name="alert" :size="20" />
              <strong>搜索失败</strong>
              <p>{{ errorMessage }}</p>
            </div>

            <div v-else-if="!hasKeyword" class="search-state search-state--hint">
              <span class="search-state__icon">
                <AppIcon name="search" :size="22" />
              </span>
              <strong>在整个知识库中查找</strong>
              <p>输入课程、知识点或笔记的关键字，也可以搜索标签与知识点关联。</p>
              <div class="search-tips">
                <span v-for="item in GROUP_META" :key="item.label" class="search-tip">
                  <AppIcon :name="item.icon" :size="13" />
                  {{ item.label }}
                </span>
              </div>
            </div>

            <div v-else-if="isLoading && !result" class="search-state">
              <span class="spinner" />
              <p>正在搜索「{{ keyword.trim() }}」…</p>
            </div>

            <div v-else-if="result && resultCount === 0" class="search-state">
              <span class="search-state__icon">
                <AppIcon name="search" :size="22" />
              </span>
              <strong>没有找到「{{ result.query }}」</strong>
              <p>试试更短的关键字，或者换一个说法。</p>
            </div>

            <div v-else-if="result" class="search-results">
              <section v-for="group in groupedRows" :key="group.kind" class="result-group">
                <h3 class="result-group__title">
                  <AppIcon :name="group.icon" :size="13" />
                  {{ group.label }}
                  <span class="result-group__count">{{ group.rows.length }}</span>
                </h3>

                <button
                  v-for="row in group.rows"
                  :id="`global-search-hit-${row.index}`"
                  :key="`${row.hit.kind}-${row.hit.id}`"
                  type="button"
                  class="result-item"
                  :class="{ 'is-active': activeIndex === row.index }"
                  @mouseenter="selectHit(row.index)"
                  @click="navigateTo(row.hit)"
                >
                  <span class="result-item__icon">
                    <AppIcon :name="group.icon" :size="15" />
                  </span>

                  <span class="result-item__body">
                    <span class="result-item__title">{{ row.hit.title }}</span>
                    <span v-if="row.hit.subtitle" class="result-item__subtitle">
                      {{ row.hit.subtitle }}
                    </span>
                    <span v-if="row.hit.excerpt" class="result-item__excerpt">
                      {{ row.hit.excerpt }}
                    </span>
                  </span>

                  <AppIcon
                    name="arrow-right"
                    :size="14"
                    class="result-item__arrow"
                  />
                </button>
              </section>
            </div>
          </div>

          <footer class="search-foot">
            <span class="search-foot__keys">
              <kbd>↑</kbd>
              <kbd>↓</kbd>
              选择
              <kbd>Enter</kbd>
              打开
            </span>
            <span v-if="hasKeyword && result && !errorMessage" class="search-foot__count">
              {{ resultCount }} 条结果
            </span>
            <span v-else class="search-foot__hint">Ctrl + K 随时唤起</span>
          </footer>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.search-overlay {
  position: fixed;
  inset: 0;
  z-index: 300;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: 10vh 24px 24px;
  background: var(--color-overlay);
  backdrop-filter: blur(2px);
}

.search-panel {
  display: flex;
  flex-direction: column;
  width: min(680px, 100%);
  max-height: min(72vh, 620px);
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-pop);
}

.search-head {
  display: flex;
  flex: 0 0 auto;
  gap: 10px;
  align-items: center;
  padding: 13px 14px 13px 16px;
  border-bottom: 1px solid var(--color-border);
}

.search-head__icon {
  color: var(--color-text-muted);
}

.search-head__input {
  flex: 1;
  min-width: 0;
  padding: 2px 0;
  color: var(--color-text);
  font-size: 15px;
  background: transparent;
  border: 0;
  outline: none;
}

.search-head__input::placeholder {
  color: var(--color-text-muted);
}

.search-head__spinner {
  flex: 0 0 auto;
  width: 14px;
  height: 14px;
}

.search-head__clear {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 22px;
  height: 22px;
  padding: 0;
  color: var(--color-text-muted);
  background: transparent;
  border: 0;
  border-radius: 5px;
  transition:
    color var(--transition-fast),
    background-color var(--transition-fast);
}

.search-head__clear:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.search-head__esc {
  flex: 0 0 auto;
}

.search-body {
  flex: 1;
  min-height: 0;
  padding: 6px 8px 8px;
  overflow-y: auto;
}

.search-state {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
  justify-content: center;
  min-height: 210px;
  padding: 24px;
  color: var(--color-text-muted);
  font-size: 12.5px;
  text-align: center;
}

.search-state strong {
  color: var(--color-text);
  font-size: 13.5px;
  font-weight: 600;
}

.search-state p {
  max-width: 380px;
  line-height: 1.7;
}

.search-state__icon {
  display: grid;
  place-items: center;
  width: 42px;
  height: 42px;
  margin-bottom: 4px;
  color: var(--color-text-secondary);
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 10px;
}

.search-state--error {
  color: var(--color-danger);
}

.search-state--error strong,
.search-state--error p {
  color: var(--color-danger);
}

.search-tips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: center;
  margin-top: 10px;
}

.search-tip {
  display: inline-flex;
  gap: 4px;
  align-items: center;
  padding: 3px 8px;
  color: var(--color-text-secondary);
  font-size: 10.5px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.result-group {
  padding: 2px 0 4px;
}

.result-group__title {
  display: flex;
  gap: 5px;
  align-items: center;
  padding: 9px 10px 6px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.04em;
}

.result-group__count {
  padding: 0 5px;
  font-size: 10px;
  font-variant-numeric: tabular-nums;
  background: var(--color-surface-soft);
  border-radius: 999px;
}

.result-item {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  width: 100%;
  padding: 8px 9px;
  color: var(--color-text);
  text-align: left;
  background: transparent;
  border: 0;
  border-radius: 7px;
  transition: background-color var(--transition-fast);
}

.result-item.is-active {
  background: var(--color-surface-hover);
}

.result-item__icon {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 28px;
  height: 28px;
  margin-top: 1px;
  color: var(--color-text-secondary);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 6px;
}

.result-item__body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.result-item__title {
  overflow: hidden;
  font-size: 13px;
  font-weight: 500;
  line-height: 1.5;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.result-item__subtitle {
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 10.5px;
  line-height: 1.5;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.result-item__excerpt {
  display: -webkit-box;
  margin-top: 1px;
  overflow: hidden;
  color: var(--color-text-secondary);
  font-size: 11.5px;
  line-height: 1.55;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  line-clamp: 2;
}

.result-item__arrow {
  flex: 0 0 auto;
  margin-top: 7px;
  color: var(--color-text-muted);
  opacity: 0;
  transform: translateX(-3px);
  transition:
    opacity var(--transition-fast),
    transform var(--transition-fast);
}

.result-item.is-active .result-item__arrow {
  opacity: 1;
  transform: translateX(0);
}

.search-foot {
  display: flex;
  flex: 0 0 auto;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px;
  color: var(--color-text-muted);
  font-size: 10.5px;
  border-top: 1px solid var(--color-border);
}

.search-foot__keys {
  display: inline-flex;
  gap: 4px;
  align-items: center;
}

.search-foot__count {
  color: var(--color-text-secondary);
  font-variant-numeric: tabular-nums;
}

kbd {
  display: inline-grid;
  min-width: 18px;
  height: 17px;
  padding: 0 4px;
  place-items: center;
  color: var(--color-text-secondary);
  font-family: var(--font-sans);
  font-size: 10px;
  line-height: 1;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border-strong);
  border-radius: 4px;
}

.palette-enter-active,
.palette-leave-active {
  transition: opacity 130ms ease;
}

.palette-enter-active .search-panel,
.palette-leave-active .search-panel {
  transition:
    opacity 150ms ease,
    transform 150ms cubic-bezier(0.16, 1, 0.3, 1);
}

.palette-enter-from,
.palette-leave-to {
  opacity: 0;
}

.palette-enter-from .search-panel,
.palette-leave-to .search-panel {
  opacity: 0;
  transform: translateY(-8px) scale(0.99);
}

@media (max-width: 600px) {
  .search-overlay {
    padding: 8vh 12px 12px;
  }

  .search-panel {
    max-height: 80vh;
  }
}
</style>
