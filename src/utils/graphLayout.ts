import type { GraphPoint, RelationDetail } from '@/types'

/** 图谱节点的固定绘制尺寸，布局计算与边锚点都依赖它 */
export const GRAPH_NODE_WIDTH = 200
export const GRAPH_NODE_HEIGHT = 66

/** 节点左上角坐标 */
export interface GraphPosition {
  x: number
  y: number
}

const LEVEL_GAP = 74
const SIBLING_GAP = 26
const ROOT_GAP = 46

/**
 * 层次布局：按 parent_id 生成从左到右的分层树。
 *
 * 与自上而下的组织图相比，横向布局把「层级」放在横轴、「同级兄弟」放在纵轴，
 * 更适合宽屏画布，也能显著减少多门顶层知识点并排时的横向跨度。
 * 每个子树先测量自身高度，再在父节点中心线上纵向排布子节点；
 * 数据异常（父节点缺失 / 成环）时退化为顶层节点，保证节点不丢失。
 */
export function layoutHierarchy(points: GraphPoint[]): Map<string, GraphPosition> {
  const positions = new Map<string, GraphPosition>()
  if (points.length === 0) return positions

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

  const measured = new Map<string, number>()
  const measuring = new Set<string>()

  function measure(id: string): number {
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
  x: number
  y: number
}

interface ForceEdge {
  source: string
  target: string
  weight: number
  rest: number
}

/**
 * 力导向布局：层级边与语义边共同作为弹簧，所有节点互相排斥。
 *
 * 节点初始位置由知识点 ID 的哈希决定，因此刷新 / 筛选后的布局稳定；
 * 迭代次数随节点数下降，避免大课程阻塞界面。最终坐标以节点中心为准，
 * 返回时转换为左上角坐标。
 */
export function layoutForce(
  points: GraphPoint[],
  relations: RelationDetail[],
): Map<string, GraphPosition> {
  const positions = new Map<string, GraphPosition>()
  if (points.length === 0) return positions

  const index = new Map(points.map((point, order) => [point.id, order]))
  const count = points.length
  const radius = Math.max(160, Math.sqrt(count) * 130)

  const nodes: ForceNode[] = points.map((point) => {
    const angle = seededRandom(point.id, 1) * Math.PI * 2
    const distance = radius * (0.35 + seededRandom(point.id, 2) * 0.65)
    return {
      id: point.id,
      x: Math.cos(angle) * distance,
      y: Math.sin(angle) * distance,
    }
  })

  const edges: ForceEdge[] = []

  for (const point of points) {
    if (!point.parent_id) continue
    if (!index.has(point.parent_id) || point.parent_id === point.id) continue
    edges.push({
      source: point.parent_id,
      target: point.id,
      weight: 0.55,
      rest: 150,
    })
  }

  const seen = new Set<string>()
  for (const relation of relations) {
    if (!index.has(relation.source_id) || !index.has(relation.target_id)) continue
    const key = [relation.source_id, relation.target_id].sort().join('|')
    if (seen.has(key)) continue
    seen.add(key)
    edges.push({
      source: relation.source_id,
      target: relation.target_id,
      weight: 0.85,
      rest: 220,
    })
  }

  if (count === 1) {
    positions.set(points[0].id, { x: -GRAPH_NODE_WIDTH / 2, y: -GRAPH_NODE_HEIGHT / 2 })
    return positions
  }

  const iterations = count > 160 ? 60 : count > 80 ? 90 : 130
  const idealDistance = 260
  const displacement = new Map<string, { x: number; y: number }>()

  for (let iteration = 0; iteration < iterations; iteration += 1) {
    for (const node of nodes) displacement.set(node.id, { x: 0, y: 0 })

    // 两两排斥
    for (let left = 0; left < nodes.length; left += 1) {
      for (let right = left + 1; right < nodes.length; right += 1) {
        const a = nodes[left]
        const b = nodes[right]
        let dx = a.x - b.x
        let dy = a.y - b.y
        let distance = Math.hypot(dx, dy)

        if (distance < 0.01) {
          dx = 1
          dy = 1
          distance = 1.414
        }

        const force = Math.min(
          90,
          ((idealDistance * idealDistance) / distance) * 0.035,
        )
        const ux = dx / distance
        const uy = dy / distance
        const aDisp = displacement.get(a.id)!
        const bDisp = displacement.get(b.id)!
        aDisp.x += ux * force
        aDisp.y += uy * force
        bDisp.x -= ux * force
        bDisp.y -= uy * force
      }
    }

    // 边弹簧
    for (const edge of edges) {
      const a = nodes[index.get(edge.source)!]
      const b = nodes[index.get(edge.target)!]
      const dx = b.x - a.x
      const dy = b.y - a.y
      const distance = Math.max(0.01, Math.hypot(dx, dy))
      const force = (distance - edge.rest) * 0.16 * edge.weight
      const ux = dx / distance
      const uy = dy / distance
      const aDisp = displacement.get(a.id)!
      const bDisp = displacement.get(b.id)!
      aDisp.x += ux * force
      aDisp.y += uy * force
      bDisp.x -= ux * force
      bDisp.y -= uy * force
    }

    // 向心力，避免孤立节点飘走
    for (const node of nodes) {
      const disp = displacement.get(node.id)!
      disp.x -= node.x * 0.05
      disp.y -= node.y * 0.05
    }

    const temperature = Math.max(1.2, 16 * (1 - iteration / iterations))

    for (const node of nodes) {
      const disp = displacement.get(node.id)!
      const length = Math.hypot(disp.x, disp.y) || 1
      const scale = Math.min(temperature, length) / length

      node.x += disp.x * scale
      node.y += disp.y * scale
    }
  }

  for (const node of nodes) {
    positions.set(node.id, {
      x: node.x - GRAPH_NODE_WIDTH / 2,
      y: node.y - GRAPH_NODE_HEIGHT / 2,
    })
  }

  // 力导向结果常常接近正方形，而图图画布更宽；适度压缩纵向跨度，
  // 让节点在宽屏上保持更大字号，同时不改变拓扑与相对位置。
  const xs = nodes.map((node) => node.x)
  const ys = nodes.map((node) => node.y)
  const width = Math.max(...xs) - Math.min(...xs)
  const height = Math.max(...ys) - Math.min(...ys)
  const centerY = (Math.max(...ys) + Math.min(...ys)) / 2
  const targetAspect = 1.7

  if (width > 0 && height > 0 && width / height < targetAspect) {
    const scaleY = width / targetAspect / height
    for (const node of nodes) {
      positions.set(node.id, {
        x: node.x - GRAPH_NODE_WIDTH / 2,
        y: centerY + (node.y - centerY) * scaleY - GRAPH_NODE_HEIGHT / 2,
      })
    }
  }

  return positions
}
