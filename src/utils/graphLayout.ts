import type { GraphPoint, RelationDetail } from '@/types'

/** 图谱节点的固定绘制尺寸，布局计算与边锚点都依赖它 */
export const GRAPH_NODE_WIDTH = 200
export const GRAPH_NODE_HEIGHT = 66

/** 节点左上角坐标 */
export interface GraphPosition {
  x: number
  y: number
}

/** 层次布局：相邻层之间的水平间距，需要容纳父子连线的水平贝塞尔 */
const LEVEL_GAP = 110
/** 层次布局：兄弟子树之间的纵向间距 */
const SIBLING_GAP = 30
/** 层次布局：不同顶层子树之间的纵向间距 */
const ROOT_GAP = 60

interface HierarchyTree {
  byId: Map<string, GraphPoint>
  children: Map<string, GraphPoint[]>
  roots: GraphPoint[]
}

/**
 * 按 parent_id 构建森林。
 * 父节点缺失、自引用、成环等异常数据中的节点退化为顶层节点，保证不丢节点。
 */
function buildTree(points: GraphPoint[]): HierarchyTree {
  const byId = new Map(points.map((point) => [point.id, point]))
  const children = new Map<string, GraphPoint[]>()
  const roots: GraphPoint[] = []

  for (const point of points) {
    if (point.parent_id && byId.has(point.parent_id) && point.parent_id !== point.id) {
      const list = children.get(point.parent_id)
      if (list) list.push(point)
      else children.set(point.parent_id, [point])
    } else {
      roots.push(point)
    }
  }

  return { byId, children, roots }
}

/** 子树高度测量：叶子为节点高，内部节点为各子树高度之和加兄弟间距 */
function createMeasurer(children: Map<string, GraphPoint[]>) {
  const measured = new Map<string, number>()
  const measuring = new Set<string>()

  return function measure(id: string): number {
    const cached = measured.get(id)
    if (cached !== undefined) return cached

    // 成环保护：环上的节点按叶子高度参与计算，不会递归爆栈
    if (measuring.has(id)) return GRAPH_NODE_HEIGHT
    measuring.add(id)

    const kids = children.get(id) ?? []
    let height = GRAPH_NODE_HEIGHT

    if (kids.length) {
      const childrenHeight =
        kids.reduce((sum, child) => sum + measure(child.id), 0) +
        (kids.length - 1) * SIBLING_GAP
      height = Math.max(GRAPH_NODE_HEIGHT, childrenHeight)
    }

    measuring.delete(id)
    measured.set(id, height)
    return height
  }
}

/**
 * 层次布局：按 parent_id 生成从左到右的分层树。
 *
 * 布局分两步：
 * 1. 紧凑树形放置：每个子树先测量自身高度，父节点垂直居中于子节点区间，
 *    多棵顶层子树纵向堆叠，保证任何情况下节点矩形互不重叠；
 * 2. 重心排序（barycenter）：根据语义关联另一端的纵向位置对兄弟节点与
 *    顶层节点重排，迭代数轮，显著减少跨层语义连线的交叉。
 *
 * 数据异常（父节点缺失 / 成环）时退化为顶层节点，保证节点不丢失；
 * 排序与放置均为确定性计算，同一份数据每次结果一致。
 */
