import { invokeCommand } from './tauri'
import type { Course } from '@/types'

/** 新建 / 修改课程的入参 */
export interface CourseInput {
  name: string
  description: string
}

/**
 * 课程管理模块的后端命令封装。
 *
 * Rust 端统一校验课程名称与描述，前端只负责采集输入并展示错误信息。
 */
export const coursesApi = {
  list: () => invokeCommand<Course[]>('list_courses'),

  get: (courseId: string) => invokeCommand<Course>('get_course', { courseId }),

  create: (input: CourseInput) =>
    invokeCommand<Course>('create_course', {
      name: input.name,
      description: input.description,
    }),

  update: (courseId: string, input: CourseInput) =>
    invokeCommand<Course>('update_course', {
      courseId,
      name: input.name,
      description: input.description,
    }),

  remove: (courseId: string) => invokeCommand<void>('delete_course', { courseId }),
}
