import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { coursesApi, type CourseInput } from '@/services/courses'
import type { Course } from '@/types'

function errorText(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}

/**
 * 课程管理数据
 *
 * - 列表、详情、增删改全部通过 Tauri 命令读写本地 SQLite；
 * - 修改成功后同步更新内存列表，避免页面之间来回请求；
 * - 查询失败保留在 errorMessage 中，由页面决定如何展示与重试。
 */
export const useCoursesStore = defineStore('courses', () => {
  const items = ref<Course[]>([])
  const isLoading = ref(false)
  const errorMessage = ref<string | null>(null)
  const initialized = ref(false)

  const total = computed(() => items.value.length)

  function courseById(courseId: string): Course | null {
    return items.value.find((course) => course.id === courseId) ?? null
  }

  /** 新增或替换列表中的课程 */
  function upsert(course: Course): void {
    const exists = items.value.some((item) => item.id === course.id)
    items.value = exists
      ? items.value.map((item) => (item.id === course.id ? course : item))
      : [...items.value, course]
  }

  async function fetchCourses(force = false): Promise<void> {
    if (initialized.value && !force) return

    isLoading.value = true
    errorMessage.value = null

    try {
      items.value = await coursesApi.list()
      initialized.value = true
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  async function getCourse(courseId: string): Promise<Course> {
    const course = await coursesApi.get(courseId)
    upsert(course)
    return course
  }

  async function createCourse(input: CourseInput): Promise<Course> {
    const course = await coursesApi.create(input)
    upsert(course)
    return course
  }

  async function updateCourse(courseId: string, input: CourseInput): Promise<Course> {
    const course = await coursesApi.update(courseId, input)
    upsert(course)
    return course
  }

  async function deleteCourse(courseId: string): Promise<void> {
    await coursesApi.remove(courseId)
    items.value = items.value.filter((course) => course.id !== courseId)
  }

  function dismissError(): void {
    errorMessage.value = null
  }

  return {
    items,
    isLoading,
    errorMessage,
    initialized,
    total,
    courseById,
    fetchCourses,
    getCourse,
    createCourse,
    updateCourse,
    deleteCourse,
    dismissError,
  }
})
