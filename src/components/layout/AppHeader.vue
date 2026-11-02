<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import ThemeSwitcher from '@/components/common/ThemeSwitcher.vue'
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

    <div class="app-header__actions">
      <ThemeSwitcher />
    </div>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  flex: 0 0 auto;
  gap: 12px;
  align-items: center;
  height: var(--header-height);
  padding: 0 18px 0 14px;
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  transition:
    background-color var(--transition-base),
    border-color var(--transition-base);
}

.breadcrumb {
  display: flex;
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

.app-header__actions {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-left: auto;
}
</style>
