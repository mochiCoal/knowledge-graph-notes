import { borderAnchor, boxCenter, type GraphAnchor, type GraphBox } from './graphGeometry'

/** 边路由的输入：两端节点矩形 + 其余节点矩形（用于避障） */
export interface EdgeRouteInput {
  source: GraphBox
  target: GraphBox
  /** 自环边（源与目标是同一节点） */
  selfLoop?: boolean
  /** 同一对节点之间平行边的序号（从 0 开始）与总数，用于把多条边分散到不同弧线 */
  parallelIndex?: number
  parallelCount?: number
  /** 需要绕开的无关节点矩形 */
  obstacles?: GraphBox[]
}

export interface EdgeRoute {
  /** SVG path（三次贝塞尔或自环弧线） */
  path: string
  /** 关系标签的放置位置，取曲线中点 */
  labelX: number
  labelY: number
}

interface Vec {
  x: number
  y: number
}

/** 锚点所在边对应的外法线方向，贝塞尔控制点沿该方向伸出 */
function anchorDirection(anchor: GraphAnchor): Vec {
  switch (anchor.position) {
    case 'left':
      return { x: -1, y: 0 }
    case 'top':
      return { x: 0, y: -1 }
    case 'bottom':
      return { x: 0, y: 1 }
    default:
      return { x: 1, y: 0 }
  }
}

/** 三次贝塞尔插值 */
function cubicPoint(a: Vec, c1: Vec, c2: Vec, b: Vec, t: number): Vec {
  const mt = 1 - t
  const w0 = mt * mt * mt
  const w1 = 3 * mt * mt * t
  const w2 = 3 * mt * t * t
  const w3 = t * t * t
  return {
    x: w0 * a.x + w1 * c1.x + w2 * c2.x + w3 * b.x,
    y: w0 * a.y + w1 * c1.y + w2 * c2.y + w3 * b.y,
  }
}

/** 曲线是否穿过任一障碍矩形（采样检测，矩形向外膨胀出安全间隙） */
function pathHitsObstacle(a: Vec, c1: Vec, c2: Vec, b: Vec, obstacles: GraphBox[]): boolean {
  const padding = 10
  for (const t of [0.15, 0.3, 0.5, 0.7, 0.85]) {
    const p = cubicPoint(a, c1, c2, b, t)
    for (const box of obstacles) {
      if (
        p.x > box.x - padding &&
        p.x < box.x + box.width + padding &&
        p.y > box.y - padding &&
        p.y < box.y + box.height + padding
      ) {
        return true
      }
    }
  }
  return false
}

function routeSelfLoop(source: GraphBox, parallelIndex: number): EdgeRoute {
  const centerX = source.x + source.width / 2
  const top = source.y
  const spread = 26
  // 多条自环边时逐条抬高弧顶，避免完全重叠
  const lift = 64 + parallelIndex * 26

  const a = { x: centerX - spread, y: top }
  const b = { x: centerX + spread, y: top }
  const c1 = { x: a.x - spread * 1.6, y: top - lift }
  const c2 = { x: b.x + spread * 1.6, y: top - lift }

  return {
    path: `M ${a.x} ${a.y} C ${c1.x} ${c1.y}, ${c2.x} ${c2.y}, ${b.x} ${b.y}`,
    labelX: centerX,
    labelY: top - lift * 0.78,
  }
}

/**
 * 计算一条边的 SVG 路径与标签位置。
 *
 * 路由策略：
 * - 端点使用浮动锚点（中心连线与节点边框的交点），控制点沿锚点外法线伸出，
 *   因此箭头方向始终与节点边缘垂直，不会绕节点打转；
 * - 同一对节点之间的多条平行边按序号向法线两侧交替偏移，避免完全重叠；
 * - 若曲线穿过中间无关节点，则逐步加大弧高并尝试另一侧，直到避开或达到上限；
 * - 自环边在节点上方画出独立圆弧。
 */
export function routeEdge(input: EdgeRouteInput): EdgeRoute {
  const parallelIndex = input.parallelIndex ?? 0
  const parallelCount = Math.max(1, input.parallelCount ?? 1)

  if (input.selfLoop) {
    return routeSelfLoop(input.source, parallelIndex)
  }

  const sourceAnchor = borderAnchor(input.source, boxCenter(input.target))
  const targetAnchor = borderAnchor(input.target, boxCenter(input.source))

  const a = { x: sourceAnchor.x, y: sourceAnchor.y }
  const b = { x: targetAnchor.x, y: targetAnchor.y }
  const dirA = anchorDirection(sourceAnchor)
  const dirB = anchorDirection(targetAnchor)

  const dx = b.x - a.x
  const dy = b.y - a.y
  const distance = Math.max(1, Math.hypot(dx, dy))
  // 垂直于中心连线的单位法线，平行边与避障弧都沿它偏移
  const normal = { x: -dy / distance, y: dx / distance }
  const tension = Math.min(160, Math.max(36, distance * 0.45))

  // 平行边：以中间一条为基准向两侧对称分散
  let bend = parallelCount > 1 ? (parallelIndex - (parallelCount - 1) / 2) * 64 : 0

  const controlPoints = (offset: number): { c1: Vec; c2: Vec } => ({
    c1: {
      x: a.x + dirA.x * tension + normal.x * offset,
      y: a.y + dirA.y * tension + normal.y * offset,
    },
    c2: {
      x: b.x + dirB.x * tension + normal.x * offset,
      y: b.y + dirB.y * tension + normal.y * offset,
    },
  })

  const obstacles = input.obstacles ?? []
  if (obstacles.length > 0) {
    let { c1, c2 } = controlPoints(bend)
    if (pathHitsObstacle(a, c1, c2, b, obstacles)) {
      // 优先向两侧尝试逐级加大的弧高；都失败时保留最大弧高，至少减少穿越
      let resolved = false
      for (const magnitude of [100, 180, 280]) {
        for (const sign of [1, -1]) {
          const candidate = bend + sign * magnitude
          const points = controlPoints(candidate)
          if (!pathHitsObstacle(a, points.c1, points.c2, b, obstacles)) {
            bend = candidate
            c1 = points.c1
            c2 = points.c2
            resolved = true
            break
          }
        }
        if (resolved) break
        // 记录当前最大弧高作为兜底
        bend += magnitude
      }
    }
  }

  const { c1, c2 } = controlPoints(bend)
  const label = cubicPoint(a, c1, c2, b, 0.5)

  return {
    path: `M ${a.x} ${a.y} C ${c1.x} ${c1.y}, ${c2.x} ${c2.y}, ${b.x} ${b.y}`,
    labelX: label.x,
    labelY: label.y,
  }
}
