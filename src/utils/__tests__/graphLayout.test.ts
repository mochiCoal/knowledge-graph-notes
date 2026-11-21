import { describe, expect, it } from 'vitest'

import type { GraphPoint, RelationDetail } from '@/types'
import {
  GRAPH_NODE_HEIGHT,
  GRAPH_NODE_WIDTH,
  layoutForce,
  layoutHierarchy,
} from '@/utils/graphLayout'

function point(id: string, parentId: string | null = null): GraphPoint {
  return {
    id,
    course_id: 'c1',
    parent_id: parentId,
    title: id,
    description: '',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    depth: 0,
    child_count: 0,
    note_count: 0,
  }
}

function relation(sourceId: string, targetId: string): RelationDetail {
  return {
    id: `${sourceId}-${targetId}`,
    source_id: sourceId,
    target_id: targetId,
    relation_type: 'related',
    description: '',
    created_at: '2026-01-01T00:00:00Z',
    source_title: sourceId,
    target_title: targetId,
    source_course_id: 'c1',
    source_course_name: '课程',
    target_course_id: 'c1',
    target_course_name: '课程',
    relation_label: '相关知识',
    relation_color: '#2f8f5b',
    directed: false,
  }
}

function expectFinitePositions(positions: Map<string, { x: number; y: number }>, ids: string[]): void {
  expect(positions.size).toBe(ids.length)
  for (const id of ids) {
    const position = positions.get(id)
    expect(position, `节点 ${id} 应有位置`).toBeDefined()
    expect(Number.isFinite(position!.x)).toBe(true)
    expect(Number.isFinite(position!.y)).toBe(true)
  }
}

/** 任意两个节点矩形都不允许重叠（允许边缘恰好相接） */
function expectNoOverlap(positions: Map<string, { x: number; y: number }>, ids: string[]): void {
  for (let i = 0; i < ids.length; i += 1) {
    for (let j = i + 1; j < ids.length; j += 1) {
      const a = positions.get(ids[i])!
      const b = positions.get(ids[j])!
      const overlapX =
        Math.min(a.x + GRAPH_NODE_WIDTH, b.x + GRAPH_NODE_WIDTH) - Math.max(a.x, b.x)
      const overlapY =
        Math.min(a.y + GRAPH_NODE_HEIGHT, b.y + GRAPH_NODE_HEIGHT) - Math.max(a.y, b.y)
      expect(overlapX <= 0 || overlapY <= 0, `节点 ${ids[i]} 与 ${ids[j]} 不应重叠`).toBe(true)
    }
  }
}

describe('layoutHierarchy', () => {
  it('空数据返回空布局', () => {
    expect(layoutHierarchy([]).size).toBe(0)
  })

  it('父节点在左、子节点在右，兄弟节点纵向不重叠', () => {
    const positions = layoutHierarchy([
      point('root'),
      point('a', 'root'),
      point('b', 'root'),
      point('a1', 'a'),
    ])

    expectFinitePositions(positions, ['root', 'a', 'b', 'a1'])

    const levelGap = GRAPH_NODE_WIDTH + 110
    expect(positions.get('root')!.x).toBe(0)
    expect(positions.get('a')!.x).toBe(levelGap)
    expect(positions.get('b')!.x).toBe(levelGap)
    expect(positions.get('a1')!.x).toBe(levelGap * 2)

    const siblingsGap = Math.abs(positions.get('a')!.y - positions.get('b')!.y)
    expect(siblingsGap).toBeGreaterThanOrEqual(GRAPH_NODE_HEIGHT)
  })

  it('多个顶层节点都在第一层且纵向错开', () => {
    const positions = layoutHierarchy([point('r1'), point('r2')])
    expect(positions.get('r1')!.x).toBe(0)
    expect(positions.get('r2')!.x).toBe(0)
    expect(positions.get('r1')!.y).not.toBe(positions.get('r2')!.y)
  })

  it('父节点缺失、自引用与成环数据不会丢节点或死循环', () => {
    const positions = layoutHierarchy([
      point('orphan', 'missing'),
      point('self', 'self'),
      point('cycle-a', 'cycle-b'),
      point('cycle-b', 'cycle-a'),
    ])

    expectFinitePositions(positions, ['orphan', 'self', 'cycle-a', 'cycle-b'])
  })

  it('重复调用结果一致', () => {
    const points = [point('root'), point('a', 'root'), point('b', 'root')]
    expect(layoutHierarchy(points)).toEqual(layoutHierarchy(points))
  })

  it('多兄弟与多子树的节点矩形互不重叠（含语义边重心排序）', () => {
    const points = [
      point('r1'),
      point('r2'),
      point('a', 'r1'),
      point('b', 'r1'),
      point('c', 'r1'),
      point('a1', 'a'),
      point('a2', 'a'),
      point('b1', 'b'),
      point('a1x', 'a1'),
    ]
    const ids = points.map((item) => item.id)
    const positions = layoutHierarchy(points, [relation('a1x', 'b1'), relation('c', 'r2')])

    expectFinitePositions(positions, ids)
    expectNoOverlap(positions, ids)
  })
})

describe('layoutForce', () => {
  it('空数据返回空布局', () => {
    expect(layoutForce([], []).size).toBe(0)
  })

  it('单节点居中放置', () => {
    const positions = layoutForce([point('only')], [])
    expect(positions.get('only')).toEqual({
      x: -GRAPH_NODE_WIDTH / 2,
      y: -GRAPH_NODE_HEIGHT / 2,
    })
  })

  it('同一份数据每次布局结果完全一致', () => {
    const points = [point('root'), point('a', 'root'), point('b', 'root'), point('c')]
    const relations = [relation('a', 'c'), relation('b', 'c')]

    const first = layoutForce(points, relations)
    const second = layoutForce(points, relations)
    expect(first).toEqual(second)
    expectFinitePositions(first, ['root', 'a', 'b', 'c'])
  })

  it('忽略指向不存在节点的关联，坐标仍然有效', () => {
    const positions = layoutForce(
      [point('a'), point('b')],
      [relation('a', 'ghost'), relation('ghost', 'b'), relation('a', 'b')],
    )

    expectFinitePositions(positions, ['a', 'b'])
  })

  it('层级与语义关联混合时所有节点坐标有限', () => {
    const points = [
      point('root'),
      point('a', 'root'),
      point('b', 'root'),
      point('a1', 'a'),
      point('isolated'),
    ]
    const positions = layoutForce(points, [relation('a1', 'isolated'), relation('a', 'a')])

    expectFinitePositions(positions, ['root', 'a', 'b', 'a1', 'isolated'])
  })

  it('布局后节点矩形互不重叠', () => {
    const points = [
      point('root'),
      point('a', 'root'),
      point('b', 'root'),
      point('c', 'root'),
      point('a1', 'a'),
      point('a2', 'a'),
      point('b1', 'b'),
      point('free1'),
      point('free2'),
      point('free3'),
    ]
    const relations = [
      relation('a1', 'b1'),
      relation('free1', 'a'),
      relation('free2', 'c'),
      relation('a2', 'free3'),
    ]
    const ids = points.map((item) => item.id)
    const positions = layoutForce(points, relations)

    expectFinitePositions(positions, ids)
    expectNoOverlap(positions, ids)
  })
})
