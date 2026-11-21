<script setup lang="ts">
import { computed } from 'vue'
import { BaseEdge, useVueFlow, type EdgeProps, type GraphNode } from '@vue-flow/core'
import type { GraphEdgeData } from '@/types'
import { routeEdge } from '@/utils/graphRouting'
import type { GraphBox } from '@/utils/graphGeometry'
import { GRAPH_NODE_HEIGHT, GRAPH_NODE_WIDTH } from '@/utils/graphLayout'

const props = defineProps<EdgeProps<GraphEdgeData>>()

const { nodes } = useVueFlow()

/** 从 Vue Flow 的运行时节点读取绝对位置与实测尺寸，首帧未测量时退回常量尺寸 */
function nodeBox(node: GraphNode): GraphBox {
  return {
    x: node.computedPosition?.x ?? node.position?.x ?? 0,
    y: node.computedPosition?.y ?? node.position?.y ?? 0,
    width: node.dimensions?.width || GRAPH_NODE_WIDTH,
    height: node.dimensions?.height || GRAPH_NODE_HEIGHT,
  }
}

/**
 * 统一边路由：浮动锚点 + 平行边分散 + 自环弧线 + 无关节点避障。
 * 所有几何计算集中在 utils/graphRouting，组件只负责把结果交给 BaseEdge。
 */
const route = computed(() => {
  // 依赖 Vue Flow 传入的端点坐标，节点拖拽时强制重算节点位置
  void props.sourceX
  void props.sourceY
  void props.targetX
  void props.targetY

  const selfLoop = props.source === props.target
  const obstacles = selfLoop
    ? []
    : nodes.value
        .filter((node) => node.id !== props.source && node.id !== props.target)
        .map(nodeBox)

  return routeEdge({
    source: nodeBox(props.sourceNode),
    target: nodeBox(props.targetNode),
    selfLoop,
    parallelIndex: props.data?.parallelIndex ?? 0,
    parallelCount: props.data?.parallelCount ?? 1,
    obstacles,
  })
})

const path = computed(() => route.value.path)
const labelX = computed(() => route.value.labelX)
const labelY = computed(() => route.value.labelY)

const dimmed = computed(() => Boolean(props.data?.dimmed))

const edgeStyle = computed(() => ({
  ...(props.style ?? {}),
  opacity: dimmed.value ? 0.14 : 1,
}))

const labelStyle = computed(() => ({
  ...(props.labelStyle ?? {}),
  opacity: dimmed.value ? 0.2 : 1,
}))

const label = computed(() => props.data?.labelText)
const markerEnd = computed(() => (dimmed.value ? undefined : props.markerEnd || undefined))
</script>

<template>
  <BaseEdge
    :id="id"
    :path="path"
    :marker-end="markerEnd"
    :style="edgeStyle"
    :label="label"
    :label-x="labelX"
    :label-y="labelY"
    :label-style="labelStyle"
    :label-bg-style="labelBgStyle"
    :label-bg-padding="labelBgPadding"
    :label-bg-border-radius="labelBgBorderRadius"
    :interaction-width="interactionWidth"
  />
</template>
