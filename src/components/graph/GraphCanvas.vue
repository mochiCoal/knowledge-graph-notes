<script setup lang="ts">
import { markRaw, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import {
  MarkerType,
  Panel,
  Position,
  VueFlow,
  useVueFlow,
  type Edge,
  type Node,
  type NodeMouseEvent,
} from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import AppIcon from '@/components/common/AppIcon.vue'
import GraphNode from './GraphNode.vue'
import type {
  GraphLayoutMode,
  GraphNodeData,
  GraphPoint,
  RelationDetail,
} from '@/types'
import { GRAPH_NODE_WIDTH, layoutForce, layoutHierarchy } from '@/utils/graphLayout'

import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import '@vue-flow/controls/dist/style.css'

const props = defineProps<{
  points: GraphPoint[]
  relations: RelationDetail[]
  layout: GraphLayoutMode
  showHierarchyEdges: boolean
  showEdgeLabels: boolean
  selectedId: string | null
}>()

const emit = defineEmits<{
  select: [pointId: string | null]
}>()

const { fitView } = useVueFlow()

/** Node 类型本身不含 selected，Vue Flow 的运行时节点会补上该字段 */
type GraphFlowNode = Node<GraphNodeData> & { selected?: boolean }

const nodeTypes = { knowledge: markRaw(GraphNode) }
const nodes = ref<GraphFlowNode[]>([])
const edges = ref<Edge[]>([])

let fitTimer: number | null = null

function relationCounts(): Map<string, number> {
  const counts = new Map<string, number>()
  for (const relation of props.relations) {
    counts.set(relation.source_id, (counts.get(relation.source_id) ?? 0) + 1)
    counts.set(relation.target_id, (counts.get(relation.target_id) ?? 0) + 1)
  }
  return counts
}

function buildNodes(): GraphFlowNode[] {
  const positions =
    props.layout === 'hierarchy'
      ? layoutHierarchy(props.points)
      : layoutForce(props.points, props.relations)
  const counts = relationCounts()

  return props.points.map((point, index) => ({
    id: point.id,
    type: 'knowledge',
    position: positions.get(point.id) ?? { x: index * 240, y: 0 },
    data: {
      point,
      relationCount: counts.get(point.id) ?? 0,
    },
    sourcePosition: Position.Right,
    targetPosition: Position.Left,
    draggable: true,
    connectable: false,
    selectable: true,
    selected: point.id === props.selectedId,
    style: { width: `${GRAPH_NODE_WIDTH}px` },
  }))
}

function buildEdges(): Edge[] {
  const result: Edge[] = []
  const ids = new Set(props.points.map((point) => point.id))

  if (props.showHierarchyEdges) {
    for (const point of props.points) {
      if (!point.parent_id || !ids.has(point.parent_id)) continue

      result.push({
        id: `hierarchy:${point.parent_id}:${point.id}`,
        source: point.parent_id,
        target: point.id,
        sourceHandle: 'source-top',
        targetHandle: 'target-top',
        type: props.layout === 'hierarchy' ? 'smoothstep' : 'default',
        style: {
          stroke: 'var(--color-border-strong)',
          strokeWidth: 1.3,
          strokeDasharray: '5 4',
        },
        selectable: false,
        focusable: false,
        zIndex: 0,
      })
    }
  }

  for (const relation of props.relations) {
    if (!ids.has(relation.source_id) || !ids.has(relation.target_id)) continue

    result.push({
      id: `relation:${relation.id}`,
      source: relation.source_id,
      target: relation.target_id,
      sourceHandle: 'source-bottom',
      targetHandle: 'target-bottom',
      type: 'default',
      label: props.showEdgeLabels ? relation.relation_label : undefined,
      labelBgPadding: [5, 3],
      labelBgBorderRadius: 4,
      labelBgStyle: { fill: 'var(--color-surface)', fillOpacity: 0.94 },
      labelStyle: {
        fill: 'var(--color-text-secondary)',
        fontSize: '10px',
        fontWeight: 600,
      },
      style: {
        stroke: relation.relation_color,
        strokeWidth: 1.8,
      },
      markerEnd: relation.directed
        ? {
            type: MarkerType.ArrowClosed,
            color: relation.relation_color,
            width: 16,
            height: 16,
          }
        : undefined,
      selectable: false,
      focusable: false,
      zIndex: 1,
    })
  }

  return result
}

function scheduleFit(): void {
  if (fitTimer !== null) window.clearTimeout(fitTimer)

  fitTimer = window.setTimeout(() => {
    fitTimer = null
    void nextTick(() => {
      void fitView({ padding: 0.22, duration: 380, minZoom: 0.08, maxZoom: 1.35 })
    })
  }, 90)
}

function rebuildGraph(): void {
  if (props.points.length === 0) {
    nodes.value = []
    edges.value = []
    return
  }

  nodes.value = buildNodes()
  edges.value = buildEdges()
  scheduleFit()
}

function rebuildEdges(): void {
  edges.value = buildEdges()
}

function onNodeClick(event: NodeMouseEvent): void {
  emit('select', event.node.id)
}

function onPaneClick(): void {
  emit('select', null)
}

watch(
  [() => props.points, () => props.relations, () => props.layout],
  rebuildGraph,
)

watch(
  [() => props.showHierarchyEdges, () => props.showEdgeLabels],
  rebuildEdges,
)

watch(
  () => props.selectedId,
  (selectedId) => {
    for (const node of nodes.value) {
      node.selected = node.id === selectedId
    }
  },
)

onMounted(rebuildGraph)

onBeforeUnmount(() => {
  if (fitTimer !== null) window.clearTimeout(fitTimer)
})
</script>

<template>
  <div class="graph-canvas">
    <VueFlow
      :nodes="nodes"
      :edges="edges"
      :node-types="nodeTypes"
      :min-zoom="0.08"
      :max-zoom="2"
      :default-viewport="{ x: 0, y: 0, zoom: 0.85 }"
      :fit-view-on-init="true"
      :nodes-connectable="false"
      :edges-updatable="false"
      :delete-key-code="null"
      :zoom-on-double-click="false"
      :elevate-edges-on-select="false"
      :elevate-nodes-on-select="false"
      :only-render-visible-elements="true"
      class="graph-canvas__flow"
      @node-click="onNodeClick"
      @pane-click="onPaneClick"
    >
      <Background
        variant="dots"
        :gap="22"
        :size="1.1"
        color="var(--color-border-strong)"
      />
      <Controls
        position="bottom-right"
        :show-interactive="false"
        :fit-view-params="{ padding: 0.22, duration: 300 }"
      />

      <Panel position="top-right" class="graph-canvas__panel">
        <button type="button" class="graph-canvas__action" @click="rebuildGraph">
          <AppIcon name="refresh" :size="13" />
          重新布局
        </button>
      </Panel>

      <Panel position="bottom-left" class="graph-legend">
        <span class="graph-legend__item">
          <span class="graph-legend__line graph-legend__line--hierarchy" />
          层级关系
        </span>
        <span class="graph-legend__item">
          <span class="graph-legend__line graph-legend__line--relation" />
          语义关联
        </span>
        <span class="graph-legend__item">
          <span class="graph-legend__arrow">→</span>
          有向关联
        </span>
      </Panel>
    </VueFlow>
  </div>
</template>

<style scoped>
.graph-canvas {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background:
    radial-gradient(circle at 50% 0%, var(--color-accent-soft), transparent 52%),
    var(--color-bg);
}

.graph-canvas__flow {
  background: transparent;
}

.graph-canvas__panel {
  margin: 10px;
}

.graph-canvas__action {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  height: 28px;
  padding: 0 10px;
  color: var(--color-text-secondary);
  font-size: 11px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 999px;
  box-shadow: var(--shadow-sm);
}

.graph-canvas__action:hover {
  color: var(--color-text);
  border-color: var(--color-border-strong);
  background: var(--color-surface-hover);
}

.graph-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 12px;
  align-items: center;
  padding: 7px 11px;
  margin: 10px;
  color: var(--color-text-muted);
  font-size: 10px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 999px;
  box-shadow: var(--shadow-sm);
}

