<script setup lang="ts">
import AppIcon from '@/components/common/AppIcon.vue'
import { useKnowledgePointsStore } from '@/stores/knowledgePoints'
import type { KnowledgePointNode } from '@/types'

withDefaults(
  defineProps<{
    nodes: KnowledgePointNode[]
    /** 层级深度，用于缩进 */
    depth?: number
  }>(),
  { depth: 0 },
)

const emit = defineEmits<{
  addChild: [node: KnowledgePointNode]
  edit: [node: KnowledgePointNode]
  delete: [node: KnowledgePointNode]
}>()

const store = useKnowledgePointsStore()
</script>

<template>
  <ul class="knowledge-tree" :class="{ 'knowledge-tree--nested': depth > 0 }">
    <li v-for="node in nodes" :key="node.id" class="knowledge-tree__item">
      <div
        class="knowledge-row"
        :class="{ 'is-active': store.selectedId === node.id }"
        :style="{ paddingLeft: `${8 + depth * 18}px` }"
      >
        <button
          v-if="node.children.length"
          type="button"
          class="knowledge-row__toggle"
          :class="{ 'is-open': store.isExpanded(node.id) }"
          :aria-expanded="store.isExpanded(node.id)"
          :aria-label="store.isExpanded(node.id) ? '收起子知识点' : '展开子知识点'"
          @click.stop="store.toggleExpanded(node.id)"
        >
          <AppIcon name="chevron-right" :size="13" />
        </button>
        <span v-else class="knowledge-row__toggle knowledge-row__toggle--empty" aria-hidden="true" />

        <span
          class="knowledge-row__mark"
          :class="{ 'is-leaf': !node.children.length }"
          aria-hidden="true"
        />

        <button
          type="button"
          class="knowledge-row__main"
          :title="node.description || node.title"
          @click="store.selectPoint(node.id)"
        >
          <span class="knowledge-row__title">{{ node.title }}</span>
          <span v-if="node.child_count || node.note_count" class="knowledge-row__meta">
            <span v-if="node.child_count">{{ node.child_count }} 子项</span>
            <span v-if="node.note_count">{{ node.note_count }} 笔记</span>
          </span>
        </button>

        <span class="knowledge-row__actions">
          <button
            type="button"
            class="knowledge-row__action"
            title="新建子知识点"
            aria-label="新建子知识点"
            @click.stop="emit('addChild', node)"
          >
            <AppIcon name="plus" :size="14" />
          </button>
          <button
            type="button"
            class="knowledge-row__action"
            title="编辑知识点"
            aria-label="编辑知识点"
            @click.stop="emit('edit', node)"
          >
            <AppIcon name="pencil" :size="14" />
          </button>
          <button
            type="button"
            class="knowledge-row__action knowledge-row__action--danger"
            title="删除知识点"
            aria-label="删除知识点"
            @click.stop="emit('delete', node)"
          >
            <AppIcon name="trash" :size="14" />
          </button>
        </span>
      </div>

      <KnowledgePointTree
        v-if="node.children.length && store.isExpanded(node.id)"
        :nodes="node.children"
        :depth="depth + 1"
        @add-child="emit('addChild', $event)"
        @edit="emit('edit', $event)"
        @delete="emit('delete', $event)"
      />
    </li>
  </ul>
</template>

<style scoped>
.knowledge-tree--nested {
  position: relative;
}

.knowledge-row {
  position: relative;
  display: flex;
  gap: 4px;
  align-items: center;
  min-height: 32px;
  padding-right: 6px;
  color: var(--color-text-secondary);
  border-radius: 7px;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.knowledge-row:hover {
  background: var(--color-surface-hover);
}

.knowledge-row.is-active {
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
}

.knowledge-row.is-active::before {
  content: '';
  position: absolute;
  top: 7px;
  bottom: 7px;
  left: 2px;
  width: 2px;
  background: var(--color-accent);
  border-radius: 2px;
}

.knowledge-row__toggle {
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

.knowledge-row__toggle:hover {
  color: var(--color-text);
  background: var(--color-surface);
}

.knowledge-row__toggle svg {
  transition: transform 150ms ease;
}

.knowledge-row__toggle.is-open svg {
  transform: rotate(90deg);
}

.knowledge-row__toggle--empty {
  pointer-events: none;
}

.knowledge-row__mark {
  flex: 0 0 auto;
  width: 5px;
  height: 5px;
  background: var(--color-accent);
  border-radius: 50%;
  opacity: 0.55;
}

.knowledge-row__mark.is-leaf {
  background: var(--color-text-muted);
  opacity: 0.5;
}

.knowledge-row__main {
  display: flex;
  flex: 1;
  gap: 8px;
  align-items: baseline;
  min-width: 0;
  padding: 6px 4px;
  color: inherit;
  text-align: left;
  background: none;
  border: none;
}

.knowledge-row__title {
  overflow: hidden;
  font-size: 12.5px;
  font-weight: 500;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.knowledge-row.is-active .knowledge-row__title {
  color: var(--color-accent-strong);
}

.knowledge-row__meta {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 6px;
  color: var(--color-text-muted);
  font-size: 10.5px;
}

.knowledge-row__actions {
  display: flex;
  flex: 0 0 auto;
  gap: 1px;
  opacity: 0;
  transition: opacity var(--transition-fast);
}

.knowledge-row:hover .knowledge-row__actions,
.knowledge-row:focus-within .knowledge-row__actions {
  opacity: 1;
}

.knowledge-row__action {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  padding: 0;
  color: var(--color-text-muted);
  background: transparent;
  border: none;
  border-radius: 5px;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.knowledge-row__action:hover {
  color: var(--color-accent);
  background: var(--color-surface);
}

.knowledge-row__action--danger:hover {
  color: var(--color-danger);
  background: var(--color-danger-soft);
}
</style>
