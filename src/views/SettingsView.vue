<script setup lang="ts">
import { computed } from 'vue'
import PageHeader from '@/components/common/PageHeader.vue'
import ThemeSwitcher from '@/components/common/ThemeSwitcher.vue'
import { THEME_OPTIONS } from '@/services/theme'
import { useThemeStore } from '@/stores/theme'

const themeStore = useThemeStore()

const currentThemeLabel = computed(
  () => THEME_OPTIONS.find((option) => option.value === themeStore.mode)?.label ?? '自动',
)
const resolvedThemeLabel = computed(() =>
  themeStore.resolvedTheme === 'dark' ? '当前为夜间配色' : '当前为日间配色',
)
</script>

<template>
  <div class="page">
    <PageHeader title="设置" description="调整应用外观，后续将补充数据备份与全局搜索。" />

    <section class="settings-card">
      <div class="settings-card__head">
        <h2>外观</h2>
        <p>主题切换立即生效并自动保存；自动模式会跟随 Windows 系统主题变化。</p>
      </div>

      <div class="settings-row">
        <div class="settings-row__label">
          <span>主题模式</span>
          <small>{{ currentThemeLabel }} · {{ resolvedThemeLabel }}</small>
        </div>
        <ThemeSwitcher :show-labels="true" />
      </div>
    </section>

    <section class="settings-card">
      <div class="settings-card__head">
        <h2>数据与搜索</h2>
        <p>以下功能将在后续阶段实现。</p>
      </div>

      <ul class="settings-list">
        <li>
          <span>数据库备份与恢复</span>
          <span class="settings-list__tag">阶段八</span>
        </li>
        <li>
          <span>全局搜索</span>
          <span class="settings-list__tag">阶段八</span>
        </li>
        <li>
          <span>最近编辑记录</span>
          <span class="settings-list__tag">阶段八</span>
        </li>
      </ul>
    </section>
  </div>
</template>

<style scoped>
.settings-card {
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
}

.settings-card__head {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 20px 22px 0;
}

.settings-card__head h2 {
  font-size: 14px;
  font-weight: 600;
}

.settings-card__head p {
  color: var(--color-text-muted);
  font-size: 12.5px;
}

.settings-row {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  align-items: center;
  justify-content: space-between;
  padding: 16px 22px 20px;
}

.settings-row__label {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.settings-row__label span {
  font-size: 13.5px;
}

.settings-row__label small {
  color: var(--color-text-muted);
  font-size: 11.5px;
}

.settings-list {
  padding: 6px 0 8px;
}

.settings-list li {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
  padding: 11px 22px;
  color: var(--color-text-secondary);
  font-size: 13px;
  border-top: 1px solid var(--color-border);
}

.settings-list li:first-child {
  border-top: none;
}

.settings-list__tag {
  padding: 1px 8px;
  color: var(--color-text-muted);
  font-size: 11px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}
</style>
