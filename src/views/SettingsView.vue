<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import ConfirmDialog from '@/components/common/ConfirmDialog.vue'
import PageHeader from '@/components/common/PageHeader.vue'
import ThemeSwitcher from '@/components/common/ThemeSwitcher.vue'
import { APP_NAME, APP_VERSION, DEVELOPMENT_STAGE } from '@/config'
import {
  databaseApi,
  pickDatabaseBackupPath,
  pickDatabaseRestoreFile,
  revealDatabaseFile,
} from '@/services/database'
import { THEME_OPTIONS } from '@/services/theme'
import { useAppStore } from '@/stores/app'
import { useThemeStore } from '@/stores/theme'
import type { DatabaseInfo } from '@/types'
import { formatDate, formatFileSize, formatNumber } from '@/utils/format'

const appStore = useAppStore()
const themeStore = useThemeStore()

const database = ref<DatabaseInfo | null>(null)
const isLoading = ref(true)
const loadError = ref('')
const actionError = ref('')
const notice = ref('')

const backupBusy = ref(false)
const restoreBusy = ref(false)
const restoreCandidate = ref<string | null>(null)

let reloadTimer: number | null = null

const currentThemeLabel = computed(
  () => THEME_OPTIONS.find((option) => option.value === themeStore.mode)?.label ?? '自动',
)
const resolvedThemeLabel = computed(() =>
  themeStore.resolvedTheme === 'dark' ? '当前为夜间配色' : '当前为日间配色',
)

const showRestoreConfirm = computed(() => restoreCandidate.value !== null)

const restoreMessage = computed(() => {
  const source = restoreCandidate.value
  if (!source) return ''

  return (
    `确定从「${source}」恢复吗？当前数据库会被备份文件覆盖，` +
    '恢复前会自动保存一份当前数据的安全副本。恢复完成后应用将重新加载。'
  )
})

const dataCounts = computed(() => {
  const info = database.value
  if (!info) return []

  return [
    { label: '课程', value: info.course_count },
    { label: '知识点', value: info.knowledge_point_count },
    { label: '笔记', value: info.note_count },
    { label: '标签', value: info.tag_count },
    { label: '关联', value: info.relation_count },
  ]
})

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

async function loadDatabaseInfo(): Promise<void> {
  isLoading.value = true
  loadError.value = ''

  try {
    database.value = await databaseApi.info()
  } catch (error) {
    loadError.value = errorText(error)
  } finally {
    isLoading.value = false
  }
}

async function backupDatabase(): Promise<void> {
  if (backupBusy.value) return

  actionError.value = ''
  notice.value = ''

  try {
    const destination = await pickDatabaseBackupPath()
    if (!destination) return

    backupBusy.value = true
    const result = await databaseApi.backup(destination)
    notice.value = `已备份到 ${result.path}（${formatFileSize(result.size_bytes)}）`

    await loadDatabaseInfo()
  } catch (error) {
    actionError.value = errorText(error)
  } finally {
    backupBusy.value = false
  }
}

async function chooseRestoreFile(): Promise<void> {
  if (restoreBusy.value) return

  actionError.value = ''
  notice.value = ''

  try {
    const source = await pickDatabaseRestoreFile()
    if (source) restoreCandidate.value = source
  } catch (error) {
    actionError.value = errorText(error)
  }
}

function cancelRestore(): void {
  if (restoreBusy.value) return
  restoreCandidate.value = null
}

async function confirmRestore(): Promise<void> {
  const source = restoreCandidate.value
  if (!source || restoreBusy.value) return

  restoreBusy.value = true
  actionError.value = ''
  notice.value = ''

  try {
    const result = await databaseApi.restore(source)
    restoreCandidate.value = null
    database.value = result.database
    notice.value =
      `已从备份恢复，恢复前的数据保存在 ${result.safety_backup_path}。` +
      '应用即将重新加载以读取最新数据…'

    reloadTimer = window.setTimeout(() => {
      window.location.reload()
    }, 1800)
  } catch (error) {
    actionError.value = errorText(error)
    restoreCandidate.value = null
  } finally {
    restoreBusy.value = false
  }
}

async function openDatabaseDirectory(): Promise<void> {
  if (!database.value) return

  actionError.value = ''

  try {
    await revealDatabaseFile(database.value.path)
  } catch (error) {
    actionError.value = errorText(error)
  }
}

onMounted(() => {
  void loadDatabaseInfo()
})

onBeforeUnmount(() => {
  if (reloadTimer !== null) window.clearTimeout(reloadTimer)
})
</script>

