import { computed, reactive, ref } from 'vue'
import { defineStore } from 'pinia'
import { coursesApi } from '@/services/courses'
import { notesApi, type CreateKnowledgePointInput } from '@/services/notes'
import type {
  Course,
  KnowledgePoint,
  KnowledgeRelation,
  Note,
  SaveStatus,
  Tag,
} from '@/types'
import { countCharacters } from '@/utils/format'

/** 上次选中的知识点，重新打开应用时恢复 */
const LAST_POINT_KEY = 'kgn:notes:last-point'
/** 自动保存延迟（毫秒） */
const SAVE_DELAY = 800

function readLastPointId(): string | null {
  try {
    return window.localStorage.getItem(LAST_POINT_KEY)
  } catch {
    return null
  }
}

function persistLastPointId(pointId: string | null): void {
  try {
    if (pointId) {
      window.localStorage.setItem(LAST_POINT_KEY, pointId)
    } else {
      window.localStorage.removeItem(LAST_POINT_KEY)
    }
  } catch {
    // ignore
  }
}

function errorText(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}

export interface RelatedItem {
  relation: KnowledgeRelation
  point: KnowledgePoint
}

/**
 * 笔记工作区数据与保存状态
 *
 * - 课程 / 知识点 / 笔记 / 标签 / 关联全部来自 SQLite，不使用模拟数据；
 * - 草稿修改后 800ms 自动保存，切换知识点前会先落盘；
 * - 保存状态统一为 idle | dirty | saving | saved | error，供界面展示。
 */
