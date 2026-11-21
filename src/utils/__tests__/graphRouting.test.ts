import { describe, expect, it } from 'vitest'

import { routeEdge } from '@/utils/graphRouting'
import type { GraphBox } from '@/utils/graphGeometry'

const boxA: GraphBox = { x: 0, y: 0, width: 200, height: 66 }
const boxB: GraphBox = { x: 500, y: 0, width: 200, height: 66 }

/** 从 path 中提取所有数值，验证没有 NaN / Infinity */
function pathNumbers(path: string): number[] {
  return (path.match(/-?\d+(\.\d+)?/g) ?? []).map(Number)
}

describe('routeEdge', () => {
  it('水平相邻节点：从右边中点连到左边中点，标签位于两端之间', () => {
    const route = routeEdge({ source: boxA, target: boxB })

    expect(route.path.startsWith('M 200 33 C')).toBe(true)
    expect(route.path.endsWith('500 33')).toBe(true)
    expect(route.labelX).toBeGreaterThan(200)
    expect(route.labelX).toBeLessThan(500)
    expect(route.labelY).toBeCloseTo(33)
    for (const value of pathNumbers(route.path)) {
      expect(Number.isFinite(value)).toBe(true)
    }
  })

  it('目标在正下方：从底边连到顶边', () => {
    const route = routeEdge({
      source: boxA,
      target: { x: 0, y: 400, width: 200, height: 66 },
    })

    expect(route.path.startsWith('M 100 66 C')).toBe(true)
    expect(route.path.endsWith('100 400')).toBe(true)
  })

  it('自环边：在节点上方画弧，标签位于节点之上', () => {
    const route = routeEdge({ source: boxA, target: boxA, selfLoop: true })

    expect(route.path).toContain('C')
    expect(route.labelY).toBeLessThan(0)
    for (const value of pathNumbers(route.path)) {
      expect(Number.isFinite(value)).toBe(true)
    }
  })

  it('多条平行边被分散到不同弧线，单条边保持直线对称', () => {
    const first = routeEdge({ source: boxA, target: boxB, parallelIndex: 0, parallelCount: 2 })
    const second = routeEdge({ source: boxA, target: boxB, parallelIndex: 1, parallelCount: 2 })
    const single = routeEdge({ source: boxA, target: boxB })

    expect(first.path).not.toBe(second.path)
    // 两条平行边向两侧对称偏移
    expect(first.labelY + second.labelY).toBeCloseTo(single.labelY * 2)
    expect(single.labelY).toBeCloseTo(33)
  })

  it('曲线穿过中间节点时加大弧避让', () => {
    const obstacle: GraphBox = { x: 280, y: -20, width: 200, height: 120 }
    const direct = routeEdge({ source: boxA, target: boxB })
    const avoided = routeEdge({ source: boxA, target: boxB, obstacles: [obstacle] })

    expect(avoided.path).not.toBe(direct.path)
    expect(Math.abs(avoided.labelY - direct.labelY)).toBeGreaterThan(30)
    for (const value of pathNumbers(avoided.path)) {
      expect(Number.isFinite(value)).toBe(true)
    }
  })

  it('同一份输入的路由结果完全一致', () => {
    const input = {
      source: boxA,
      target: { x: 320, y: 260, width: 200, height: 66 },
      parallelIndex: 1,
      parallelCount: 3,
      obstacles: [{ x: 240, y: 60, width: 200, height: 66 }],
    }
    expect(routeEdge(input)).toEqual(routeEdge(input))
  })
})
