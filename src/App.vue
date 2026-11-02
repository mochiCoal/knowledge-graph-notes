<script setup lang="ts">
import AppLayout from '@/components/layout/AppLayout.vue'
import TitleBar from '@/components/layout/TitleBar.vue'
import WindowResizeHandles from '@/components/layout/WindowResizeHandles.vue'
</script>

<template>
  <div class="app-window">
    <TitleBar />
    <AppLayout />
    <WindowResizeHandles />
  </div>
</template>

<style scoped>
.app-window {
  display: flex;
  flex-direction: column;
  height: 100vh;
  height: 100dvh;
  overflow: hidden;
  background: var(--color-bg);
}
</style>