export const useNotesStore = defineStore('notes', () => {
  const courses = ref<Course[]>([])
  const knowledgePoints = ref<KnowledgePoint[]>([])
  const relations = ref<KnowledgeRelation[]>([])
  const allTags = ref<Tag[]>([])
  const noteTags = ref<Tag[]>([])

  const selectedPointId = ref<string | null>(null)
  const note = ref<Note | null>(null)
  const draft = reactive({ title: '', content: '' })

  const saveStatus = ref<SaveStatus>('idle')
  const lastSavedAt = ref<Date | null>(null)
  const isLoading = ref(false)
  const isLoadingNote = ref(false)
  const errorMessage = ref<string | null>(null)
  const initialized = ref(false)

  let saveTimer: number | null = null
  let saveTask: Promise<void> | null = null

  const pointIndex = computed(() => {
    const map = new Map<string, KnowledgePoint>()
    for (const point of knowledgePoints.value) {
      map.set(point.id, point)
    }
    return map
  })

  function pointById(id: string | null): KnowledgePoint | null {
    return id ? (pointIndex.value.get(id) ?? null) : null
  }

  const selectedPoint = computed(() => pointById(selectedPointId.value))
  const selectedCourse = computed(
    () => courses.value.find((course) => course.id === selectedPoint.value?.course_id) ?? null,
  )

  const childrenByParent = computed(() => {
    const map = new Map<string, KnowledgePoint[]>()
    for (const point of knowledgePoints.value) {
      const key = point.parent_id ?? '__root__'
      const list = map.get(key)
      if (list) {
        list.push(point)
      } else {
        map.set(key, [point])
      }
    }
    return map
  })

  /** 某个父知识点下的直接子节点 */
  function childrenOf(parentId: string): KnowledgePoint[] {
    return childrenByParent.value.get(parentId) ?? []
  }

  /** 某门课程的顶层知识点 */
  function rootPointsOf(courseId: string): KnowledgePoint[] {
    return (childrenByParent.value.get('__root__') ?? []).filter(
      (point) => point.course_id === courseId,
    )
  }

  /** 某门课程下的全部知识点（用于计数） */
  function pointsOfCourse(courseId: string): KnowledgePoint[] {
    return knowledgePoints.value.filter((point) => point.course_id === courseId)
  }

  /** 当前知识点的完整路径（课程根 -> 当前节点） */
  const selectedPointPath = computed<KnowledgePoint[]>(() => {
    const path: KnowledgePoint[] = []
    const visited = new Set<string>()
    let current = selectedPoint.value

    while (current && !visited.has(current.id)) {
      visited.add(current.id)
      path.unshift(current)
      current = pointById(current.parent_id)
    }

    return path
  })

  const relatedItems = computed<RelatedItem[]>(() => {
    const currentId = selectedPointId.value
    if (!currentId) return []

    const items: RelatedItem[] = []
    for (const relation of relations.value) {
      const otherId =
        relation.source_id === currentId
          ? relation.target_id
          : relation.target_id === currentId
            ? relation.source_id
            : null
      if (!otherId) continue

      const point = pointById(otherId)
      if (point) {
        items.push({ relation, point })
      }
    }
    return items
  })

  const characterCount = computed(() => countCharacters(draft.content))

  async function loadWorkspaceData(): Promise<void> {
    const [courseList, pointList, relationList, tagList] = await Promise.all([
      coursesApi.list(),
      notesApi.listKnowledgePoints(),
      notesApi.listRelations(),
      notesApi.listTags(),
    ])

    courses.value = courseList
    knowledgePoints.value = pointList
    relations.value = relationList
    allTags.value = tagList
  }

  /** 初始化工作区：加载数据并恢复上次选择的知识点 */
  async function initialize(force = false): Promise<void> {
    if (initialized.value && !force) return

    initialized.value = true
    isLoading.value = true
    errorMessage.value = null

    try {
      await loadWorkspaceData()

      // 课程 / 知识点可能在课程管理页被删除：丢弃已失效的选中状态，
      // 避免把草稿保存回已经删除的知识点。
      if (selectedPointId.value && !pointById(selectedPointId.value)) {
        clearSelection()
      }

      const lastPoint = pointById(readLastPointId())
      const target = lastPoint ?? knowledgePoints.value[0] ?? null
      if (target) {
        await selectPoint(target.id)
      } else {
        clearSelection()
        persistLastPointId(null)
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  /** 静默刷新目录数据（课程管理页可能新增 / 改名 / 删除课程与知识点） */
  async function refreshCatalog(): Promise<void> {
    try {
      await loadWorkspaceData()

      const current = selectedPointId.value ? pointById(selectedPointId.value) : null
      if (current) return

      // 选中的知识点已不存在：清理草稿状态，切到第一个可用知识点
      clearSelection()
      const target = knowledgePoints.value[0] ?? null
      if (target) {
        await selectPoint(target.id)
      } else {
        persistLastPointId(null)
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    }
  }

  async function selectPoint(pointId: string): Promise<void> {
    if (selectedPointId.value === pointId) return

    const saved = await flushSave()
    if (!saved) {
      errorMessage.value = '当前笔记保存失败，已暂停切换知识点，请重试保存后再切换。'
      return
    }

    const point = pointById(pointId)
    if (!point) return

    selectedPointId.value = pointId
    persistLastPointId(pointId)
    note.value = null
    noteTags.value = []
    draft.title = point.title
    draft.content = ''
    saveStatus.value = 'idle'
    lastSavedAt.value = null
    errorMessage.value = null

    await loadNote(pointId)
  }

  async function loadNote(pointId: string): Promise<void> {
    isLoadingNote.value = true

    try {
      const existing = await notesApi.getNote(pointId)
      if (selectedPointId.value !== pointId) return

      if (existing) {
        note.value = existing
        draft.title = existing.title
        draft.content = existing.content
        saveStatus.value = 'saved'
        lastSavedAt.value = new Date(existing.updated_at)

        try {
          noteTags.value = await notesApi.listNoteTags(existing.id)
        } catch (error) {
          errorMessage.value = errorText(error)
        }
      }
    } catch (error) {
      if (selectedPointId.value === pointId) {
        errorMessage.value = errorText(error)
        saveStatus.value = 'error'
      }
    } finally {
      if (selectedPointId.value === pointId) {
        isLoadingNote.value = false
      }
    }
  }

  function updateTitle(value: string): void {
    if (draft.title === value) return
    draft.title = value
    markDirty()
  }

  function updateContent(value: string): void {
    if (draft.content === value) return
    draft.content = value
    markDirty()
  }

  function markDirty(): void {
    if (!selectedPointId.value || isLoadingNote.value) return

    saveStatus.value = 'dirty'
    scheduleSave()
  }

  function scheduleSave(): void {
    if (saveTimer !== null) {
      window.clearTimeout(saveTimer)
    }

    saveTimer = window.setTimeout(() => {
      saveTimer = null
      void saveNow()
    }, SAVE_DELAY)
  }

  /** 立即保存（自动保存计时器与手动 Ctrl+S 共用） */
  async function saveNow(options: { force?: boolean } = {}): Promise<void> {
    if (!selectedPointId.value) return

    if (saveTimer !== null) {
      window.clearTimeout(saveTimer)
      saveTimer = null
    }

    if (saveTask) {
      await saveTask
    }

    const needsSave = saveStatus.value === 'dirty' || saveStatus.value === 'error'
    // 内容为空且标题仍是知识点默认标题时，不创建空笔记（例如只是切换了知识点）
    const hasContent =
      note.value !== null ||
      draft.content.trim() !== '' ||
      draft.title.trim() !== (selectedPoint.value?.title ?? '')

    if (!needsSave) return
    if (!hasContent && !options.force) return

    const pointId = selectedPointId.value
    const payload = {
      noteId: note.value?.id ?? null,
      knowledgePointId: pointId,
      title: draft.title,
      content: draft.content,
    }

    saveStatus.value = 'saving'

    const task = (async () => {
      try {
        const saved = await notesApi.saveNote(payload)
        if (selectedPointId.value !== pointId) return

        note.value = saved
        lastSavedAt.value = new Date(saved.updated_at)

        // 保存期间用户又输入了内容时，保持 dirty 状态，等待下一次自动保存
        if (saveStatus.value === 'saving') {
          saveStatus.value = 'saved'
        }
      } catch (error) {
        if (selectedPointId.value === pointId) {
          saveStatus.value = 'error'
          errorMessage.value = errorText(error)
        }
      }
    })()

    saveTask = task.finally(() => {
      saveTask = null
    })

    await saveTask
  }

  /** 切换知识点 / 关闭页面前调用，确保草稿落盘；返回是否保存成功 */
  async function flushSave(): Promise<boolean> {
    if (saveTimer !== null) {
      window.clearTimeout(saveTimer)
      saveTimer = null
    }

    if (saveTask) {
      await saveTask
    }

    const needsSave = saveStatus.value === 'dirty' || saveStatus.value === 'error'
    const hasContent =
      note.value !== null ||
      draft.content.trim() !== '' ||
      draft.title.trim() !== (selectedPoint.value?.title ?? '')

    if (needsSave && !hasContent) {
      return true
    }

    if (needsSave) {
      await saveNow()
    }

    return saveStatus.value !== 'error'
  }

  /** 清空当前编辑器状态（选中的知识点被删除时使用） */
  function clearSelection(): void {
    selectedPointId.value = null
    note.value = null
    noteTags.value = []
    draft.title = ''
    draft.content = ''
    saveStatus.value = 'idle'
    lastSavedAt.value = null
  }

  async function deleteCurrentNote(): Promise<void> {
    const target = note.value
    if (!target) return

    await notesApi.deleteNote(target.id)
    if (note.value?.id !== target.id) return

    note.value = null
    noteTags.value = []
    draft.title = selectedPoint.value?.title ?? ''
    draft.content = ''
    saveStatus.value = 'idle'
    lastSavedAt.value = null
  }

  /** 在笔记页新建课程后同步目录（课程表单由课程管理模块统一提供） */
  function addCourse(course: Course): void {
    if (courses.value.some((item) => item.id === course.id)) return
    courses.value = [...courses.value, course]
  }

  async function createKnowledgePoint(
    input: CreateKnowledgePointInput,
  ): Promise<KnowledgePoint> {
    const point = await notesApi.createKnowledgePoint(input)
    knowledgePoints.value = [...knowledgePoints.value, point]
    await selectPoint(point.id)
    return point
  }

  async function addTagToNote(name: string): Promise<void> {
    const trimmed = name.trim()
    if (!trimmed || !selectedPointId.value) return

    const saved = await flushSave()
    if (!saved) {
      throw new Error('当前笔记保存失败，请稍后重试')
    }

    // 还没有笔记时先创建一条，避免标签无所依附
    if (!note.value) {
      saveStatus.value = 'dirty'
      await saveNow({ force: true })
    }
    if (!note.value) return

    const tag = await notesApi.createTag(trimmed)
    const noteId = note.value.id
    await notesApi.attachTag(noteId, tag.id)

    noteTags.value = await notesApi.listNoteTags(noteId)
    allTags.value = await notesApi.listTags()
  }

  async function removeTagFromNote(tag: Tag): Promise<void> {
    if (!note.value) return

    const noteId = note.value.id
    await notesApi.detachTag(noteId, tag.id)
    noteTags.value = noteTags.value.filter((item) => item.id !== tag.id)
  }

  function dismissError(): void {
    errorMessage.value = null
  }

  return {
    // 数据
    courses,
    knowledgePoints,
    relations,
    allTags,
    noteTags,
    // 选择与草稿
    selectedPointId,
    selectedPoint,
    selectedPointPath,
    selectedCourse,
    note,
    draft,
    characterCount,
    relatedItems,
    // 状态
    saveStatus,
    lastSavedAt,
    isLoading,
    isLoadingNote,
    errorMessage,
    initialized,
    // 行为
    initialize,
    refreshCatalog,
    selectPoint,
    updateTitle,
    updateContent,
    saveNow,
    flushSave,
    deleteCurrentNote,
    addCourse,
    createKnowledgePoint,
    addTagToNote,
    removeTagFromNote,
    childrenOf,
    rootPointsOf,
    pointsOfCourse,
    dismissError,
  }
})
