import '@/styles/index.css'

import { createApp } from 'vue'
import App from './App.vue'
import router from '@/router'
import pinia from '@/stores'

const app = createApp(App)

// Pinia 先注册，保证路由守卫、组件中可以直接使用 store
app.use(pinia)
app.use(router)

app.mount('#app')
