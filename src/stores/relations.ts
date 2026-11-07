import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { coursesApi } from '@/services/courses'
import { knowledgePointsApi } from '@/services/knowledgePoints'
import {
  relationsApi,
  type CreateRelationInput,
  type RelationTypeInput,
  type UpdateRelationInput,
} from '@/services/relations'
import type {
  Course,
  KnowledgePoint,
  RelationDetail,
  RelationSuggestion,
  RelationType,
} from '@/types'

function errorText(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}

/** 关联建议加载数量：页面展示更多，笔记侧栏只用前几条 */
const PAGE_SUGGESTION_LIMIT = 8

/**
 * 知识点关联管理数据
 *
 * - 左侧按课程筛选知识点，中间展示当前知识点的正式关联，右侧展示推荐；
 * - 关联、建议、类型均在修改后立即刷新，保证多处入口数据一致；
 * - 建议由后端根据标题 / 描述 / 笔记文本计算，本地离线、可解释。
 */
export const useRelationsStore = defineStore('relations', () => {
  const courses = ref<Course[]>([])
  const allPoints = ref<KnowledgePoint[]>([])
  const relations = ref<RelationDetail[]>([])
  const types = ref<RelationType[]>([])
  const suggestions = ref<RelationSuggestion[]>([])

  const selectedCourseId = ref<string | null>(null)
  const selectedPointId = ref<string | null>(null)
  const typeFilter = ref('')
  const keyword = ref('')

  const isLoading = ref(false)
  const isLoadingRelations = ref(false)
  const isLoadingSuggestions = ref(false)
  const isMutating = ref(false)
  const errorMessage = ref<string | null>(null)
  const relationsError = ref<string | null>(null)
  const suggestionsError = ref<string | null>(null)
  const initialized = ref(false)

  /** 请求序号：避免快速切换课程 / 知识点时旧响应覆盖新数据 */
  let relationsRequest = 0
  let suggestionsRequest = 0

  const selectedCourse = computed(
    () => courses.value.find((course) => course.id === selectedCourseId.value) ?? null,
  )

  const pointIndex = computed(() => {
    const map = new Map<string, KnowledgePoint>()
    for (const point of allPoints.value) {
      map.set(point.id, point)
    }
    return map
  })

  const selectedPoint = computed(() =>
    selectedPointId.value ? (pointIndex.value.get(selectedPointId.value) ?? null) : null,
  )

  /** 当前课程下的知识点（按创建时间排序） */
  const coursePoints = computed(() => {
    if (!selectedCourseId.value) return []
    return allPoints.value.filter((point) => point.course_id === selectedCourseId.value)
  })

  /** 按关键字过滤后的知识点（标题 / 描述） */
  const filteredPoints = computed(() => {
    const value = keyword.value.trim().toLowerCase()
    if (!value) return coursePoints.value

    return coursePoints.value.filter(
      (point) =>
        point.title.toLowerCase().includes(value) ||
        point.description.toLowerCase().includes(value),
    )
  })

  /** 每个知识点涉及的关联数量 */
  const relationCountByPoint = computed(() => {
    const counts = new Map<string, number>()
    for (const relation of relations.value) {
      counts.set(relation.source_id, (counts.get(relation.source_id) ?? 0) + 1)
      counts.set(relation.target_id, (counts.get(relation.target_id) ?? 0) + 1)
    }
    return counts
  })

  const selectedPointRelations = computed(() => {
    const pointId = selectedPointId.value
    if (!pointId) return []

    return relations.value.filter((relation) => {
      const involvesPoint =
        relation.source_id === pointId || relation.target_id === pointId
      const matchesType = !typeFilter.value || relation.relation_type === typeFilter.value
      return involvesPoint && matchesType
    })
  })

  const totalRelations = computed(() => relations.value.length)

  function relationCountOf(pointId: string): number {
    return relationCountByPoint.value.get(pointId) ?? 0
  }

  function relationTypeOf(typeId: string): RelationType | null {
    return types.value.find((type) => type.id === typeId) ?? null
  }

  function relationLabel(typeId: string): string {
    return relationTypeOf(typeId)?.label ?? typeId
  }

  function relationColor(typeId: string): string {
    return relationTypeOf(typeId)?.color ?? 'var(--color-accent)'
  }

  /** 建议对中与当前知识点相对的另一端标题 */
  function suggestionOtherTitle(suggestion: RelationSuggestion): string {
    return suggestion.source_id === selectedPointId.value
      ? suggestion.target_title
      : suggestion.source_title
  }

  async function refreshTypes(): Promise<void> {
    try {
      types.value = await relationsApi.listTypes()
    } catch (error) {
      errorMessage.value = errorText(error)
    }
  }

  async function loadCourseRelations(courseId: string): Promise<void> {
    const requestId = ++relationsRequest
    isLoadingRelations.value = true
    relationsError.value = null

    try {
      const result = await relationsApi.listByCourse(courseId)
      if (requestId === relationsRequest) {
        relations.value = result
      }
    } catch (error) {
      if (requestId === relationsRequest) {
        relations.value = []
        relationsError.value = errorText(error)
      }
    } finally {
      if (requestId === relationsRequest) {
        isLoadingRelations.value = false
      }
    }
  }

  async function loadSuggestions(pointId: string): Promise<void> {
    const requestId = ++suggestionsRequest
    isLoadingSuggestions.value = true
    suggestionsError.value = null

    try {
      const result = await relationsApi.generateSuggestions(
        pointId,
        PAGE_SUGGESTION_LIMIT,
      )
      if (requestId === suggestionsRequest) {
        suggestions.value = result
      }
    } catch (error) {
      if (requestId === suggestionsRequest) {
        suggestions.value = []
        suggestionsError.value = errorText(error)
      }
    } finally {
      if (requestId === suggestionsRequest) {
        isLoadingSuggestions.value = false
      }
    }
  }

  async function selectPoint(pointId: string | null): Promise<void> {
    selectedPointId.value = pointId
    suggestionsError.value = null

    if (!pointId) {
      suggestionsRequest += 1
      suggestions.value = []
      isLoadingSuggestions.value = false
      return
    }

    await loadSuggestions(pointId)
  }

  async function selectCourse(
    courseId: string,
    preferredPointId: string | null = null,
  ): Promise<void> {
    selectedCourseId.value = courseId
    typeFilter.value = ''
    keyword.value = ''

    await loadCourseRelations(courseId)

    const available = coursePoints.value
    const target =
      (preferredPointId && available.some((point) => point.id === preferredPointId)
        ? preferredPointId
        : null) ??
      (selectedPointId.value &&
      available.some((point) => point.id === selectedPointId.value)
        ? selectedPointId.value
        : null) ??
      available[0]?.id ??
      null

    await selectPoint(target)
  }

  async function initialize(
    preferredCourseId: string | null = null,
    preferredPointId: string | null = null,
  ): Promise<void> {
    isLoading.value = true
    errorMessage.value = null

    try {
      const [courseList, pointList, typeList] = await Promise.all([
        coursesApi.list(),
        knowledgePointsApi.list(),
        relationsApi.listTypes(),
      ])

      courses.value = courseList
      allPoints.value = pointList
      types.value = typeList

      // 深链可能只带知识点：从知识点反推所属课程
      let courseId = preferredCourseId
      if (!courseId && preferredPointId) {
        courseId =
          pointList.find((point) => point.id === preferredPointId)?.course_id ?? null
      }

      const targetCourse =
        (courseId && courseList.some((course) => course.id === courseId)
          ? courseId
          : null) ??
        (selectedCourseId.value &&
        courseList.some((course) => course.id === selectedCourseId.value)
          ? selectedCourseId.value
          : null) ??
        courseList[0]?.id ??
        null

      if (targetCourse) {
        await selectCourse(targetCourse, preferredPointId)
      } else {
        selectedCourseId.value = null
        selectedPointId.value = null
        relations.value = []
        suggestions.value = []
      }

      initialized.value = true
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  /** 顶部刷新 / 错误重试 */
  async function refresh(): Promise<void> {
    if (!selectedCourseId.value) {
      await initialize()
      return
    }

    isLoading.value = true
    errorMessage.value = null

    try {
      const [courseList, pointList] = await Promise.all([
        coursesApi.list(),
        knowledgePointsApi.list(),
      ])
      courses.value = courseList
      allPoints.value = pointList
      await refreshTypes()

      const courseId =
        (selectedCourseId.value &&
        courseList.some((course) => course.id === selectedCourseId.value)
          ? selectedCourseId.value
          : null) ?? courseList[0]?.id

      if (courseId) {
        await selectCourse(courseId, selectedPointId.value)
      } else {
        selectedCourseId.value = null
        selectedPointId.value = null
        relations.value = []
        suggestions.value = []
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  /** 关联变更后同步列表、类型使用次数与建议 */
  async function afterRelationChange(): Promise<void> {
    const tasks: Promise<void>[] = [refreshTypes()]

    if (selectedCourseId.value) {
      tasks.push(loadCourseRelations(selectedCourseId.value))
    }
    if (selectedPointId.value) {
      tasks.push(loadSuggestions(selectedPointId.value))
    }

    await Promise.all(tasks)
  }

  async function runMutation<T>(action: () => Promise<T>): Promise<T> {
    isMutating.value = true
    try {
      return await action()
    } finally {
      isMutating.value = false
    }
  }

  async function createRelation(input: CreateRelationInput): Promise<RelationDetail> {
    return runMutation(async () => {
      const detail = await relationsApi.create(input)
      await afterRelationChange()
      return detail
    })
  }

  async function updateRelation(
    relationId: string,
    input: UpdateRelationInput,
  ): Promise<RelationDetail> {
    return runMutation(async () => {
      const detail = await relationsApi.update(relationId, input)
      await afterRelationChange()
      return detail
    })
  }

  async function deleteRelation(relationId: string): Promise<void> {
    await runMutation(async () => {
      await relationsApi.remove(relationId)
      await afterRelationChange()
    })
  }

  async function generateSuggestions(): Promise<void> {
    if (!selectedPointId.value) return
    await loadSuggestions(selectedPointId.value)
  }

  async function acceptSuggestion(suggestionId: string): Promise<RelationDetail> {
    return runMutation(async () => {
      const detail = await relationsApi.acceptSuggestion(suggestionId)
      suggestions.value = suggestions.value.filter((item) => item.id !== suggestionId)

      const tasks: Promise<void>[] = [refreshTypes()]
      if (selectedCourseId.value) {
        tasks.push(loadCourseRelations(selectedCourseId.value))
      }
      await Promise.all(tasks)

      return detail
    })
  }

  async function rejectSuggestion(suggestionId: string): Promise<void> {
    await runMutation(async () => {
      await relationsApi.rejectSuggestion(suggestionId)
      suggestions.value = suggestions.value.filter((item) => item.id !== suggestionId)
    })
  }

  async function createType(input: RelationTypeInput): Promise<RelationType> {
    return runMutation(async () => {
      const type = await relationsApi.createType(input)
      await Promise.all([refreshTypes(), loadCourseRelationsIfAny()])
      return type
    })
  }

  async function updateType(
    typeId: string,
    input: RelationTypeInput,
  ): Promise<RelationType> {
    return runMutation(async () => {
      const type = await relationsApi.updateType(typeId, input)
      await Promise.all([refreshTypes(), loadCourseRelationsIfAny()])
      return type
    })
  }

  async function deleteType(typeId: string): Promise<void> {
    await runMutation(async () => {
      await relationsApi.removeType(typeId)
      await refreshTypes()
    })
  }

  /** 类型变更后刷新关联详情里的展示名称与颜色 */
  async function loadCourseRelationsIfAny(): Promise<void> {
    if (selectedCourseId.value) {
      await loadCourseRelations(selectedCourseId.value)
    }
  }

  function setTypeFilter(value: string): void {
    typeFilter.value = value
  }

  function dismissError(): void {
    errorMessage.value = null
    relationsError.value = null
    suggestionsError.value = null
  }

  return {
    // 数据
    courses,
    allPoints,
    relations,
    types,
    suggestions,
    selectedCourseId,
    selectedPointId,
    selectedCourse,
    selectedPoint,
    coursePoints,
    filteredPoints,
    selectedPointRelations,
    totalRelations,
    typeFilter,
    keyword,
    // 状态
    isLoading,
    isLoadingRelations,
    isLoadingSuggestions,
    isMutating,
    errorMessage,
    relationsError,
    suggestionsError,
    initialized,
    // 查询
    relationCountOf,
    relationTypeOf,
    relationLabel,
    relationColor,
    suggestionOtherTitle,
    // 行为
    initialize,
    refresh,
    refreshTypes,
    selectCourse,
    selectPoint,
    setTypeFilter,
    generateSuggestions,
    createRelation,
    updateRelation,
    deleteRelation,
    acceptSuggestion,
    rejectSuggestion,
    createType,
    updateType,
    deleteType,
    dismissError,
  }
})
