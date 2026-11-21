<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppHeader from './AppHeader.vue'
import AppSidebar from './AppSidebar.vue'
import GlobalSearchPalette from '@/components/search/GlobalSearchPalette.vue'
import { useAppStore } from '@/stores/app'

const route = useRoute()
const appStore = useAppStore()
const contentRef = ref<HTMLElement | null>(null)

const isImmersive = computed(() => route.meta.immersive === true)

/** 页面过渡名称：沉浸式工作区使用纯淡入淡出，其余页面保留轻微位移 */
const transitionName = ref(isImmersive.value ? 'page-fade' : 'page')

/**
 * 实际生效的沉浸式布局。
 *
 * 内容区 padding / 最大宽度在旧页面完全离场后再切换（before-enter），
 * 避免正在淡出的旧页面突然收缩，也保证新页面一入场就是最终布局。
 */
const appliedImmersive = ref(isImmersive.value)

/**
 * 笔记页等沉浸式页面默认收起应用级导航，把宽度留给三栏工作区。
 * 离开时恢复进入前的状态；如果用户手动展开过，则尊重用户的选择。
 */
let sidebarBeforeImmersive: boolean | null = null

/**
 * 路由一变化就同步侧边栏，让收起 / 展开动画与旧页面离场并行。
 * 如果等到新页面入场才收起，整个淡入过程中内容宽度都在变化，会明显晃动。
 */
function syncSidebar(immersive: boolean): void {
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
syncSidebar(isImmersive.value)

watch(isImmersive, (immersive, previous) => {
  // 只要往返涉及沉浸式页面，就用纯淡入淡出，避免工作区 / 画布出现位移
  transitionName.value = immersive || previous ? 'page-fade' : 'page'
  syncSidebar(immersive)
})

/** 旧页面已离场：切换内容区布局并归零滚动位置，再让新页面从顶部进入 */
function handleBeforeEnter(): void {
  appliedImmersive.value = isImmersive.value
  contentRef.value?.scrollTo({ top: 0 })
}

/** Ctrl + K / Cmd + K 打开或关闭全局搜索；搜索打开时 Esc 也能关闭 */
function handleGlobalKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && appStore.searchOpen) {
    event.preventDefault()
    appStore.closeSearch()
    return
  }

  if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== 'k') return

  event.preventDefault()
  appStore.toggleSearch()
}

onMounted(() => document.addEventListener('keydown', handleGlobalKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', handleGlobalKeydown))
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
            <Transition :name="transitionName" mode="out-in" @before-enter="handleBeforeEnter">
              <component :is="Component" :key="currentRoute.path" />
            </Transition>
          </RouterView>
        </div>
      </main>
    </div>

    <GlobalSearchPalette :open="appStore.searchOpen" @close="appStore.closeSearch()" />
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
