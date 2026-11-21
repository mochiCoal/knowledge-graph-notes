import { describe, expect, it } from 'vitest'

import { borderAnchor, boxCenter } from '@/utils/graphGeometry'

const box = { x: 0, y: 0, width: 200, height: 66 }

describe('borderAnchor', () => {
  it('目标在右侧时锚点落在右边中点', () => {
    const anchor = borderAnchor(box, { x: 1000, y: 33 })
    expect(anchor).toEqual({ x: 200, y: 33, position: 'right' })
  })

  it('目标在左侧时锚点落在左边中点', () => {
    const anchor = borderAnchor(box, { x: -1000, y: 33 })
    expect(anchor).toEqual({ x: 0, y: 33, position: 'left' })
  })

  it('目标在下方时锚点落在下边', () => {
    const anchor = borderAnchor(box, { x: 100, y: 1000 })
    expect(anchor).toEqual({ x: 100, y: 66, position: 'bottom' })
  })

  it('目标在上方时锚点落在上边', () => {
    const anchor = borderAnchor(box, { x: 100, y: -1000 })
    expect(anchor).toEqual({ x: 100, y: 0, position: 'top' })
  })

  it('斜向目标优先命中更近的边', () => {
    const anchor = borderAnchor(box, { x: 400, y: 233 })
    expect(anchor.position).toBe('bottom')
    expect(anchor.y).toBeCloseTo(66)
    expect(anchor.x).toBeCloseTo(149.5)
  })

  it('目标与中心重合时退化为右边中点，不产生 NaN', () => {
    const anchor = borderAnchor(box, boxCenter(box))
    expect(anchor).toEqual({ x: 200, y: 33, position: 'right' })
  })

  it('节点从左上角偏移时锚点使用绝对坐标', () => {
    const anchor = borderAnchor({ x: 500, y: 300, width: 200, height: 66 }, { x: -1000, y: 333 })
    expect(anchor.position).toBe('left')
    expect(anchor.x).toBeCloseTo(500)
    expect(anchor.y).toBeCloseTo(333)
  })
})

describe('boxCenter', () => {
  it('返回矩形中心', () => {
    expect(boxCenter(box)).toEqual({ x: 100, y: 33 })
  })
})