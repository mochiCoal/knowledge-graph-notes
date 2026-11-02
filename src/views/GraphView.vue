<script setup lang="ts">
import PageHeader from '@/components/common/PageHeader.vue'
import PlaceholderPanel from '@/components/common/PlaceholderPanel.vue'

const features = [
  '知识点节点与关联连线',
  '拖拽 / 缩放 / 平移',
  '节点详情面板',
  '按课程、层级、关联类型筛选',
]
</script>

<template>
  <div class="page">
    <PageHeader
      title="知识图谱"
      description="以交互式图谱展示课程知识结构，并呈现知识点之间的语义关联。"
    />

    <PlaceholderPanel
      icon="graph"
      phase="阶段七"
      title="知识图谱模块待开发"
      description="将使用 Vue Flow 渲染知识点节点与关联边，支持拖拽、缩放、平移以及节点详情联动。"
      :features="features"
    />
  </div>
</template>
