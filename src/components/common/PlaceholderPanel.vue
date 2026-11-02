<script setup lang="ts">
import AppIcon from './AppIcon.vue'
import type { IconName } from './icons'

withDefaults(
  defineProps<{
    /** 所属开发阶段，例如「阶段二」 */
    phase: string
    title: string
    description: string
    icon?: IconName
    /** 该模块计划实现的功能 */
    features?: string[]
  }>(),
  {
    icon: 'sparkles',
    features: () => [],
  },
)
</script>

<template>
  <section class="placeholder">
    <div class="placeholder__icon">
      <AppIcon :name="icon" :size="26" />
    </div>

    <span class="placeholder__phase">{{ phase }}</span>
    <h2 class="placeholder__title">{{ title }}</h2>
    <p class="placeholder__description">{{ description }}</p>

    <div v-if="features.length" class="placeholder__features">
      <span class="placeholder__features-label">计划功能</span>
      <ul>
        <li v-for="feature in features" :key="feature">{{ feature }}</li>
      </ul>
    </div>

    <p class="placeholder__note">页面骨架已就绪，数据与交互将在对应阶段接入。</p>
  </section>
</template>

<style scoped>
.placeholder {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
  padding: 34px 34px 26px;
  overflow: hidden;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
}

.placeholder::before {
  content: '';
  position: absolute;
  inset: 0 0 auto;
  height: 3px;
  background: var(--gradient-brand);
  opacity: 0.75;
}

.placeholder__icon {
  display: grid;
  place-items: center;
  width: 52px;
  height: 52px;
  margin-bottom: 6px;
  color: var(--color-accent-strong);
  background: var(--color-accent-soft);
  border-radius: 15px;
}

.placeholder__phase {
  padding: 2px 10px;
  color: var(--color-accent-strong);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  background: var(--color-accent-soft);
  border-radius: 999px;
}

.placeholder__title {
  margin-top: 2px;
  font-size: 18px;
  font-weight: 600;
}

.placeholder__description {
  max-width: 620px;
  color: var(--color-text-secondary);
  font-size: 13.5px;
}

.placeholder__features {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
  margin-top: 14px;
}

.placeholder__features-label {
  color: var(--color-text-muted);
  font-size: 11px;
  letter-spacing: 0.08em;
}

.placeholder__features ul {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 8px 20px;
}

.placeholder__features li {
  position: relative;
  padding-left: 16px;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.placeholder__features li::before {
  content: '';
  position: absolute;
  top: 9px;
  left: 2px;
  width: 5px;
  height: 5px;
  background: var(--color-accent);
  border-radius: 50%;
  opacity: 0.65;
}

.placeholder__note {
  width: 100%;
  margin-top: 20px;
  padding-top: 14px;
  color: var(--color-text-muted);
  font-size: 12px;
  border-top: 1px dashed var(--color-border);
}
</style>
