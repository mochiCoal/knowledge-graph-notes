import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { coursesApi } from '@/services/courses'
import { knowledgePointsApi, type KnowledgePointInput } from '@/services/knowledgePoints'
import type {
  Course,
  KnowledgePoint,
  KnowledgePointDeletion,
  KnowledgePointDetail,
  KnowledgePointNode,
} from '@/types'

function errorText(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}

/** 深度优先展开知识点树 */
function flattenNodes(nodes: KnowledgePointNode[]): KnowledgePointNode[] {
  const result: KnowledgePointNode[] = []

  for (const node of nodes) {
    result.push(node)
    result.push(...flattenNodes(node.children))
  }

  return result
}

function findNode(nodes: KnowledgePointNode[], id: string): KnowledgePointNode | null {
  for (const node of nodes) {
    if (node.id === id) return node
    const found = findNode(node.children, id)
    if (found) return found
  }

  return null
}

/**
 * 知识点管理数据
 *
 * - 课程列表、知识点树、详情全部来自本地 SQLite；
 * - 树由后端按 parent_id 组装，前端只负责展示与折叠状态；
 * - 增删改与层级调整后立即重新拉取树，保证层级与统计始终一致。
 */
export const useKnowledgePointsStore = defineStore('knowledgePoints', () => {
  const courses = ref<Course[]>([])
  /** 当前课程的知识点树 */
  const tree = ref<KnowledgePointNode[]>([])
  /** 全部知识点（用于统计每门课程的知识点数量） */
  const allPoints = ref<KnowledgePoint[]>([])
  const activeCourseId = ref('')
  const selectedId = ref<string | null>(null)
  const detail = ref<KnowledgePointDetail | null>(null)
  /** 用户手动折叠的节点，默认全部展开 */
  const collapsedIds = ref<Set<string>>(new Set())

  const isLoading = ref(false)
  const isLoadingDetail = ref(false)
  const errorMessage = ref<string | null>(null)
  const detailError = ref<string | null>(null)
  const initialized = ref(false)

  const activeCourse = computed(
    () => courses.value.find((course) => course.id === activeCourseId.value) ?? null,
  )

  const pointCountByCourse = computed(() => {
    const counts = new Map<string, number>()
    for (const point of allPoints.value) {
      counts.set(point.course_id, (counts.get(point.course_id) ?? 0) + 1)
    }
    return counts
  })

  /** 当前课程的知识点总数 */
  const totalCount = computed(() => flattenNodes(tree.value).length)

  const selectedNode = computed(() =>
    selectedId.value ? findNode(tree.value, selectedId.value) : null,
  )

  function pointCountOf(courseId: string): number {
    return pointCountByCourse.value.get(courseId) ?? 0
  }

  function resetState(): void {
    tree.value = []
    allPoints.value = []
    activeCourseId.value = ''
    selectedId.value = null
    detail.value = null
    collapsedIds.value = new Set()
  }

  /** 优先使用指定课程，其次保持当前课程，最后回退到第一门课程 */
  function resolveCourse(preferred?: string | null): Course | null {
    const byId = (id: string | null | undefined): Course | null =>
      id ? (courses.value.find((course) => course.id === id) ?? null) : null

    return byId(preferred) ?? byId(activeCourseId.value) ?? courses.value[0] ?? null
  }

  async function loadAll(): Promise<void> {
    if (!activeCourseId.value) {
      tree.value = []
      allPoints.value = await knowledgePointsApi.list()
      return
    }

    const [treeData, points] = await Promise.all([
      knowledgePointsApi.tree(activeCourseId.value),
      knowledgePointsApi.list(),
    ])

    tree.value = treeData
    allPoints.value = points

    // 选中的知识点可能刚被删除或迁移到其它课程
    if (selectedId.value && !findNode(tree.value, selectedId.value)) {
      selectedId.value = null
      detail.value = null
    }
  }

  /** 详情请求序号：避免快速切换知识点时旧响应覆盖新详情 */
  let detailRequest = 0

  async function loadDetail(pointId: string): Promise<void> {
    const requestId = ++detailRequest
    isLoadingDetail.value = true
    detailError.value = null

    try {
      const loaded = await knowledgePointsApi.detail(pointId)
      if (requestId !== detailRequest) return
      if (selectedId.value === pointId) {
        detail.value = loaded
      }
    } catch (error) {
      if (requestId !== detailRequest) return
      if (selectedId.value === pointId) {
        detail.value = null
        detailError.value = errorText(error)
      }
    } finally {
      if (requestId === detailRequest) {
        isLoadingDetail.value = false
      }
    }
  }

  /** 初始化：加载课程列表，选中目标课程并展开知识点树 */
  async function initialize(preferredCourseId?: string | null): Promise<void> {
    isLoading.value = true
    errorMessage.value = null

    try {
      courses.value = await coursesApi.list()
      const target = resolveCourse(preferredCourseId)

      if (!target) {
        resetState()
        return
      }

      if (target.id !== activeCourseId.value) {
        activeCourseId.value = target.id
        selectedId.value = null
        detail.value = null
        collapsedIds.value = new Set()
      }

      await loadAll()

      if (selectedId.value) {
        await loadDetail(selectedId.value)
      } else if (tree.value[0]) {
        await selectPoint(tree.value[0].id)
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
      initialized.value = true
    }
  }

  /** 切换课程：清空选中状态并加载该课程的知识点树 */
  async function selectCourse(courseId: string): Promise<void> {
    if (courseId === activeCourseId.value) return

    activeCourseId.value = courseId
    selectedId.value = null
    detail.value = null
    detailError.value = null
    collapsedIds.value = new Set()
    isLoading.value = true
    errorMessage.value = null

    try {
      await loadAll()
      if (tree.value[0]) {
        await selectPoint(tree.value[0].id)
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  /** 手动刷新（顶部刷新按钮 / 错误重试） */
  async function refresh(): Promise<void> {
    if (!activeCourseId.value) {
      await initialize()
      return
    }

    isLoading.value = true
    errorMessage.value = null

    try {
      await loadAll()
      if (selectedId.value) {
        await loadDetail(selectedId.value)
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  /** 操作成功后静默刷新：失败只记录错误，不打断已经完成的写操作 */
  async function syncAfterChange(): Promise<void> {
    try {
      await loadAll()
      // 层级 / 子节点统计可能已经变化，同步刷新当前选中知识点的详情
      if (selectedId.value) {
        await loadDetail(selectedId.value)
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    }
  }

  async function selectPoint(pointId: string): Promise<void> {
    selectedId.value = pointId
    await loadDetail(pointId)
  }

  function isExpanded(pointId: string): boolean {
    return !collapsedIds.value.has(pointId)
  }

  function toggleExpanded(pointId: string): void {
    const next = new Set(collapsedIds.value)
    if (next.has(pointId)) {
      next.delete(pointId)
    } else {
      next.add(pointId)
    }
    collapsedIds.value = next
  }

  function expand(pointId: string): void {
    if (!collapsedIds.value.has(pointId)) return
    const next = new Set(collapsedIds.value)
    next.delete(pointId)
    collapsedIds.value = next
  }

  /** 父知识点（顶层知识点返回 null） */
  function parentOf(pointId: string): KnowledgePointNode | null {
    const node = findNode(tree.value, pointId)
    if (!node?.parent_id) return null
    return findNode(tree.value, node.parent_id)
  }

  /** 同级节点（按树的展示顺序） */
  function siblingsOf(pointId: string): KnowledgePointNode[] {
    const parent = parentOf(pointId)
    return parent ? parent.children : tree.value
  }

  /** 同级中的前一个节点，用于「降为子级」 */
  function previousSibling(pointId: string): KnowledgePointNode | null {
    const siblings = siblingsOf(pointId)
    const index = siblings.findIndex((node) => node.id === pointId)
    return index > 0 ? siblings[index - 1] : null
  }

  /** 新建知识点（parentId 为空表示顶层） */
  async function createPoint(
    input: KnowledgePointInput,
  ): Promise<KnowledgePoint> {
    const point = await knowledgePointsApi.create(activeCourseId.value, input)

    if (input.parentId) expand(input.parentId)
    await syncAfterChange()
    await selectPoint(point.id)

    return point
  }

  /** 修改知识点（名称、描述与父节点） */
  async function updatePoint(
    pointId: string,
    input: KnowledgePointInput,
  ): Promise<KnowledgePoint> {
    const point = await knowledgePointsApi.update(pointId, input)

    if (input.parentId) expand(input.parentId)
    await syncAfterChange()

    return point
  }

  /** 调整层级（parentId 为空表示提升为顶层知识点） */
  async function movePoint(pointId: string, parentId: string | null): Promise<KnowledgePoint> {
    const point = await knowledgePointsApi.move(pointId, parentId)

    if (parentId) expand(parentId)
    await syncAfterChange()

    return point
  }

  /** 迁移到另一门课程，整棵子树跟随迁移 */
  async function movePointToCourse(pointId: string, courseId: string): Promise<KnowledgePoint> {
    const point = await knowledgePointsApi.moveToCourse(pointId, courseId)
    await syncAfterChange()
    return point
  }

  /** 删除知识点及其全部后代 */
  async function deletePoint(pointId: string): Promise<KnowledgePointDeletion> {
    const result = await knowledgePointsApi.remove(pointId)
    await syncAfterChange()
    return result
  }

  function dismissError(): void {
    errorMessage.value = null
  }

  function dismissDetailError(): void {
    detailError.value = null
  }

  return {
    // 数据
    courses,
    tree,
    allPoints,
    activeCourseId,
    activeCourse,
    // 选择与详情
    selectedId,
    selectedNode,
    detail,
    totalCount,
    // 状态
    collapsedIds,
    isLoading,
    isLoadingDetail,
    errorMessage,
    detailError,
    initialized,
    // 查询
    pointCountOf,
    parentOf,
    siblingsOf,
    previousSibling,
    isExpanded,
    // 行为
    initialize,
    selectCourse,
    selectPoint,
    refresh,
    createPoint,
    updatePoint,
    movePoint,
    movePointToCourse,
    deletePoint,
    toggleExpanded,
    expand,
    dismissError,
    dismissDetailError,
  }
})
