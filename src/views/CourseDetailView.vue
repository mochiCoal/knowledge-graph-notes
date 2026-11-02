<script setup lang="ts">
import PageHeader from '@/components/common/PageHeader.vue'
import PlaceholderPanel from '@/components/common/PlaceholderPanel.vue'

defineProps<{
  /** 由路由参数 courseId 注入（路由配置中开启了 props: true） */
  courseId: string
}>()

const features = ['课程基本信息', '知识点层级树', '课程下的笔记列表', '课程知识图谱入口']
</script>

<template>
  <div class="page">
    <PageHeader title="课程详情" description="查看课程下的知识点层级、笔记与知识图谱。">
      <template #actions>
        <RouterLink to="/courses" class="ghost-button">返回课程列表</RouterLink>
      </template>
    </PageHeader>

    <div class="course-context">
      <span>当前课程 ID</span>
      <code class="course-context__id">{{ courseId }}</code>
    </div>

    <PlaceholderPanel
      icon="courses"
      phase="阶段二"
      title="课程详情页待开发"
      description="该页面将通过路由参数加载指定课程，并作为知识点与笔记的容器页面。"
      :features="features"
    />
  </div>
</template>

<style scoped>
.course-context {
  display: inline-flex;
  align-self: flex-start;
  gap: 10px;
  align-items: center;
  padding: 8px 14px;
  color: var(--color-text-secondary);
  font-size: 12px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.course-context__id {
  color: var(--color-accent-strong);
  font-weight: 500;
}
</style>