<template>
  <div class="page settings-page">
    <PageHeader
      title="设置"
      description="调整应用外观、备份与恢复本地数据库，并查看应用信息。"
    />

    <div v-if="actionError" class="alert-bar" role="alert">
      <AppIcon name="alert" :size="16" />
      <span>{{ actionError }}</span>
      <button type="button" class="alert-bar__dismiss" @click="actionError = ''">关闭</button>
    </div>

    <div v-if="notice" class="notice-bar" role="status">
      <AppIcon name="check" :size="15" />
      <span>{{ notice }}</span>
      <button type="button" class="alert-bar__dismiss" @click="notice = ''">关闭</button>
    </div>

    <section class="settings-card">
      <div class="settings-card__head">
        <div class="settings-card__title">
          <span class="settings-card__icon">
            <AppIcon name="sun" :size="16" />
          </span>
          <div>
            <h2>外观</h2>
            <p>主题切换立即生效并自动保存；自动模式会跟随 Windows 系统主题变化。</p>
          </div>
        </div>
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
        <div class="settings-card__title">
          <span class="settings-card__icon">
            <AppIcon name="database" :size="16" />
          </span>
          <div>
            <h2>数据与备份</h2>
            <p>备份会把整个知识库写成一个 SQLite 文件，恢复会覆盖当前数据库。</p>
          </div>
        </div>
      </div>

      <div v-if="isLoading" class="settings-state" role="status">
        <span class="spinner" />
        <span>正在读取数据库信息…</span>
      </div>

      <div v-else-if="loadError" class="settings-state settings-state--error" role="alert">
        <AppIcon name="alert" :size="18" />
        <span>{{ loadError }}</span>
        <button type="button" class="ghost-button" @click="loadDatabaseInfo">重新加载</button>
      </div>

      <template v-else-if="database">
        <dl class="data-grid">
          <div class="data-grid__row data-grid__row--path">
            <dt>数据库文件</dt>
            <dd class="data-grid__path">{{ database.path }}</dd>
          </div>
          <div class="data-grid__row">
            <dt>文件大小</dt>
            <dd>{{ formatFileSize(database.size_bytes) }}</dd>
          </div>
          <div class="data-grid__row">
            <dt>最后修改</dt>
            <dd>{{ database.modified_at ? formatDate(database.modified_at) : '—' }}</dd>
          </div>
          <div class="data-grid__row">
            <dt>SQLite 版本</dt>
            <dd>{{ database.sqlite_version }}</dd>
          </div>
        </dl>

        <div class="data-counts" aria-label="数据库记录数">
          <span v-for="item in dataCounts" :key="item.label" class="data-count">
            <strong>{{ formatNumber(item.value) }}</strong>
            {{ item.label }}
          </span>
        </div>

        <div class="settings-actions">
          <button
            type="button"
            class="primary-button"
            :disabled="backupBusy"
            @click="backupDatabase"
          >
            <AppIcon name="download" :size="15" />
            {{ backupBusy ? '正在备份…' : '备份数据库' }}
          </button>
          <button
            type="button"
            class="ghost-button"
            :disabled="restoreBusy"
            @click="chooseRestoreFile"
          >
            <AppIcon name="upload" :size="15" />
            恢复数据库
          </button>
          <button type="button" class="ghost-button" @click="openDatabaseDirectory">
            <AppIcon name="folder" :size="15" />
            在文件夹中显示
          </button>
        </div>

        <p class="settings-note">
          <AppIcon name="alert" :size="14" />
          恢复前会自动在当前数据库旁保存一份
          <code>knowledge_graph-before-restore-*.db</code> 安全副本，可用于回滚。
        </p>
      </template>
    </section>

    <section class="settings-card">
      <div class="settings-card__head">
        <div class="settings-card__title">
          <span class="settings-card__icon">
            <AppIcon name="search" :size="16" />
          </span>
          <div>
            <h2>全局搜索</h2>
            <p>在任意页面按 Ctrl + K（macOS 为 Cmd + K）唤起搜索面板。</p>
          </div>
        </div>
      </div>

      <div class="settings-row">
        <div class="settings-row__label">
          <span>搜索整个知识库</span>
          <small>课程、知识点、笔记、标签与知识点关联</small>
        </div>
        <button type="button" class="ghost-button" @click="appStore.openSearch()">
          <AppIcon name="search" :size="15" />
          打开搜索
          <kbd class="settings-kbd">Ctrl K</kbd>
        </button>
      </div>
    </section>

    <section class="settings-card">
      <div class="settings-card__head">
        <div class="settings-card__title">
          <span class="settings-card__icon">
            <AppIcon name="graph" :size="16" />
          </span>
          <div>
            <h2>关于</h2>
            <p>应用信息与数据存储方式。</p>
          </div>
        </div>
      </div>

      <dl class="about-list">
        <div class="about-list__row">
          <dt>应用名称</dt>
          <dd>{{ APP_NAME }}</dd>
        </div>
        <div class="about-list__row">
          <dt>版本</dt>
          <dd>v{{ APP_VERSION }}</dd>
        </div>
        <div class="about-list__row">
          <dt>开发阶段</dt>
          <dd>{{ DEVELOPMENT_STAGE }}</dd>
        </div>
        <div class="about-list__row">
          <dt>技术栈</dt>
          <dd>Tauri 2 · Vue 3 · Rust · SQLite · Vue Flow</dd>
        </div>
        <div class="about-list__row">
          <dt>数据存储</dt>
          <dd>完全本地，不依赖网络；笔记使用 Markdown 格式。</dd>
        </div>
      </dl>
    </section>

    <ConfirmDialog
      :open="showRestoreConfirm"
      title="恢复数据库"
      :message="restoreMessage"
      confirm-text="恢复并覆盖"
      :busy="restoreBusy"
      @cancel="cancelRestore"
      @confirm="confirmRestore"
    />
  </div>
