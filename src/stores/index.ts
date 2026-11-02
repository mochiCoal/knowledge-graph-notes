import { createPinia } from 'pinia'

/**
 * 全局 Pinia 实例，在 main.ts 中通过 app.use(pinia) 注册。
 * 后续业务 store 统一放在 src/stores 目录下，按领域拆分文件。
 */
export const pinia = createPinia()

export default pinia
