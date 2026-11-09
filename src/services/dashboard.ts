import { invokeCommand } from './tauri'
import type { DashboardOverview } from '@/types'

/**
 * 首页概览数据。
 *
 * 课程 / 知识点 / 笔记 / 标签 / 关联数量与两组最近记录一次返回，
 * 避免首页为了统计数据连续发起多次查询。
 */
export const dashboardApi = {
  overview: () => invokeCommand<DashboardOverview>('get_dashboard_overview'),
}
