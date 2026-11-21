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
import GraphEdge from './GraphEdge.vue'
import GraphNode from './GraphNode.vue'
import type {
  GraphEdgeData,
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
const edgeTypes = { graph: markRaw(GraphEdge) }
const nodes = ref<GraphFlowNode[]>([])
const edges = ref<Edge<GraphEdgeData>[]>([])

let fitTimer: number | null = null

function relationCounts(): Map<string, number> {
  const counts = new Map<string, number>()
  for (const relation of props.relations) {
    counts.set(relation.source_id, (counts.get(relation.source_id) ?? 0) + 1)
    counts.set(relation.target_id, (counts.get(relation.target_id) ?? 0) + 1)
  }
  return counts
}

/** 同名知识点的区分信息与完整层级路径 */
function graphMeta(): {
  paths: Map<string, string>
  scopes: Map<string, string | null>
} {
  const byId = new Map(props.points.map((point) => [point.id, point]))
  const titleCount = new Map<string, number>()

  for (const point of props.points) {
    titleCount.set(point.title, (titleCount.get(point.title) ?? 0) + 1)
  }

  const paths = new Map<string, string>()
  const pathOf = (point: GraphPoint): string => {
    const cached = paths.get(point.id)
    if (cached) return cached

    const titles: string[] = []
    const visited = new Set<string>()
    let current: GraphPoint | undefined = point

    while (current && !visited.has(current.id)) {
      visited.add(current.id)
      titles.unshift(current.title)
      current = current.parent_id ? byId.get(current.parent_id) : undefined
    }

    const path = titles.join(' › ')
    paths.set(point.id, path)
    return path
  }

  const scopes = new Map<string, string | null>()
  const seen = new Map<string, number>()

  for (const point of props.points) {
    pathOf(point)

    // 标题唯一时不需要额外的区分信息
    if ((titleCount.get(point.title) ?? 0) <= 1) {
      scopes.set(point.id, null)
      continue
    }

    const order = (seen.get(point.title) ?? 0) + 1
    seen.set(point.title, order)

    // 同名节点优先用父级标题区分，并追加序号保证同一父级下也不会重名
    const parent = point.parent_id ? byId.get(point.parent_id) : undefined
    scopes.set(point.id, parent ? `${parent.title} #${order}` : `#${order}`)
  }

  return { paths, scopes }
}

/** 选中节点的一跳邻域：父节点、子节点与语义关联的另一端 */
function neighborIds(pointId: string | null): Set<string> {
  const result = new Set<string>()
  if (!pointId) return result

  const point = props.points.find((item) => item.id === pointId)
  if (!point) return result

  const visible = new Set(props.points.map((item) => item.id))
  if (point.parent_id && visible.has(point.parent_id)) result.add(point.parent_id)

  for (const item of props.points) {
    if (item.parent_id === pointId) result.add(item.id)
  }

  for (const relation of props.relations) {
    if (relation.source_id === pointId && visible.has(relation.target_id)) {
      result.add(relation.target_id)
    }
    if (relation.target_id === pointId && visible.has(relation.source_id)) {
      result.add(relation.source_id)
    }
  }

  return result
}

/** 聚焦集合：选中节点及其一跳邻域；孤立节点不做弱化 */
function focusIds(pointId: string | null): Set<string> | null {
  if (!pointId) return null

  const neighbors = neighborIds(pointId)
  if (neighbors.size === 0) return null

  return new Set([pointId, ...neighbors])
}

function buildNodes(): GraphFlowNode[] {
  const positions =
    props.layout === 'hierarchy'
      ? layoutHierarchy(props.points, props.relations)
      : layoutForce(props.points, props.relations)
  const counts = relationCounts()
  const { paths, scopes } = graphMeta()
  const active = focusIds(props.selectedId)

  return props.points.map((point, index) => ({
    id: point.id,
    type: 'knowledge',
    position: positions.get(point.id) ?? { x: index * 240, y: 0 },
    data: {
      point,
      relationCount: counts.get(point.id) ?? 0,
      dimmed: active ? !active.has(point.id) : false,
      scope: scopes.get(point.id) ?? null,
      pathLabel: paths.get(point.id) ?? point.title,
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

function buildEdges(): Edge<GraphEdgeData>[] {
  const result: Edge<GraphEdgeData>[] = []
  const ids = new Set(props.points.map((point) => point.id))
  const selectedId = props.selectedId
  // 与节点一致：孤立节点选中时不弱化任何连线
  const focused = focusIds(selectedId) !== null

  if (props.showHierarchyEdges) {
    for (const point of props.points) {
      if (!point.parent_id || !ids.has(point.parent_id)) continue

      const highlighted =
        selectedId !== null && (point.parent_id === selectedId || point.id === selectedId)

      result.push({
        id: `hierarchy:${point.parent_id}:${point.id}`,
        source: point.parent_id,
        target: point.id,
        type: 'graph',
        data: {
          kind: 'hierarchy',
          highlighted,
          dimmed: focused && !highlighted,
        },
        style: {
          stroke: highlighted ? 'var(--color-accent)' : 'var(--color-border-strong)',
          strokeWidth: highlighted ? 1.8 : 1.3,
          strokeDasharray: '5 4',
        },
        selectable: false,
        focusable: false,
        zIndex: highlighted ? 2 : 0,
      })
    }
  }

  // 同一对节点（含双向与自环）之间的语义边分组，多条边由路由层分散到不同弧线
  const groups = new Map<string, RelationDetail[]>()
  for (const relation of props.relations) {
    if (!ids.has(relation.source_id) || !ids.has(relation.target_id)) continue
    const key = [relation.source_id, relation.target_id].sort().join('|')
    const group = groups.get(key)
    if (group) group.push(relation)
    else groups.set(key, [relation])
  }

  for (const group of groups.values()) {
    group.forEach((relation, parallelIndex) => {
      const highlighted =
        selectedId !== null &&
        (relation.source_id === selectedId || relation.target_id === selectedId)
      // 标签按需显示：默认只显示选中节点的一跳关联，勾选后显示全部
      const labelText =
        props.showEdgeLabels || highlighted ? relation.relation_label : undefined

      result.push({
        id: `relation:${relation.id}`,
        source: relation.source_id,
        target: relation.target_id,
        type: 'graph',
        data: {
          kind: 'relation',
          highlighted,
          dimmed: focused && !highlighted,
          labelText,
          parallelIndex,
          parallelCount: group.length,
        },
        labelBgPadding: [5, 3],
        labelBgBorderRadius: 4,
        labelBgStyle: { fill: 'var(--color-surface)', fillOpacity: 0.94 },
        labelStyle: {
          fill: 'var(--color-text-secondary)',
          fontSize: highlighted ? '10.5px' : '10px',
          fontWeight: 600,
        },
        style: {
          stroke: relation.relation_color,
          strokeWidth: highlighted ? 2.6 : 1.8,
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
        zIndex: highlighted ? 3 : 1,
      })
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

// 只有节点集合或布局方式变化才重排；关联筛选变化仅更新计数与连线，
// 保留当前节点位置（包括用户手动拖拽的结果）
watch(
  [() => props.points, () => props.layout],
  rebuildGraph,
)

watch(
  () => props.relations,
  () => {
    const counts = relationCounts()
    for (const node of nodes.value) {
      if (node.data) {
        node.data = { ...node.data, relationCount: counts.get(node.id) ?? 0 }
      }
    }
    rebuildEdges()
  },
)

watch(
  [() => props.showHierarchyEdges, () => props.showEdgeLabels],
  rebuildEdges,
)

watch(
  () => props.selectedId,
  (selectedId) => {
    const active = focusIds(selectedId)

    for (const node of nodes.value) {
      node.selected = node.id === selectedId
      // 整体替换 data，确保 Vue Flow 内部的节点组件能收到更新
      if (node.data) {
        node.data = { ...node.data, dimmed: active ? !active.has(node.id) : false }
      }
    }

    // 标签与弱化状态依赖选中项，重建边的成本远低于重排节点
    rebuildEdges()
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
      :edge-types="edgeTypes"
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
  /* 重新布局 / 切换布局时节点平滑移动；拖拽时禁用以免跟手延迟 */
  transition: transform 320ms ease;
}

.graph-canvas :deep(.vue-flow__node.dragging) {
  transition: none;
}

.graph-canvas :deep(.vue-flow__node.selected) {
  box-shadow: none;
}

.graph-canvas :deep(.vue-flow__edge-path) {
  stroke-linecap: round;
  transition:
    opacity 160ms ease,
    stroke 160ms ease,
    stroke-width 160ms ease;
}

.graph-canvas :deep(.vue-flow__edge-textbg) {
  transition: opacity 160ms ease;
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
