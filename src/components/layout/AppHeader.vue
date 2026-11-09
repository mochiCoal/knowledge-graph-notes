<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import { useAppStore } from '@/stores/app'

interface Crumb {
  title: string
  path: string
}

const route = useRoute()
const appStore = useAppStore()
const { sidebarCollapsed } = storeToRefs(appStore)

/** 根据当前路由的 title / parent 元信息生成面包屑 */
const crumbs = computed<Crumb[]>(() => {
  if (route.path === '/') {
    return [{ title: '首页', path: '/' }]
  }

  const items: Crumb[] = [{ title: '首页', path: '/' }]

  if (route.meta.parent) {
    items.push(route.meta.parent)
  }

  items.push({ title: route.meta.title ?? '未知页面', path: route.path })

  return items
})

const collapseLabel = computed(() => (sidebarCollapsed.value ? '展开侧边栏' : '收起侧边栏'))
</script>

<template>
  <header class="app-header">
    <button
      type="button"
      class="icon-button"
      :aria-label="collapseLabel"
      :title="collapseLabel"
      @click="appStore.toggleSidebar()"
    >
      <AppIcon name="panel-left" :size="18" />
    </button>

    <nav class="breadcrumb" aria-label="面包屑导航">
      <template v-for="(crumb, index) in crumbs" :key="crumb.path">
        <span v-if="index > 0" class="breadcrumb__separator" aria-hidden="true">/</span>
        <RouterLink v-if="index < crumbs.length - 1" :to="crumb.path" class="breadcrumb__link">
          {{ crumb.title }}
        </RouterLink>
        <span v-else class="breadcrumb__current">{{ crumb.title }}</span>
      </template>
    </nav>

    <button
      type="button"
      class="search-trigger"
      :class="{ 'is-active': appStore.searchOpen }"
      aria-label="打开全局搜索"
      title="全局搜索（Ctrl + K）"
      @click="appStore.openSearch()"
    >
      <AppIcon name="search" :size="15" />
      <span class="search-trigger__label">搜索</span>
      <kbd class="search-trigger__kbd">Ctrl K</kbd>
    </button>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  flex: 0 0 auto;
  gap: 12px;
  align-items: center;
  height: var(--header-height);
  padding: 0 20px 0 14px;
  background: transparent;
}

.breadcrumb {
  display: flex;
  flex: 1;
  gap: 8px;
  align-items: center;
  min-width: 0;
  font-size: 12.5px;
}

.breadcrumb__link {
  color: var(--color-text-muted);
  transition: color var(--transition-fast);
}

.breadcrumb__link:hover {
  color: var(--color-accent);
}

.breadcrumb__separator {
  color: var(--color-border-strong);
}

.breadcrumb__current {
  overflow: hidden;
  color: var(--color-text);
  font-weight: 500;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.search-trigger {
  display: inline-flex;
  flex: 0 0 auto;
  gap: 6px;
  align-items: center;
  height: 28px;
  padding: 0 8px;
  color: var(--color-text-muted);
  font-size: 11.5px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  transition:
    color var(--transition-fast),
    border-color var(--transition-fast),
    background-color var(--transition-fast);
}

.search-trigger:hover,
.search-trigger.is-active {
  color: var(--color-text);
  background: var(--color-surface-hover);
  border-color: var(--color-border-strong);
}

.search-trigger__label {
  line-height: 1;
}

.search-trigger__kbd {
  display: inline-grid;
  height: 17px;
  padding: 0 4px;
  place-items: center;
  color: var(--color-text-muted);
  font-family: var(--font-sans);
  font-size: 9.5px;
  line-height: 1;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 4px;
}

@media (max-width: 720px) {
  .search-trigger__label,
  .search-trigger__kbd {
    display: none;
  }

  .search-trigger {
    width: 30px;
    padding: 0;
    justify-content: center;
  }
}

</style>
