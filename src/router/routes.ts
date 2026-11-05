import type { RouteRecordRaw } from 'vue-router'

/**
 * 应用路由表
 *
 * 约定：
 * 1. 业务页面统一使用懒加载，减小首屏体积；
 * 2. `meta.nav === true` 的顶级路由会自动出现在左侧导航栏（按 `meta.order` 排序），
 *    因此新增页面时只需在这里补充一条路由，无需再改导航组件；
 * 3. `meta.parent` 用于生成面包屑，例如课程详情页的上级是课程管理。
 */
export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/views/HomeView.vue'),
    meta: { title: '首页', icon: 'home', nav: true, order: 1 },
  },
  {
    path: '/courses',
    name: 'courses',
    component: () => import('@/views/CoursesView.vue'),
    meta: { title: '课程管理', icon: 'courses', nav: true, order: 2 },
  },
  {
    path: '/courses/:courseId',
    name: 'course-detail',
    component: () => import('@/views/CourseDetailView.vue'),
    props: true,
    meta: {
      title: '课程详情',
      nav: false,
      parent: { title: '课程管理', path: '/courses' },
    },
  },
  {
    path: '/knowledge-points/:courseId?',
    name: 'knowledge-points',
    component: () => import('@/views/KnowledgePointsView.vue'),
    props: true,
    meta: { title: '知识点管理', icon: 'tree', nav: true, order: 3 },
  },
  {
    path: '/notes',
    name: 'notes',
    component: () => import('@/views/NotesView.vue'),
    meta: { title: '笔记', icon: 'notes', nav: true, order: 4, immersive: true },
  },
  {
    path: '/graph',
    name: 'graph',
    component: () => import('@/views/GraphView.vue'),
    meta: { title: '知识图谱', icon: 'graph', nav: true, order: 5 },
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/views/SettingsView.vue'),
    meta: { title: '设置', icon: 'settings', nav: true, order: 6 },
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    component: () => import('@/views/NotFoundView.vue'),
    meta: { title: '页面未找到' },
  },
]
