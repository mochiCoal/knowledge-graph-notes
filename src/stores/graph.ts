import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { coursesApi } from '@/services/courses'
import { knowledgePointsApi } from '@/services/knowledgePoints'
import { relationsApi } from '@/services/relations'
import type {
  Course,
  GraphLayoutMode,
  GraphPoint,
  KnowledgePointNode,
  RelationDetail,
  RelationType,
} from '@/types'

function errorText(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}

/** 把后端的知识点树扁平化，保留层级、子节点与笔记数量 */
function flattenTree(nodes: KnowledgePointNode[]): GraphPoint[] {
  const result: GraphPoint[] = []

  const walk = (items: KnowledgePointNode[]): void => {
    for (const item of items) {
      const { children, ...point } = item
      result.push(point)
      if (children.length) walk(children)
    }
  }

  walk(nodes)

  return result
}

/** 选中的知识点与它的一条关联 */
export interface SelectedRelationItem {
  relation: RelationDetail
  otherPoint: GraphPoint
  /** 当前知识点是否为关联的源点 */
  outgoing: boolean
}

/**
 * 知识图谱数据
 *
 * 数据全部来自 SQLite：知识点树 + 课程涉及的关联 + 关联类型。
 * 页面只维护筛选、布局与选中状态；节点位置由 `utils/graphLayout` 计算。
 */
