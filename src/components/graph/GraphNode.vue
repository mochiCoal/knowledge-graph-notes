<script setup lang="ts">
import { computed } from 'vue'
import { Handle, Position, type NodeProps } from '@vue-flow/core'
import AppIcon from '@/components/common/AppIcon.vue'
import type { GraphNodeData } from '@/types'

const props = defineProps<NodeProps<GraphNodeData>>()

const depthLabel = computed(() =>
  props.data.point.depth === 0 ? '顶层' : `${props.data.point.depth} 级`,
)
</script>

<template>
  <div
    class="graph-node"
    :class="{
      'graph-node--root': data.point.depth === 0,
      'graph-node--related': data.relationCount > 0,
      'graph-node--selected': selected,
    }"
  >
    <Handle
      id="target-top"
      type="target"
      :position="Position.Left"
      :style="{ top: '34%' }"
      class="graph-node__handle"
    />
    <Handle
      id="source-top"
      type="source"
      :position="Position.Right"
      :style="{ top: '34%' }"
      class="graph-node__handle"
    />
    <Handle
      id="target-bottom"
      type="target"
      :position="Position.Left"
      :style="{ top: '68%' }"
      class="graph-node__handle"
    />
    <Handle
      id="source-bottom"
      type="source"
      :position="Position.Right"
      :style="{ top: '68%' }"
      class="graph-node__handle"
    />

    <header class="graph-node__head">
      <span class="graph-node__level">{{ depthLabel }}</span>
      <span v-if="data.relationCount > 0" class="graph-node__relations">
        <AppIcon name="link" :size="11" />
        {{ data.relationCount }}
      </span>
    </header>

    <div class="graph-node__title" :title="data.point.title">
      {{ data.point.title }}
    </div>

    <footer class="graph-node__meta">
      <span>{{ data.point.child_count }} 子项</span>
      <span>{{ data.point.note_count }} 笔记</span>
    </footer>
  </div>
</template>

<style scoped>
.graph-node {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 3px;
  width: 100%;
  padding: 8px 11px 9px;
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-left: 3px solid var(--color-border-strong);
  border-radius: 9px;
  box-shadow: var(--shadow-sm);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast),
    transform var(--transition-fast);
}

.graph-node:hover {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-md);
}

.graph-node--root {
  border-left-color: var(--color-accent);
}

.graph-node--related:not(.graph-node--root) {
  border-left-color: var(--color-success);
}

.graph-node--selected {
  border-color: var(--color-accent);
  border-left-color: var(--color-accent);
  box-shadow:
    0 0 0 2px var(--color-accent-soft),
    var(--shadow-md);
}

.graph-node__head {
  display: flex;
  gap: 6px;
  align-items: center;
  justify-content: space-between;
}

.graph-node__level {
  color: var(--color-text-muted);
  font-size: 9.5px;
  font-weight: 500;
  letter-spacing: 0.03em;
}

.graph-node__relations {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 2px;
  align-items: center;
  padding: 0 5px;
  color: var(--color-accent-strong);
  font-size: 9.5px;
  line-height: 16px;
  background: var(--color-accent-soft);
  border-radius: 999px;
}

.graph-node__title {
  overflow: hidden;
  color: var(--color-text);
  font-size: 12.5px;
  font-weight: 600;
  line-height: 1.35;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.graph-node__meta {
  display: flex;
  gap: 8px;
  color: var(--color-text-muted);
  font-size: 9.5px;
  line-height: 1.2;
}

.graph-node__handle {
  width: 5px;
  height: 5px;
  background: var(--color-border-strong);
  border: 1px solid var(--color-surface);
  opacity: 0;
}

.graph-node:hover .graph-node__handle,
.graph-node--selected .graph-node__handle {
  opacity: 1;
}
</style>
