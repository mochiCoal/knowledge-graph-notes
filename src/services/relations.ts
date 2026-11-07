import { invokeCommand } from './tauri'
import type {
  KnowledgeRelation,
  RelationDetail,
  RelationSuggestion,
  RelationType,
} from '@/types'

/** 新建关联入参 */
export interface CreateRelationInput {
  sourceId: string
  targetId: string
  relationType: string
  description: string
}

/** 修改关联入参（两端知识点保持不变） */
export interface UpdateRelationInput {
  relationType: string
  description: string
}

/** 新建 / 修改关联类型入参 */
export interface RelationTypeInput {
  label: string
  color: string
  directed: boolean
  description: string
}

/**
 * 知识点关联模块的后端命令封装。
 *
 * 注意：Tauri 会把 Rust 的 snake_case 参数名映射为 camelCase，
 * 因此这里调用时统一使用 camelCase 参数。
 */
export const relationsApi = {
  /** 全部关联（轻量结构，笔记工作区判断相关知识点） */
  list: () => invokeCommand<KnowledgeRelation[]>('list_relations'),

  /** 全部关联详情 */
  listDetails: () => invokeCommand<RelationDetail[]>('list_relation_details'),

  /** 与某个知识点相关的关联 */
  listForPoint: (knowledgePointId: string) =>
    invokeCommand<RelationDetail[]>('list_relation_details_for_point', { knowledgePointId }),

  /** 某门课程涉及的关联（任一端属于该课程） */
  listByCourse: (courseId: string) =>
    invokeCommand<RelationDetail[]>('list_relation_details_by_course', { courseId }),

  /** 手动创建关联 */
  create: (input: CreateRelationInput) =>
    invokeCommand<RelationDetail>('create_relation', {
      sourceId: input.sourceId,
      targetId: input.targetId,
      relationType: input.relationType,
      description: input.description,
    }),

  /** 修改关联类型与描述 */
  update: (relationId: string, input: UpdateRelationInput) =>
    invokeCommand<RelationDetail>('update_relation', {
      relationId,
      relationType: input.relationType,
      description: input.description,
    }),

  /** 删除关联 */
  remove: (relationId: string) => invokeCommand<void>('delete_relation', { relationId }),

  /** 关联类型字典（含使用次数） */
  listTypes: () => invokeCommand<RelationType[]>('list_relation_types'),

  /** 新增自定义关联类型 */
  createType: (input: RelationTypeInput) =>
    invokeCommand<RelationType>('create_relation_type', {
      label: input.label,
      color: input.color,
      directed: input.directed,
      description: input.description,
    }),

  /** 修改关联类型 */
  updateType: (typeId: string, input: RelationTypeInput) =>
    invokeCommand<RelationType>('update_relation_type', {
      typeId,
      label: input.label,
      color: input.color,
      directed: input.directed,
      description: input.description,
    }),

  /** 删除未被使用的自定义关联类型 */
  removeType: (typeId: string) =>
    invokeCommand<void>('delete_relation_type', { typeId }),

  /** 生成（并刷新）某个知识点的关联建议 */
  generateSuggestions: (knowledgePointId: string, limit = 8) =>
    invokeCommand<RelationSuggestion[]>('generate_relation_suggestions', {
      knowledgePointId,
      limit,
    }),

  /** 读取已保存的待处理建议（不重新计算） */
  listSuggestions: (knowledgePointId: string, limit = 8) =>
    invokeCommand<RelationSuggestion[]>('list_relation_suggestions', {
      knowledgePointId,
      limit,
    }),

  /** 接受建议：写入正式关联 */
  acceptSuggestion: (suggestionId: string) =>
    invokeCommand<RelationDetail>('accept_relation_suggestion', { suggestionId }),

  /** 拒绝建议：之后不再重复推荐 */
  rejectSuggestion: (suggestionId: string) =>
    invokeCommand<void>('reject_relation_suggestion', { suggestionId }),
}
