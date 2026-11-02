<template>
  <div class="not-found">
    <span class="not-found__code">404</span>
    <h1 class="not-found__title">页面不存在</h1>
    <p class="not-found__text">你访问的页面可能已被移除，或者地址输入有误。</p>
    <RouterLink to="/" class="primary-button">返回首页</RouterLink>
  </div>
</template>

<style scoped>
.not-found {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  text-align: center;
}

.not-found__code {
  font-size: 64px;
  font-weight: 700;
  line-height: 1;
  letter-spacing: -0.04em;
  background: var(--gradient-brand);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.not-found__title {
  margin-top: 6px;
  font-size: 20px;
  font-weight: 600;
}

.not-found__text {
  color: var(--color-text-secondary);
  font-size: 13px;
}

.not-found .primary-button {
  margin-top: 12px;
}
</style>
