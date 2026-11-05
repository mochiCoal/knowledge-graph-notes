import '@/styles/index.css'

import { createApp } from 'vue'
import App from './App.vue'
import router from '@/router'
import { prefetchRouteComponents } from '@/router/prefetch'
import pinia from '@/stores'

const app = createApp(App)

// Pinia 先注册，保证路由守卫、组件中可以直接使用 store
app.use(pinia)
app.use(router)

app.mount('#app')

// 首屏渲染完成后空闲预取其它页面，让后续路由切换不再等待 JS 分块下载
prefetchRouteComponents()