</template>

<style scoped>
.settings-page {
  gap: 20px;
}

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
  padding: 18px 22px 0;
}

.settings-card__title {
  display: flex;
  gap: 11px;
  align-items: flex-start;
}

.settings-card__icon {
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  width: 30px;
  height: 30px;
  margin-top: 1px;
  color: var(--color-text-secondary);
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 7px;
}

.settings-card__head h2 {
  font-size: 14px;
  font-weight: 600;
}

.settings-card__head p {
  margin-top: 2px;
  color: var(--color-text-muted);
  font-size: 12.5px;
  line-height: 1.6;
}

.settings-row {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  align-items: center;
  justify-content: space-between;
  padding: 14px 22px 18px;
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

.settings-state {
  display: flex;
  gap: 9px;
  align-items: center;
  padding: 16px 22px 20px;
  color: var(--color-text-muted);
  font-size: 12.5px;
}

.settings-state--error {
  color: var(--color-danger);
}

.settings-state .ghost-button {
  margin-left: auto;
  height: 28px;
  font-size: 12px;
}

.data-grid {
  display: flex;
  flex-direction: column;
  margin: 12px 22px 0;
  overflow: hidden;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.data-grid__row {
  display: grid;
  grid-template-columns: minmax(110px, 0.8fr) minmax(0, 2.2fr);
  gap: 14px;
  padding: 9px 12px;
  border-top: 1px solid var(--color-border);
}

.data-grid__row:first-child {
  border-top: 0;
}

.data-grid dt {
  color: var(--color-text-muted);
  font-size: 12px;
}

.data-grid dd {
  min-width: 0;
  color: var(--color-text-secondary);
  font-size: 12.5px;
}

.data-grid__row--path dd {
  overflow-wrap: anywhere;
}

.data-grid__path {
  font-family: var(--font-mono);
  font-size: 11.5px;
  line-height: 1.55;
}

.data-counts {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 12px 22px 0;
}

.data-count {
  display: inline-flex;
  gap: 5px;
  align-items: baseline;
  padding: 3px 9px;
  color: var(--color-text-muted);
  font-size: 11px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 999px;
}

.data-count strong {
  color: var(--color-text);
  font-size: 12px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.settings-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 14px 22px 0;
}

.settings-note {
  display: flex;
  gap: 6px;
  align-items: flex-start;
  margin: 12px 22px 18px;
  padding: 9px 11px;
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.6;
  background: var(--color-surface-soft);
  border-radius: var(--radius-sm);
}

.settings-note svg {
  flex: 0 0 auto;
  margin-top: 1px;
}

.settings-note code {
  font-size: 10.5px;
}

.settings-kbd {
  padding: 0 4px;
  color: var(--color-text-muted);
  font-family: var(--font-sans);
  font-size: 9.5px;
  line-height: 16px;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: 4px;
}

.about-list {
  padding: 8px 0 6px;
}

.about-list__row {
  display: grid;
  grid-template-columns: minmax(90px, 0.4fr) minmax(0, 1.6fr);
  gap: 16px;
  padding: 9px 22px;
  border-top: 1px solid var(--color-border);
}

.about-list__row:first-child {
  border-top: 0;
}

.about-list dt {
  color: var(--color-text-muted);
  font-size: 12px;
}

.about-list dd {
  color: var(--color-text-secondary);
  font-size: 12.5px;
  line-height: 1.6;
}

.alert-bar,
.notice-bar {
  display: flex;
  gap: 9px;
  align-items: center;
  padding: 10px 12px;
  font-size: 12px;
  border-radius: var(--radius-sm);
}

.alert-bar {
  color: var(--color-danger);
  background: var(--color-danger-soft);
}

.notice-bar {
  color: var(--color-success);
  background: var(--color-success-soft);
}

.alert-bar span,
.notice-bar span {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}

.alert-bar__dismiss {
  flex: 0 0 auto;
  padding: 0;
  color: inherit;
  font-size: 11.5px;
  background: transparent;
  border: 0;
  text-decoration: underline;
  text-underline-offset: 3px;
}

@media (max-width: 600px) {
  .settings-row,
  .data-grid,
  .settings-actions {
    align-items: stretch;
  }

  .data-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .data-grid__row {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .about-list__row {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
}
</style>
