<script setup lang="ts">
import { computed } from 'vue'
import { Handle, Position, type NodeProps } from '@vue-flow/core'
import AppIcon from '@/components/common/AppIcon.vue'
import type { GraphNodeData } from '@/types'

const props = defineProps<NodeProps<GraphNodeData>>()

const depthLabel = computed(() =>
  props.data.point.depth === 0 ? '顶层' : `L${props.data.point.depth}`,
)
</script>

<template>
  <div
    class="graph-node"
    :class="{
      'graph-node--root': data.point.depth === 0,
      'graph-node--related': data.relationCount > 0,
      'graph-node--selected': selected,
      'graph-node--dimmed': data.dimmed,
    }"
  >
    <!-- 连接点仅用于 Vue Flow 内部识别边的端点，实际几何由边路由的浮动锚点计算 -->
    <Handle type="target" :position="Position.Left" class="graph-node__handle" />
    <Handle type="source" :position="Position.Right" class="graph-node__handle" />

    <header class="graph-node__head">
      <span class="graph-node__level">{{ depthLabel }}</span>
      <span v-if="data.relationCount > 0" class="graph-node__relations">
        <AppIcon name="link" :size="10" />
        {{ data.relationCount }}
      </span>
    </header>

    <div class="graph-node__title" :title="data.pathLabel || data.point.title">
      <span class="graph-node__title-text">{{ data.point.title }}</span>
      <span v-if="data.scope" class="graph-node__scope">{{ data.scope }}</span>
    </div>

    <footer class="graph-node__meta">
      <span class="graph-node__meta-item">
        <AppIcon name="tree" :size="10" />
        {{ data.point.child_count }} 子项
      </span>
      <span class="graph-node__meta-item">
        <AppIcon name="notes" :size="10" />
        {{ data.point.note_count }} 笔记
      </span>
    </footer>
  </div>
</template>

<style scoped>
.graph-node {
  box-sizing: border-box;
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
  padding: 9px 12px 10px;
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  box-shadow: var(--shadow-sm);
  transition:
    border-color var(--transition-fast),
    box-shadow var(--transition-fast),
    opacity var(--transition-fast),
    filter var(--transition-fast);
}

/* 左侧身份色条：顶层用主题色，有语义关联的节点用成功色 */
.graph-node::before {
  content: '';
  position: absolute;
  top: 10px;
  bottom: 10px;
  left: 0;
  width: 3px;
  background: var(--color-border-strong);
  border-radius: 0 3px 3px 0;
  opacity: 0.75;
}

.graph-node--root::before {
  background: var(--color-accent);
  opacity: 1;
}

.graph-node--related:not(.graph-node--root)::before {
  background: var(--color-success);
  opacity: 1;
}

.graph-node:hover {
  border-color: var(--color-accent);
  box-shadow: var(--shadow-md);
}

.graph-node--dimmed {
  opacity: 0.32;
  filter: saturate(0.4);
}

.graph-node--dimmed:hover {
  opacity: 1;
  filter: none;
}

.graph-node--selected {
  border-color: var(--color-accent);
  box-shadow:
    0 0 0 3px var(--color-accent-soft),
    var(--shadow-md);
}

.graph-node--selected::before {
  background: var(--color-accent);
  opacity: 1;
}

.graph-node__head {
  display: flex;
  gap: 6px;
  align-items: center;
  justify-content: space-between;
  min-height: 16px;
}

.graph-node__level {
  color: var(--color-text-muted);
  font-size: 9px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.graph-node--root .graph-node__level {
  color: var(--color-accent-strong);
}

.graph-node__relations {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 2px;
  align-items: center;
  padding: 0 5px;
  color: var(--color-accent-strong);
  font-size: 9.5px;
  font-weight: 600;
  line-height: 15px;
  background: var(--color-accent-soft);
  border-radius: 999px;
}

.graph-node__title {
  display: flex;
  gap: 4px;
  align-items: baseline;
  min-width: 0;
  overflow: hidden;
  color: var(--color-text);
  font-size: 12.5px;
  font-weight: 600;
  line-height: 1.35;
  white-space: nowrap;
}

.graph-node__title-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.graph-node__scope {
  flex: 0 1 auto;
  min-width: 0;
  max-width: 48%;
  overflow: hidden;
  color: var(--color-text-muted);
  font-size: 9px;
  font-weight: 500;
  text-overflow: ellipsis;
}

.graph-node__meta {
  display: flex;
  gap: 10px;
  color: var(--color-text-muted);
  font-size: 9px;
  line-height: 1.3;
}

.graph-node__meta-item {
  display: inline-flex;
  gap: 3px;
  align-items: center;
  opacity: 0.85;
}

.graph-node__handle {
  width: 5px;
  height: 5px;
  background: var(--color-accent);
  border: 1px solid var(--color-surface);
  opacity: 0;
  pointer-events: none;
}
</style>
