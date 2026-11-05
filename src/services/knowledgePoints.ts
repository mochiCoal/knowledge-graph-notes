import { invokeCommand } from './tauri'
import type {
  KnowledgePoint,
  KnowledgePointDeletion,
  KnowledgePointDetail,
  KnowledgePointNode,
} from '@/types'

/** 新建 / 修改知识点的入参 */
export interface KnowledgePointInput {
  title: string
  description: string
  /** 父知识点 ID，null 表示顶层知识点 */
  parentId: string | null
}

/**
 * 知识点管理模块的后端命令封装。
 *
 * 注意：Tauri 会把 Rust 的 snake_case 参数名映射为 camelCase，
 * 因此这里调用时统一使用 camelCase 参数。
 */
export const knowledgePointsApi = {
  /** 全部知识点（笔记工作区使用） */
  list: () => invokeCommand<KnowledgePoint[]>('list_knowledge_points'),

  /** 某门课程下的全部知识点（扁平列表） */
  listByCourse: (courseId: string) =>
    invokeCommand<KnowledgePoint[]>('list_knowledge_points_by_course', { courseId }),

  /** 某门课程的知识点树（含层级、子节点与笔记数量） */
  tree: (courseId: string) =>
    invokeCommand<KnowledgePointNode[]>('get_knowledge_point_tree', { courseId }),

  /** 知识点详情（面包屑路径、层级与统计） */
  detail: (knowledgePointId: string) =>
    invokeCommand<KnowledgePointDetail>('get_knowledge_point_detail', { knowledgePointId }),

  create: (courseId: string, input: KnowledgePointInput) =>
    invokeCommand<KnowledgePoint>('create_knowledge_point', {
      courseId,
      parentId: input.parentId,
      title: input.title,
      description: input.description,
    }),

  update: (knowledgePointId: string, input: KnowledgePointInput) =>
    invokeCommand<KnowledgePoint>('update_knowledge_point', {
      knowledgePointId,
      parentId: input.parentId,
      title: input.title,
      description: input.description,
    }),

  /** 仅调整层级：parentId 为 null 表示提升为顶层知识点 */
  move: (knowledgePointId: string, parentId: string | null) =>
    invokeCommand<KnowledgePoint>('move_knowledge_point', { knowledgePointId, parentId }),

  /** 迁移到另一门课程（整棵子树跟随迁移） */
  moveToCourse: (knowledgePointId: string, courseId: string) =>
    invokeCommand<KnowledgePoint>('move_knowledge_point_to_course', {
      knowledgePointId,
      courseId,
    }),

  /** 删除知识点及其全部后代 */
  remove: (knowledgePointId: string) =>
    invokeCommand<KnowledgePointDeletion>('delete_knowledge_point', { knowledgePointId }),
}