export function layoutHierarchy(
  points: GraphPoint[],
  relations: RelationDetail[] = [],
): Map<string, GraphPosition> {
  if (points.length === 0) return new Map()

  const { children, roots } = buildTree(points)
  const measure = createMeasurer(children)

  function placeAll(): Map<string, GraphPosition> {
    const positions = new Map<string, GraphPosition>()
    const placed = new Set<string>()

    function place(point: GraphPoint, top: number, level: number): void {
      if (placed.has(point.id)) return
      placed.add(point.id)

      const height = measure(point.id)
      const x = level * (GRAPH_NODE_WIDTH + LEVEL_GAP)
      const y = top + (height - GRAPH_NODE_HEIGHT) / 2
      positions.set(point.id, { x, y })

      const kids = children.get(point.id) ?? []
      if (!kids.length) return

      const totalHeight =
        kids.reduce((sum, child) => sum + measure(child.id), 0) +
        (kids.length - 1) * SIBLING_GAP
      let cursor = top + (height - totalHeight) / 2

      for (const child of kids) {
        place(child, cursor, level + 1)
        cursor += measure(child.id) + SIBLING_GAP
      }
    }

    let cursor = 0
    for (const root of roots) {
      place(root, cursor, 0)
      cursor += measure(root.id) + ROOT_GAP
    }

    // 兜底：成环等异常数据中的节点作为顶层节点补齐
    for (const point of points) {
      if (placed.has(point.id)) continue
      place(point, cursor, 0)
      cursor += measure(point.id) + ROOT_GAP
    }

    return positions
  }

  let positions = placeAll()
  if (relations.length === 0) return positions

  // 语义关联邻接表（只保留两端都在图内、非自环的边）
  const ids = new Set(points.map((point) => point.id))
  const neighbors = new Map<string, string[]>()
  for (const relation of relations) {
    if (relation.source_id === relation.target_id) continue
    if (!ids.has(relation.source_id) || !ids.has(relation.target_id)) continue
    const a = neighbors.get(relation.source_id)
    if (a) a.push(relation.target_id)
    else neighbors.set(relation.source_id, [relation.target_id])
    const b = neighbors.get(relation.target_id)
    if (b) b.push(relation.source_id)
    else neighbors.set(relation.target_id, [relation.source_id])
  }

  const centerY = (id: string): number => {
    const position = positions.get(id)
    return position ? position.y + GRAPH_NODE_HEIGHT / 2 : 0
  }

  // 重心排序键：语义邻居的纵向中心均值；无邻居时用自身位置，排序后保持不动
  const sortKey = (point: GraphPoint): number => {
    const linked = neighbors.get(point.id)
    if (!linked || linked.length === 0) return centerY(point.id)
    return linked.reduce((sum, id) => sum + centerY(id), 0) / linked.length
  }

  for (let round = 0; round < 3; round += 1) {
    const ordered = (list: GraphPoint[]): GraphPoint[] =>
      list
        .map((point, index) => ({ point, index, key: sortKey(point) }))
        .sort((a, b) => a.key - b.key || a.index - b.index)
        .map((entry) => entry.point)

    roots.splice(0, roots.length, ...ordered(roots))
    for (const [id, kids] of children) {
      if (kids.length > 1) children.set(id, ordered(kids))
    }

    positions = placeAll()
  }

  return positions
}

/** 稳定的 0~1 伪随机数，保证同一份数据每次布局结果一致 */
function seededRandom(id: string, salt: number): number {
  let hash = 2166136261
  const text = `${salt}:${id}`

  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index)
    hash = Math.imul(hash, 16777619)
  }

  return ((hash >>> 0) % 100000) / 100000
}

interface ForceNode {
  id: string
  /** 节点中心坐标 */
  x: number
  y: number
}

interface ForceEdge {
  source: number
  target: number
  weight: number
  rest: number
}

/** 碰撞检测时节点矩形向外膨胀的边距，保证节点之间留有视觉间隙 */
const COLLISION_MARGIN = 16

/**
 * 把仍然重叠的节点矩形沿最小平移轴分开。
 * 力导向迭代收敛后仍可能有个别重叠，这里做确定性的兜底分离。
 */
function separateOverlaps(nodes: ForceNode[]): void {
  const halfW = GRAPH_NODE_WIDTH / 2 + COLLISION_MARGIN
  const halfH = GRAPH_NODE_HEIGHT / 2 + COLLISION_MARGIN

  for (let round = 0; round < 24; round += 1) {
    let overlapped = false

    for (let left = 0; left < nodes.length; left += 1) {
      for (let right = left + 1; right < nodes.length; right += 1) {
        const a = nodes[left]
        const b = nodes[right]
        const dx = b.x - a.x
        const dy = b.y - a.y
        const overlapX = halfW * 2 - Math.abs(dx)
        const overlapY = halfH * 2 - Math.abs(dy)

        if (overlapX <= 0 || overlapY <= 0) continue
        overlapped = true

        if (overlapX <= overlapY) {
          // 沿水平方向分开；完全重合时用确定性的微小偏移打破对称
          const direction = dx !== 0 ? Math.sign(dx) : left % 2 === 0 ? 1 : -1
          const push = overlapX / 2 + 0.5
          a.x -= direction * push
          b.x += direction * push
        } else {
          const direction = dy !== 0 ? Math.sign(dy) : left % 2 === 0 ? 1 : -1
          const push = overlapY / 2 + 0.5
          a.y -= direction * push
          b.y += direction * push
        }
      }
    }

    if (!overlapped) return
  }
}

/**
 * 力导向布局：层级边与语义边共同作为弹簧，节点按矩形尺寸互相排斥。
 *
 * 与旧实现的关键差异：
 * - 初始位置来自层次布局（叠加确定性微抖动），结构稳定、收敛快，
 *   从层次布局切换到关系网络时视觉上也更连贯；
 * - 排斥力之外叠加矩形碰撞力，节点不再被当作质点，不会发生卡片重叠；
 * - 迭代结束后再做一次重叠分离兜底，并移除了旧实现事后压缩纵轴
 *   （会把已排好的节点重新压重叠）的做法。
 */
