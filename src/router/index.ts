import { createRouter, createWebHashHistory } from 'vue-router'
import { routes } from './routes'

/**
 * 路由实例
 *
 * 使用 hash 模式：Tauri 打包后前端由自定义协议提供资源，
 * hash 模式无需服务端 history 回退规则，开发环境与桌面环境行为一致，
 * 刷新或直接进入子路由时都不会出现 404。
 */
const router = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior: () => ({ top: 0 }),
})

export default router
