<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import NoteMetaPanel from '@/components/notes/NoteMetaPanel.vue'
import NoteWorkspace from '@/components/notes/NoteWorkspace.vue'
import NotesSidebar from '@/components/notes/NotesSidebar.vue'
import { useNotesStore } from '@/stores/notes'
import { useWorkspaceStore } from '@/stores/workspace'

const route = useRoute()
const notes = useNotesStore()
const workspace = useWorkspaceStore()

onMounted(async () => {
  // 课程管理页可能改动课程 / 知识点：已初始化时静默刷新目录，
  // 首次进入才显示加载状态。
  if (notes.initialized) {
    await notes.refreshCatalog()
  } else {
    await notes.initialize()
  }

  // 知识点管理页的「在笔记中打开」会带上 point 查询参数
  const pointId = typeof route.query.point === 'string' ? route.query.point : ''
  if (pointId && pointId !== notes.selectedPointId) {
    await notes.selectPoint(pointId)
  }
})

onBeforeUnmount(() => {
  // 离开笔记页前把未保存的草稿落盘
  void notes.flushSave()
})
</script>

<template>
  <div class="notes-layout">
    <NotesSidebar />
    <NoteWorkspace />
    <NoteMetaPanel v-if="workspace.metaOpen" />
  </div>
</template>

<style scoped>
.notes-layout {
  display: flex;
  width: 100%;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--color-bg);
}
</style>
