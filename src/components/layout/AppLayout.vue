<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import AppHeader from './AppHeader.vue'
import AppSidebar from './AppSidebar.vue'
import { useAppStore } from '@/stores/app'

const route = useRoute()
const appStore = useAppStore()
const contentRef = ref<HTMLElement | null>(null)

const isImmersive = computed(() => route.meta.immersive === true)

/**
 * 实际生效的沉浸式布局。
 *
 * 路由变化时如果立刻切换 padding / 侧边栏，正在淡出的旧页面会突然收缩，
 * 因此这里把布局切换推迟到旧页面离场之后、新页面以 opacity: 0 插入的瞬间。
 */
const appliedImmersive = ref(isImmersive.value)

/**
 * 笔记页等沉浸式页面默认收起应用级导航，把宽度留给三栏工作区。
 * 离开时恢复进入前的状态；如果用户手动展开过，则尊重用户的选择。
 */
let sidebarBeforeImmersive: boolean | null = null

function applyRouteLayout(): void {
  const immersive = isImmersive.value
  appliedImmersive.value = immersive

  if (immersive) {
    sidebarBeforeImmersive = appStore.sidebarCollapsed
    appStore.setSidebarCollapsed(true)
    return
  }

  if (sidebarBeforeImmersive === null) return

  if (appStore.sidebarCollapsed) {
    appStore.setSidebarCollapsed(sidebarBeforeImmersive)
  }
  sidebarBeforeImmersive = null
}

// 首屏没有路由过渡，直接应用一次布局
applyRouteLayout()

/** 旧页面已离场：先把滚动位置归零，再让新页面从顶部进入 */
function handleBeforeEnter(): void {
  contentRef.value?.scrollTo({ top: 0 })
}
</script>

<template>
  <div class="app-shell">
    <AppSidebar />

    <div class="app-main">
      <AppHeader />

      <main
        ref="contentRef"
        class="app-content"
        :class="{ 'app-content--flush': appliedImmersive }"
      >
        <div
          class="app-content__inner"
          :class="{ 'app-content__inner--flush': appliedImmersive }"
        >
          <RouterView v-slot="{ Component, route: currentRoute }">
            <Transition
              name="page"
              mode="out-in"
              @before-enter="handleBeforeEnter"
              @enter="applyRouteLayout"
            >
              <component :is="Component" :key="currentRoute.path" />
            </Transition>
          </RouterView>
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  background: var(--color-bg);
}

.app-main {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
}

.app-content {
  flex: 1;
  padding: 38px 48px 72px;
  overflow-y: auto;
  scrollbar-gutter: stable;
}

.app-content__inner {
  width: 100%;
  max-width: 1160px;
  margin: 0 auto;
}

/* 沉浸式页面：占满整个内容区，由页面自己管理滚动 */
.app-content--flush {
  padding: 0;
  overflow: hidden;
}

.app-content__inner--flush {
  height: 100%;
  max-width: none;
}

@media (max-width: 800px) {
  .app-content {
    padding: 28px 24px 56px;
    scrollbar-gutter: auto;
  }
}

@media (max-width: 520px) {
  .app-content {
    padding: 24px 16px 48px;
  }
}
</style>