.graph-legend__item {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  white-space: nowrap;
}

.graph-legend__line {
  width: 18px;
  height: 0;
  border-top: 1.5px solid var(--color-border-strong);
}

.graph-legend__line--hierarchy {
  border-top-style: dashed;
}

.graph-legend__line--relation {
  border-top-color: var(--color-accent);
}

.graph-legend__arrow {
  color: var(--color-accent-strong);
  font-size: 12px;
  line-height: 1;
}

.graph-canvas :deep(.vue-flow) {
  background: transparent;
}

.graph-canvas :deep(.vue-flow__pane) {
  cursor: grab;
}

.graph-canvas :deep(.vue-flow__pane:active) {
  cursor: grabbing;
}

.graph-canvas :deep(.vue-flow__node) {
  border: none;
  background: transparent;
  padding: 0;
  font-family: inherit;
}

.graph-canvas :deep(.vue-flow__node.selected) {
  box-shadow: none;
}

.graph-canvas :deep(.vue-flow__edge-path) {
  stroke-linecap: round;
}

.graph-canvas :deep(.vue-flow__edge-text) {
  font-family: inherit;
}

.graph-canvas :deep(.vue-flow__controls) {
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-md);
}

.graph-canvas :deep(.vue-flow__controls-button) {
  width: 28px;
  height: 28px;
  color: var(--color-text-secondary);
  background: var(--color-surface);
  border-bottom-color: var(--color-border);
}

.graph-canvas :deep(.vue-flow__controls-button:hover) {
  background: var(--color-surface-hover);
}

.graph-canvas :deep(.vue-flow__controls-button svg) {
  fill: currentcolor;
}

.graph-canvas :deep(.vue-flow__attribution) {
  display: none;
}
</style>
