<script setup lang="ts">
import { computed, ref } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import { useNotesStore } from '@/stores/notes'
import type { KnowledgePoint } from '@/types'

const props = defineProps<{
  point: KnowledgePoint
  depth: number
}>()

const notes = useNotesStore()
const expanded = ref(true)

const children = computed(() => notes.childrenOf(props.point.id))
const hasChildren = computed(() => children.value.length > 0)
const isActive = computed(() => notes.selectedPointId === props.point.id)
</script>

<template>
  <li class="kp-node">
    <div
      class="kp-row"
      :class="{ 'is-active': isActive }"
      :style="{ paddingLeft: `${10 + depth * 14}px` }"
      role="button"
      tabindex="0"
      :aria-current="isActive ? 'true' : undefined"
      @click="notes.selectPoint(point.id)"
      @keydown.enter.prevent="notes.selectPoint(point.id)"
      @keydown.space.prevent="notes.selectPoint(point.id)"
    >
      <button
        v-if="hasChildren"
        type="button"
        class="kp-row__toggle"
        :class="{ 'is-open': expanded }"
        :aria-expanded="expanded"
        :aria-label="expanded ? '收起子知识点' : '展开子知识点'"
        @click.stop="expanded = !expanded"
        @keydown.stop
      >
        <AppIcon name="chevron-right" :size="13" />
      </button>
      <span v-else class="kp-row__toggle kp-row__toggle--empty" aria-hidden="true" />

      <span class="kp-row__title">{{ point.title }}</span>
    </div>

    <ul v-if="hasChildren && expanded" class="kp-children">
      <KnowledgeTreeNode
        v-for="child in children"
        :key="child.id"
        :point="child"
        :depth="depth + 1"
      />
    </ul>
  </li>
</template>

<style scoped>
.kp-row {
  position: relative;
  display: flex;
  gap: 4px;
  align-items: center;
  height: 29px;
  padding-right: 8px;
  color: var(--color-text-secondary);
  font-size: 12.5px;
  white-space: nowrap;
  cursor: pointer;
  border-radius: 6px;
  user-select: none;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.kp-row:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.kp-row.is-active {
  color: var(--color-text);
  font-weight: 600;
  background: var(--color-surface-hover);
}

.kp-row.is-active::before {
  display: none;
}

.kp-row:focus-visible {
  outline-offset: -2px;
}

.kp-row__toggle {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 18px;
  height: 18px;
  padding: 0;
  color: var(--color-text-muted);
  background: transparent;
  border: none;
  border-radius: 4px;
}

.kp-row__toggle:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.kp-row__toggle svg {
  transition: transform 150ms ease;
}

.kp-row__toggle.is-open svg {
  transform: rotate(90deg);
}

.kp-row__toggle--empty {
  pointer-events: none;
}

.kp-row__title {
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
