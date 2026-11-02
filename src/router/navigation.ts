import type { IconName } from '@/components/common/icons'
import { routes } from './routes'

/** 侧边导航项 */
export interface NavItem {
  path: string
  title: string
  icon: IconName
  order: number
}

/**
 * 从路由表推导侧边导航项。
 *
 * 这样导航与路由始终一致：新增页面只需在 routes.ts 里声明 `meta.nav`，
 * 不需要再维护一份独立的菜单配置。
 */
const items: NavItem[] = routes.flatMap((route) => {
  const meta = route.meta

  if (meta?.nav !== true || !meta.icon) {
    return []
  }

  return [
    {
      path: route.path,
      title: meta.title ?? route.path,
      icon: meta.icon,
      order: meta.order ?? Number.MAX_SAFE_INTEGER,
    },
  ]
})

export const navItems: NavItem[] = items.sort((a, b) => a.order - b.order)
