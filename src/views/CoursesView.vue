<script setup lang="ts">
import PageHeader from '@/components/common/PageHeader.vue'
import PlaceholderPanel from '@/components/common/PlaceholderPanel.vue'

const features = ['课程列表与空状态', '新增 / 编辑 / 删除课程', '课程数据校验', '课程详情入口']
</script>

<template>
  <div class="page">
    <PageHeader
      title="课程管理"
      description="课程是知识点与笔记的组织入口，本模块提供课程的完整管理能力。"
    />

    <PlaceholderPanel
      icon="courses"
      phase="阶段二"
      title="课程管理模块待开发"
      description="这里将展示全部课程，并提供新增、编辑、删除课程以及进入课程详情的能力。"
      :features="features"
    />
  </div>
</template>