export const useGraphStore = defineStore('graph', () => {
  const courses = ref<Course[]>([])
  const tree = ref<KnowledgePointNode[]>([])
  const relations = ref<RelationDetail[]>([])
  const types = ref<RelationType[]>([])

  const activeCourseId = ref<string | null>(null)
  const selectedPointId = ref<string | null>(null)
  const activeTypeIds = ref<string[]>([])
  const maxDepth = ref<number | null>(null)
  const layout = ref<GraphLayoutMode>('hierarchy')
  const showHierarchyEdges = ref(true)
  const showEdgeLabels = ref(true)

  const isLoading = ref(false)
  const isLoadingCourse = ref(false)
  const errorMessage = ref<string | null>(null)
  const initialized = ref(false)

  /** 用户是否亲手改过类型筛选：刷新时应保留，而不是重新全选 */
  let typeFilterTouched = false

  const activeCourse = computed(
    () => courses.value.find((course) => course.id === activeCourseId.value) ?? null,
  )

  const allPoints = computed(() => flattenTree(tree.value))

  const pointIndex = computed(() => {
    const map = new Map<string, GraphPoint>()
    for (const point of allPoints.value) map.set(point.id, point)
    return map
  })

  const maxTreeDepth = computed(() =>
    allPoints.value.reduce((max, point) => Math.max(max, point.depth), 0),
  )

  /** 当前图谱中真正绘制出来的知识点（按层级上限过滤） */
  const visiblePoints = computed(() => {
    const limit = maxDepth.value
    return limit === null
      ? allPoints.value
      : allPoints.value.filter((point) => point.depth <= limit)
  })

  const visibleIds = computed(() => new Set(visiblePoints.value.map((point) => point.id)))

  /** 只保留两端都在课程内、且类型未被过滤的语义关联 */
  const courseRelations = computed(() => {
    const ids = new Set(allPoints.value.map((point) => point.id))
    return relations.value.filter(
      (relation) => ids.has(relation.source_id) && ids.has(relation.target_id),
    )
  })

  const visibleRelations = computed(() => {
    if (activeTypeIds.value.length === 0) return []
    const ids = visibleIds.value
    const typeIds = new Set(activeTypeIds.value)
    return courseRelations.value.filter(
      (relation) =>
        typeIds.has(relation.relation_type) &&
        ids.has(relation.source_id) &&
        ids.has(relation.target_id),
    )
  })

  const relationCountByPoint = computed(() => {
    const counts = new Map<string, number>()
    for (const relation of courseRelations.value) {
      counts.set(relation.source_id, (counts.get(relation.source_id) ?? 0) + 1)
      counts.set(relation.target_id, (counts.get(relation.target_id) ?? 0) + 1)
    }
    return counts
  })

  const relationCountByType = computed(() => {
    const counts = new Map<string, number>()
    for (const relation of courseRelations.value) {
      counts.set(relation.relation_type, (counts.get(relation.relation_type) ?? 0) + 1)
    }
    return counts
  })

  const hierarchyEdgeCount = computed(() => {
    const ids = visibleIds.value
    return visiblePoints.value.filter(
      (point) => point.parent_id !== null && ids.has(point.parent_id),
    ).length
  })

  const selectedPoint = computed(() =>
    selectedPointId.value ? (pointIndex.value.get(selectedPointId.value) ?? null) : null,
  )

  const selectedPointPath = computed<GraphPoint[]>(() => {
    const path: GraphPoint[] = []
    const visited = new Set<string>()
    let current = selectedPoint.value

    while (current && !visited.has(current.id)) {
      visited.add(current.id)
      path.unshift(current)
      current = current.parent_id ? (pointIndex.value.get(current.parent_id) ?? null) : null
    }

    return path
  })

  const selectedRelations = computed<SelectedRelationItem[]>(() => {
    const current = selectedPoint.value
    if (!current) return []

    const items: SelectedRelationItem[] = []
    for (const relation of courseRelations.value) {
      const outgoing = relation.source_id === current.id
      const isTarget = relation.target_id === current.id
      if (!outgoing && !isTarget) continue

      const otherPoint = pointIndex.value.get(outgoing ? relation.target_id : relation.source_id)
      if (otherPoint) items.push({ relation, otherPoint, outgoing })
    }

    return items
  })

  const selectedRelationCount = computed(() => selectedRelations.value.length)

  function relationCountOf(pointId: string): number {
    return relationCountByPoint.value.get(pointId) ?? 0
  }

  function typeUsageOf(typeId: string): number {
    return relationCountByType.value.get(typeId) ?? 0
  }

  function isTypeActive(typeId: string): boolean {
    return activeTypeIds.value.includes(typeId)
  }

  function typeLabel(typeId: string): string {
    return types.value.find((type) => type.id === typeId)?.label ?? typeId
  }

  function applyTypes(nextTypes: RelationType[]): void {
    types.value = nextTypes

    if (!typeFilterTouched) {
      activeTypeIds.value = nextTypes.map((type) => type.id)
      return
    }

    // 用户已经手动筛选过：保留仍可用的选择
    const available = new Set(nextTypes.map((type) => type.id))
    activeTypeIds.value = activeTypeIds.value.filter((id) => available.has(id))
  }

  async function loadCourse(courseId: string, resetFilters = false): Promise<void> {
    isLoadingCourse.value = true
    errorMessage.value = null

    try {
      const [treeData, relationData, typeData] = await Promise.all([
        knowledgePointsApi.tree(courseId),
        relationsApi.listByCourse(courseId),
        relationsApi.listTypes(),
      ])

      activeCourseId.value = courseId
      tree.value = treeData
      relations.value = relationData
      applyTypes(typeData)
      if (resetFilters) maxDepth.value = null

      const points = flattenTree(treeData)
      selectedPointId.value = points[0]?.id ?? null
    } catch (error) {
      errorMessage.value = errorText(error)
      tree.value = []
      relations.value = []
      selectedPointId.value = null
    } finally {
      isLoadingCourse.value = false
    }
  }

  async function initialize(preferredCourseId: string | null = null): Promise<void> {
    isLoading.value = true
    errorMessage.value = null

    try {
      const [courseList, typeList] = await Promise.all([
        coursesApi.list(),
        relationsApi.listTypes(),
      ])

      courses.value = courseList
      applyTypes(typeList)

      const courseId =
        (preferredCourseId &&
        courseList.some((course) => course.id === preferredCourseId)
          ? preferredCourseId
          : null) ??
        (activeCourseId.value &&
        courseList.some((course) => course.id === activeCourseId.value)
          ? activeCourseId.value
          : null) ??
        courseList[0]?.id ??
        null

      if (courseId) {
        await loadCourse(courseId, true)
      } else {
        activeCourseId.value = null
        tree.value = []
        relations.value = []
        selectedPointId.value = null
      }

      initialized.value = true
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  async function selectCourse(courseId: string): Promise<void> {
    if (courseId === activeCourseId.value) return
    selectedPointId.value = null
    await loadCourse(courseId, true)
  }

  async function refresh(): Promise<void> {
    if (!activeCourseId.value) {
      await initialize()
      return
    }

    isLoading.value = true
    errorMessage.value = null

    try {
      const [courseList, typeList] = await Promise.all([
        coursesApi.list(),
        relationsApi.listTypes(),
      ])
      courses.value = courseList
      applyTypes(typeList)

      if (!courseList.some((course) => course.id === activeCourseId.value)) {
        const fallback = courseList[0]?.id ?? null
        if (fallback) {
          await loadCourse(fallback)
        } else {
          activeCourseId.value = null
          tree.value = []
          relations.value = []
          selectedPointId.value = null
        }
        return
      }

      const previousSelected = selectedPointId.value
      await loadCourse(activeCourseId.value)

      const previousPoint = previousSelected
        ? pointIndex.value.get(previousSelected)
        : undefined
      const previousVisible =
        previousPoint &&
        (maxDepth.value === null || previousPoint.depth <= maxDepth.value)

      if (previousVisible) {
        selectedPointId.value = previousSelected
      } else {
        selectedPointId.value = visiblePoints.value[0]?.id ?? null
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  function selectPoint(pointId: string | null): void {
    selectedPointId.value = pointId
  }

  /** 从详情面板点击关联知识点时，必要时自动放宽层级筛选 */
  function revealPoint(pointId: string): void {
    const point = pointIndex.value.get(pointId)
    if (!point) return

    if (maxDepth.value !== null && point.depth > maxDepth.value) {
      maxDepth.value = null
    }

    selectedPointId.value = pointId
  }

  function setMaxDepth(value: number | null): void {
    maxDepth.value = value

    const current = selectedPoint.value
    if (current && value !== null && current.depth > value) {
      selectedPointId.value = visiblePoints.value[0]?.id ?? null
    }
  }

  function setLayout(value: GraphLayoutMode): void {
    layout.value = value
  }

  function toggleType(typeId: string): void {
    typeFilterTouched = true
    activeTypeIds.value = isTypeActive(typeId)
      ? activeTypeIds.value.filter((id) => id !== typeId)
      : [...activeTypeIds.value, typeId]
  }

  function selectAllTypes(): void {
    typeFilterTouched = false
    activeTypeIds.value = types.value.map((type) => type.id)
  }

  function setShowHierarchyEdges(value: boolean): void {
    showHierarchyEdges.value = value
  }

  function setShowEdgeLabels(value: boolean): void {
    showEdgeLabels.value = value
  }

  function dismissError(): void {
    errorMessage.value = null
  }

  return {
    // 数据
    courses,
    tree,
    relations,
    types,
    activeCourseId,
    activeCourse,
    selectedPointId,
    selectedPoint,
    selectedPointPath,
    selectedRelations,
    allPoints,
    visiblePoints,
    visibleRelations,
    courseRelations,
    maxTreeDepth,
    maxDepth,
    layout,
    activeTypeIds,
    showHierarchyEdges,
    showEdgeLabels,
    // 统计
    relationCountByPoint,
    relationCountByType,
    hierarchyEdgeCount,
    selectedRelationCount,
    // 状态
    isLoading,
    isLoadingCourse,
    errorMessage,
    initialized,
    // 查询
    relationCountOf,
    typeUsageOf,
    isTypeActive,
    typeLabel,
    // 行为
    initialize,
    refresh,
    selectCourse,
    selectPoint,
    revealPoint,
    setMaxDepth,
    setLayout,
    toggleType,
    selectAllTypes,
    setShowHierarchyEdges,
    setShowEdgeLabels,
    dismissError,
  }
})
