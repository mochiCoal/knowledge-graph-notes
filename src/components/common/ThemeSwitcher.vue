<script setup lang="ts">
import AppIcon from './AppIcon.vue'
import type { IconName } from './icons'
import { THEME_OPTIONS, type ThemeMode } from '@/services/theme'
import { useThemeStore } from '@/stores/theme'

withDefaults(
  defineProps<{
    /** 是否显示文字标签（目前仅在设置页使用，统一显示文字标签） */
    showLabels?: boolean
  }>(),
  {
    showLabels: false,
  },
)

const themeStore = useThemeStore()

const ICONS: Record<ThemeMode, IconName> = {
  auto: 'monitor',
  light: 'sun',
  dark: 'moon',
}
</script>

<template>
  <div class="theme-switcher" role="group" aria-label="主题模式">
    <button
      v-for="option in THEME_OPTIONS"
      :key="option.value"
      type="button"
      class="theme-switcher__option"
      :class="{ 'is-active': themeStore.mode === option.value }"
      :aria-pressed="themeStore.mode === option.value"
      :title="showLabels ? option.description : option.label"
      @click="themeStore.setMode(option.value)"
    >
      <AppIcon :name="ICONS[option.value]" :size="15" />
      <span v-if="showLabels" class="theme-switcher__label">{{ option.label }}</span>
    </button>
  </div>
</template>

<style scoped>
.theme-switcher {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 9px;
}

.theme-switcher__option {
  display: inline-flex;
  gap: 5px;
  align-items: center;
  justify-content: center;
  height: 26px;
  padding: 0 8px;
  color: var(--color-text-secondary);
  font-size: 12px;
  background: transparent;
  border: none;
  border-radius: 6px;
  transition:
    background-color var(--transition-fast),
    color var(--transition-fast);
}

.theme-switcher__option:hover {
  color: var(--color-text);
}

.theme-switcher__option.is-active {
  color: var(--color-text);
  background: var(--color-surface);
  box-shadow: var(--shadow-sm);
}

.theme-switcher__label {
  line-height: 1;
}
</style>
