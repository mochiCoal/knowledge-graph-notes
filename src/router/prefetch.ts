import type { RouteRecordRaw } from 'vue-router'
import { routes } from './routes'

type LazyRouteComponent = () => Promise<unknown>

function collectLoaders(records: readonly RouteRecordRaw[]): LazyRouteComponent[] {
  const loaders: LazyRouteComponent[] = []

  for (const record of records) {
    if (typeof record.component === 'function') {
      loaders.push(record.component as LazyRouteComponent)
    }
    if (record.children?.length) {
      loaders.push(...collectLoaders(record.children))
    }
  }

  return loaders
}

/**
 * 在浏览器空闲时预取所有懒加载页面。
 *
 * 页面组件使用动态 import 拆分，首次点击导航时如果才开始下载分块，
 * 会出现「点击后停一下才开始切换」的延迟；预取后各页面切换基本即时响应。
 * 预取失败不提示，真正导航时仍会重新加载并展示错误。
 */
export function prefetchRouteComponents(): void {
  const run = (): void => {
    for (const load of collectLoaders(routes)) {
      try {
        void load().catch(() => {})
      } catch {
        // 忽略预取阶段的异常（例如开发环境下模块被重载）
      }
    }
  }

  const idleWindow = window as Window & {
    requestIdleCallback?: (callback: IdleRequestCallback, options?: IdleRequestOptions) => number
  }

  if (idleWindow.requestIdleCallback) {
    idleWindow.requestIdleCallback(run, { timeout: 2000 })
  } else {
    window.setTimeout(run, 600)
  }
}
