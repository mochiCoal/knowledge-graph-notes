<script setup lang="ts">
defineProps<{
  /** 页面标题 */
  title: string
  /** 页面说明，可选 */
  description?: string
}>()
</script>

<template>
  <header class="page-header">
    <div class="page-header__text">
      <h1 class="page-header__title">{{ title }}</h1>
      <p v-if="description" class="page-header__description">{{ description }}</p>
    </div>

    <div v-if="$slots.actions" class="page-header__actions">
      <slot name="actions" />
    </div>
  </header>
</template>

<style scoped>
.page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
}

.page-header__text {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.page-header__title {
  font-size: 22px;
  font-weight: 600;
  line-height: 1.3;
  letter-spacing: -0.01em;
}

.page-header__description {
  max-width: 640px;
  color: var(--color-text-secondary);
  font-size: 13.5px;
}

.page-header__actions {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  gap: 8px;
  padding-top: 2px;
}
</style>
