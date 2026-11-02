<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppHeader from './AppHeader.vue'
import AppSidebar from './AppSidebar.vue'
import { useAppStore } from '@/stores/app'

const route = useRoute()
const appStore = useAppStore()
const contentRef = ref<HTMLElement | null>(null)

const isImmersive = computed(() => route.meta.immersive === true)

/**
 * 笔记页等沉浸式页面默认收起应用级导航，把宽度留给三栏工作区。
 * 离开时恢复进入前的状态；如果用户手动展开过，则尊重用户的选择。
 */
let sidebarBeforeImmersive: boolean | null = null

watch(
  isImmersive,
  (immersive) => {
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
  },
  { immediate: true },
)

/** 路由切换后把内容区滚动回顶部（内容区独立滚动） */
watch(
  () => route.fullPath,
  () => {
    contentRef.value?.scrollTo({ top: 0 })
  },
)
</script>

<template>
  <div class="app-shell">
    <AppSidebar />

    <div class="app-main">
      <AppHeader />

      <main
        ref="contentRef"
        class="app-content"
        :class="{ 'app-content--flush': isImmersive }"
      >
        <div
          class="app-content__inner"
          :class="{ 'app-content__inner--flush': isImmersive }"
        >
          <RouterView v-slot="{ Component, route: currentRoute }">
            <Transition name="page" mode="out-in">
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
  padding: 26px 24px 48px;
  overflow-y: auto;
}

.app-content__inner {
  width: 100%;
  max-width: 1080px;
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
</style>
