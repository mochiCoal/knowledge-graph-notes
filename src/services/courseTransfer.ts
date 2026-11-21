import { open, save } from '@tauri-apps/plugin-dialog'
import { invokeCommand } from './tauri'
import { toFileName } from '@/utils/format'
import type { CourseExportResult, CourseImportResult } from '@/types'

/** 课程包文件过滤器，导入 / 导出共用 */
const COURSE_PACKAGE_FILTER = {
  name: '课程包',
  extensions: ['json'],
}

/** 生成文件名中的时间戳，例如 20261109-1710 */
function fileTimestamp(date = new Date()): string {
  const pad = (value: number): string => String(value).padStart(2, '0')
  return (
    `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}` +
    `-${pad(date.getHours())}${pad(date.getMinutes())}`
  )
}

/**
 * 课程包导出 / 导入的服务封装。
 *
 * 文件位置由系统保存 / 打开对话框选择，真正的读写与数据校验全部在 Rust 端完成。
 */

/** 导出课程及其下全部内容为单个 JSON 课程包；用户取消时返回 null */
export async function exportCoursePackage(course: {
  id: string
  name: string
}): Promise<CourseExportResult | null> {
  const destination = await save({
    title: '导出课程包',
    defaultPath: `${toFileName(course.name)}-课程包-${fileTimestamp()}.json`,
    filters: [COURSE_PACKAGE_FILTER],
  })

  if (!destination) return null

  return invokeCommand<CourseExportResult>('export_course_package', {
    courseId: course.id,
    destination,
  })
}

/** 选择课程包并导入；用户取消时返回 null */
export async function importCoursePackage(): Promise<CourseImportResult | null> {
  const selected = await open({
    title: '选择要导入的课程包',
    multiple: false,
    directory: false,
    filters: [COURSE_PACKAGE_FILTER],
  })

  if (typeof selected !== 'string') return null

  return invokeCommand<CourseImportResult>('import_course_package', {
    source: selected,
  })
}