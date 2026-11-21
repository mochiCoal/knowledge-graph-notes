/** 图谱节点的矩形区域（左上角坐标 + 宽高） */
export interface GraphBox {
  x: number
  y: number
  width: number
  height: number
}

/** 从节点边框上引出的锚点，position 表示锚点所在的那条边 */
export interface GraphAnchor {
  x: number
  y: number
  position: 'left' | 'right' | 'top' | 'bottom'
}

/**
 * 计算「从节点中心指向目标点」的射线与节点矩形的交点。
 *
 * Vue Flow 的默认连线依赖固定 handle，跨层语义边会从右侧出、左侧入，
 * 目标在左边时只能绕一大圈。浮动锚点让连线始终落在朝向对方的那条边上，
 * 既消除绕行，也让拖拽节点时连线贴身跟随。
 */
export function borderAnchor(box: GraphBox, toward: { x: number; y: number }): GraphAnchor {
  const width = Math.max(1, box.width)
  const height = Math.max(1, box.height)
  const centerX = box.x + width / 2
  const centerY = box.y + height / 2
  const dx = toward.x - centerX
  const dy = toward.y - centerY

  // 两点重合时没有方向，退化为右边中点，避免除零
  if (Math.abs(dx) < 0.001 && Math.abs(dy) < 0.001) {
    return { x: centerX + width / 2, y: centerY, position: 'right' }
  }

  const scaleX = Math.abs(dx) > 0.001 ? width / 2 / Math.abs(dx) : Number.POSITIVE_INFINITY
  const scaleY = Math.abs(dy) > 0.001 ? height / 2 / Math.abs(dy) : Number.POSITIVE_INFINITY
  const scale = Math.min(scaleX, scaleY)

  const horizontal = scaleX < scaleY
  const position = horizontal
    ? dx > 0
      ? 'right'
      : 'left'
    : dy > 0
      ? 'bottom'
      : 'top'

  return {
    x: centerX + dx * scale,
    y: centerY + dy * scale,
    position,
  }
}

/** 节点中心点 */
export function boxCenter(box: GraphBox): { x: number; y: number } {
  return {
    x: box.x + box.width / 2,
    y: box.y + box.height / 2,
  }
}