export function layoutForce(
  points: GraphPoint[],
  relations: RelationDetail[],
): Map<string, GraphPosition> {
  const positions = new Map<string, GraphPosition>()
  if (points.length === 0) return positions

  if (points.length === 1) {
    positions.set(points[0].id, { x: -GRAPH_NODE_WIDTH / 2, y: -GRAPH_NODE_HEIGHT / 2 })
    return positions
  }

  const count = points.length
  const index = new Map(points.map((point, order) => [point.id, order]))
  const initial = layoutHierarchy(points)

  const nodes: ForceNode[] = points.map((point) => {
    const base = initial.get(point.id) ?? { x: 0, y: 0 }
    return {
      id: point.id,
      x: base.x + GRAPH_NODE_WIDTH / 2 + (seededRandom(point.id, 1) - 0.5) * 40,
      y: base.y + GRAPH_NODE_HEIGHT / 2 + (seededRandom(point.id, 2) - 0.5) * 40,
    }
  })

  const edges: ForceEdge[] = []

  for (const point of points) {
    if (!point.parent_id) continue
    if (!index.has(point.parent_id) || point.parent_id === point.id) continue
    edges.push({
      source: index.get(point.parent_id)!,
      target: index.get(point.id)!,
      weight: 0.6,
      rest: GRAPH_NODE_WIDTH + 90,
    })
  }

  const seen = new Set<string>()
  for (const relation of relations) {
    if (!index.has(relation.source_id) || !index.has(relation.target_id)) continue
    if (relation.source_id === relation.target_id) continue
    const key = [relation.source_id, relation.target_id].sort().join('|')
    if (seen.has(key)) continue
    seen.add(key)
    edges.push({
      source: index.get(relation.source_id)!,
      target: index.get(relation.target_id)!,
      weight: 0.9,
      rest: GRAPH_NODE_WIDTH + 130,
    })
  }

  const iterations = count > 160 ? 140 : count > 80 ? 200 : 260
  const idealDistance = 250
  const halfW = GRAPH_NODE_WIDTH / 2 + COLLISION_MARGIN
  const halfH = GRAPH_NODE_HEIGHT / 2 + COLLISION_MARGIN
  const dispX = new Float64Array(count)
  const dispY = new Float64Array(count)

  for (let iteration = 0; iteration < iterations; iteration += 1) {
    dispX.fill(0)
    dispY.fill(0)

    // 两两排斥（长程库仑力）+ 矩形碰撞力
    for (let left = 0; left < count; left += 1) {
      for (let right = left + 1; right < count; right += 1) {
        const a = nodes[left]
        const b = nodes[right]
        let dx = b.x - a.x
        let dy = b.y - a.y
        let distance = Math.hypot(dx, dy)

        if (distance < 0.01) {
          dx = 1
          dy = 1
          distance = Math.SQRT2
        }

        const ux = dx / distance
        const uy = dy / distance

        const repulsion = Math.min(60, ((idealDistance * idealDistance) / distance) * 0.02)
        dispX[left] -= ux * repulsion
        dispY[left] -= uy * repulsion
        dispX[right] += ux * repulsion
        dispY[right] += uy * repulsion

        // 矩形重叠：沿最小重叠轴施加额外的强碰撞力，把卡片当作有体积的矩形
        const overlapX = halfW * 2 - Math.abs(dx)
        const overlapY = halfH * 2 - Math.abs(dy)
        if (overlapX > 0 && overlapY > 0) {
          if (overlapX <= overlapY) {
            const push = overlapX * 0.3 * Math.sign(dx || 1)
            dispX[left] -= push
            dispX[right] += push
          } else {
            const push = overlapY * 0.3 * Math.sign(dy || 1)
            dispY[left] -= push
            dispY[right] += push
          }
        }
      }
    }

    // 边弹簧
    for (const edge of edges) {
      const a = nodes[edge.source]
      const b = nodes[edge.target]
      const dx = b.x - a.x
      const dy = b.y - a.y
      const distance = Math.max(0.01, Math.hypot(dx, dy))
      const force = (distance - edge.rest) * 0.05 * edge.weight
      const ux = (dx / distance) * force
      const uy = (dy / distance) * force
      dispX[edge.source] += ux
      dispY[edge.source] += uy
      dispX[edge.target] -= ux
      dispY[edge.target] -= uy
    }

    // 向心力，避免孤立节点飘走
    for (let i = 0; i < count; i += 1) {
      dispX[i] -= nodes[i].x * 0.015
      dispY[i] -= nodes[i].y * 0.015
    }

    const temperature = Math.max(1, 30 * (1 - iteration / iterations))

    for (let i = 0; i < count; i += 1) {
      const length = Math.hypot(dispX[i], dispY[i]) || 1
      const scale = Math.min(temperature, length) / length
      nodes[i].x += dispX[i] * scale
      nodes[i].y += dispY[i] * scale
    }
  }

  // 兜底：迭代收敛后仍重叠的矩形做确定性分离
  separateOverlaps(nodes)

  for (const node of nodes) {
    positions.set(node.id, {
      x: node.x - GRAPH_NODE_WIDTH / 2,
      y: node.y - GRAPH_NODE_HEIGHT / 2,
    })
  }

  return positions
}
