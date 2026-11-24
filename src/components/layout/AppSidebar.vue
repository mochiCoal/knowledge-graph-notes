<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import AppIcon from '@/components/common/AppIcon.vue'
import { APP_VERSION } from '@/config'
import { navItems } from '@/router/navigation'
import { useAppStore } from '@/stores/app'

const route = useRoute()
const appStore = useAppStore()
const { sidebarCollapsed } = storeToRefs(appStore)

/** 当前路由是否属于该导航项（含子路由，例如课程详情归属于课程管理） */
function isActive(path: string): boolean {
  if (path === '/') return route.path === '/'
  return route.path === path || route.path.startsWith(`${path}/`)
}
</script>

<template>
  <aside class="sidebar" :class="{ 'is-collapsed': sidebarCollapsed }">
    <nav class="sidebar__nav" aria-label="主导航">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ 'is-active': isActive(item.path) }"
        :title="sidebarCollapsed ? item.title : undefined"
      >
        <span class="nav-item__icon">
          <AppIcon :name="item.icon" :size="18" />
        </span>
        <span class="nav-item__label">{{ item.title }}</span>
      </RouterLink>
    </nav>

    <footer class="sidebar__footer">
      <span class="sidebar__status" aria-hidden="true"></span>
      <div class="sidebar__footer-text">
        <span>本地 SQLite 已就绪</span>
        <span>v{{ APP_VERSION }}</span>
      </div>
    </footer>
  </aside>
</template>

<style scoped>
.sidebar {
  display: flex;
  flex: 0 0 auto;
  flex-direction: column;
  width: var(--sidebar-width);
  height: 100%;
  padding: 14px 10px 12px;
  overflow: hidden;
  background: var(--color-sidebar);
  border-right: 1px solid var(--color-border);
  transition: width var(--transition-base);
}

.sidebar.is-collapsed {
  width: var(--sidebar-width-collapsed);
  padding-right: 12px;
  padding-left: 12px;
}

.sidebar__nav {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 4px;
  padding-top: 4px;
  overflow-x: hidden;
  overflow-y: auto;
}

.nav-item {
  display: flex;
  gap: 10px;
  align-items: center;
  height: 34px;
  padding: 0 9px;
  color: var(--color-text-secondary);
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  border-radius: 6px;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.nav-item:hover {
  color: var(--color-text);
  background: var(--color-surface-hover);
}

.nav-item.is-active {
  color: var(--color-text);
  font-weight: 600;
  background: var(--color-surface-hover);
}

.nav-item__icon {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 20px;
}

.nav-item__label {
  overflow: hidden;
  text-overflow: ellipsis;
}

.sidebar__footer {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-top: 12px;
  padding: 12px 10px 4px;
  color: var(--color-text-muted);
  border-top: 1px solid var(--color-border);
}

.sidebar__status {
  flex: 0 0 auto;
  width: 7px;
  height: 7px;
  background: var(--color-success);
  border-radius: 50%;
}

.sidebar__footer-text {
  display: flex;
  flex-direction: column;
  font-size: 11px;
  line-height: 1.45;
  white-space: nowrap;
}

/* 折叠状态：隐藏文字，图标居中显示 */
.is-collapsed .nav-item__label,
.is-collapsed .sidebar__footer-text {
  display: none;
}

.is-collapsed .nav-item {
  justify-content: center;
  gap: 0;
  padding: 0;
}

.is-collapsed .sidebar__footer {
  justify-content: center;
  padding-right: 0;
  padding-left: 0;
}
</style>
