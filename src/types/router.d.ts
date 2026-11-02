import type { IconName } from '@/components/common/icons'

/**
 * 扩展 vue-router 的路由元信息类型，
 * 让路由表、侧边导航与面包屑共享同一份带类型的配置。
 */
declare module 'vue-router' {
  interface RouteMeta {
    /** 页面标题，用于导航、面包屑与页面头部 */
    title?: string
    /** 侧边导航图标 */
    icon?: IconName
    /** 是否出现在侧边导航栏 */
    nav?: boolean
    /** 导航排序，数字越小越靠前 */
    order?: number
    /** 上级页面，用于面包屑（例如课程详情 -> 课程管理） */
    parent?: { title: string; path: string }
    /** 沉浸式页面：占满内容区、去掉内容区留白，并默认收起应用级导航 */
    immersive?: boolean
  }
}

export {}
