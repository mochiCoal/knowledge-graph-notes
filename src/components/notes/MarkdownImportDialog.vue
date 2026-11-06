<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppIcon from '@/components/common/AppIcon.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import { coursesApi } from '@/services/courses'
import { pickMarkdownFile, readMarkdownFile } from '@/services/markdownFiles'
import { notesApi } from '@/services/notes'
import type { Course, KnowledgePoint, MarkdownFile, Note } from '@/types'

const props = withDefaults(
  defineProps<{
    open: boolean
    /** 打开时默认导入到的知识点 */
    defaultPointId?: string | null
  }>(),
  {
    defaultPointId: null,
  },
)

const emit = defineEmits<{
  close: []
  imported: [pointId: string, title: string]
}>()

const courses = ref<Course[]>([])
const points = ref<KnowledgePoint[]>([])
const courseId = ref('')
const pointId = ref('')

const file = ref<MarkdownFile | null>(null)
const title = ref('')

const readingFile = ref(false)
const importing = ref(false)
const errorMessage = ref('')
const existingNote = ref<Note | null>(null)

/** 详情请求序号：避免快速切换知识点时旧响应覆盖新提示 */
let noteRequest = 0

const pointsOfCourse = computed(() =>
  points.value.filter((point) => point.course_id === courseId.value),
)

const hasPoints = computed(() => points.value.length > 0)

const canImport = computed(
  () =>
    file.value !== null &&
    pointId.value !== '' &&
    title.value.trim() !== '' &&
    !importing.value &&
    !readingFile.value,
)

const previewText = computed(() => {
  if (!file.value) return ''
  const content = file.value.content
  return content.length > 2000
    ? `${content.slice(0, 2000)}\n\n…（仅预览前 2000 个字符）`
    : content
})

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** 生成知识点的完整路径，例如「线性表 / 链表」 */
function pathLabel(point: KnowledgePoint): string {
  const byId = new Map(points.value.map((item) => [item.id, item] as const))
  const parts: string[] = []
  const visited = new Set<string>()
  let current: KnowledgePoint | null = point

  while (current && !visited.has(current.id)) {
    visited.add(current.id)
    parts.unshift(current.title)
    current = current.parent_id ? (byId.get(current.parent_id) ?? null) : null
  }

  return parts.join(' / ')
}

const pointOptions = computed(() =>
  pointsOfCourse.value.map((point) => ({ id: point.id, label: pathLabel(point) })),
)

function reset(): void {
  file.value = null
  title.value = ''
  errorMessage.value = ''
  readingFile.value = false
  importing.value = false
  existingNote.value = null
}

/** 打开时加载课程与知识点，并选中默认知识点 */
watch(
  () => props.open,
  async (open) => {
    if (!open) return

    reset()
    noteRequest += 1

    try {
      const [courseList, pointList] = await Promise.all([
        coursesApi.list(),
        notesApi.listKnowledgePoints(),
      ])
      courses.value = courseList
      points.value = pointList
    } catch (error) {
      errorMessage.value = errorText(error)
      return
    }

    const preferred =
      props.defaultPointId ? points.value.find((point) => point.id === props.defaultPointId) : null
    const target = preferred ?? points.value[0] ?? null

    courseId.value = target?.course_id ?? courses.value[0]?.id ?? ''
    pointId.value = target?.id ?? ''
  },
)

/** 切换课程时，若当前知识点不属于该课程则自动选中第一项 */
watch(courseId, (next) => {
  if (!next) return

  const current = points.value.find((point) => point.id === pointId.value)
  if (current?.course_id === next) return

  const first = points.value.find((point) => point.course_id === next)
  pointId.value = first?.id ?? ''
})

/** 目标知识点已有笔记时提示会覆盖 */
watch(pointId, async (id) => {
  existingNote.value = null
  const requestId = ++noteRequest

  if (!id) return

  try {
    const found = await notesApi.getNote(id)
    if (requestId === noteRequest) {
      existingNote.value = found
    }
  } catch {
    // 只影响覆盖提示，静默忽略
  }
})

async function chooseFile(): Promise<void> {
  if (readingFile.value) return

  errorMessage.value = ''
  readingFile.value = true

  try {
    const path = await pickMarkdownFile()
    if (!path) return

    const loaded = await readMarkdownFile(path)
    file.value = loaded
    title.value = loaded.title
  } catch (error) {
    errorMessage.value = errorText(error)
  } finally {
    readingFile.value = false
  }
}

async function submit(): Promise<void> {
  const selected = file.value
  if (!selected || !pointId.value || importing.value) return

  const trimmedTitle = title.value.trim()
  if (!trimmedTitle) {
    errorMessage.value = '笔记标题不能为空'
    return
  }

  importing.value = true
  errorMessage.value = ''

  try {
    const saved = await notesApi.saveNote({
      noteId: existingNote.value?.id ?? null,
      knowledgePointId: pointId.value,
      title: trimmedTitle,
      content: selected.content,
    })

    emit('imported', saved.knowledge_point_id, saved.title)
    emit('close')
  } catch (error) {
    errorMessage.value = errorText(error)
  } finally {
    importing.value = false
  }
}
</script>

<template>
  <BaseModal :open="open" title="导入 Markdown 笔记" :width="580" @close="emit('close')">
    <div class="form-stack">
      <div class="file-picker">
        <div class="file-picker__info">
          <span class="file-picker__name" :class="{ 'is-empty': !file }">
            {{ file?.file_name ?? '尚未选择文件' }}
          </span>
          <span class="file-picker__hint">
            {{
              file
                ? `${file.character_count} 字 · 首行一级标题会自动作为笔记标题`
                : '支持 .md / .markdown / .txt，UTF-8 编码，最大 2 MB'
            }}
          </span>
        </div>

        <button type="button" class="ghost-button" :disabled="readingFile" @click="chooseFile">
          <AppIcon name="upload" :size="15" />
          {{ readingFile ? '读取中…' : file ? '重新选择' : '选择文件' }}
        </button>
      </div>

      <template v-if="file">
        <label class="field">
          <span class="field__label">笔记标题</span>
          <input v-model="title" class="input" type="text" maxlength="200" />
        </label>

        <div class="field-row">
          <label class="field field--half">
            <span class="field__label">所属课程</span>
            <select v-model="courseId" class="select">
              <option v-for="course in courses" :key="course.id" :value="course.id">
                {{ course.name }}
              </option>
            </select>
          </label>

          <label class="field field--half">
            <span class="field__label">导入到知识点</span>
            <select v-model="pointId" class="select">
              <option v-for="option in pointOptions" :key="option.id" :value="option.id">
                {{ option.label }}
              </option>
            </select>
          </label>
        </div>

        <p v-if="existingNote" class="overwrite-hint">
          <AppIcon name="alert" :size="14" />
          该知识点已有笔记「{{ existingNote.title }}」，导入后会覆盖其标题与正文。
        </p>

        <div class="field">
          <span class="field__label">
            正文预览
            <span class="field__hint">导入后可在笔记工作区继续编辑</span>
          </span>
          <pre class="preview">{{ previewText }}</pre>
        </div>
      </template>

      <p v-else-if="!hasPoints" class="import-hint">
        还没有知识点，请先在「知识点管理」中创建知识点，再导入笔记。
      </p>
      <p v-else class="import-hint">
        选择文件后可以先预览内容，再决定导入到哪个知识点。
      </p>

      <p v-if="errorMessage" class="form-error">{{ errorMessage }}</p>
    </div>

    <template #footer>
      <button type="button" class="ghost-button" @click="emit('close')">取消</button>
      <button type="button" class="primary-button" :disabled="!canImport" @click="submit">
        {{ importing ? '导入中…' : '导入笔记' }}
      </button>
    </template>
  </BaseModal>
</template>

<style scoped>
.file-picker {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  background: var(--color-surface-soft);
  border: 1px dashed var(--color-border-strong);
  border-radius: var(--radius-sm);
}

.file-picker__info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.file-picker__name {
  overflow: hidden;
  color: var(--color-text);
  font-size: 12.5px;
  font-weight: 500;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.file-picker__name.is-empty {
  color: var(--color-text-muted);
  font-weight: 400;
}

.file-picker__hint {
  color: var(--color-text-muted);
  font-size: 11px;
  line-height: 1.5;
}

.file-picker .ghost-button {
  flex: 0 0 auto;
}

.field-row {
  display: flex;
  gap: 12px;
}

.field--half {
  flex: 1;
  min-width: 0;
}

.overwrite-hint {
  display: flex;
  gap: 7px;
  align-items: flex-start;
  padding: 9px 11px;
  color: var(--color-danger);
  font-size: 11.5px;
  line-height: 1.6;
  background: var(--color-danger-soft);
  border-radius: var(--radius-sm);
}

.import-hint {
  color: var(--color-text-muted);
  font-size: 12px;
  line-height: 1.7;
}

.preview {
  max-height: 220px;
  padding: 11px 13px;
  overflow: auto;
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.7;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  background: var(--color-surface-soft);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
}
</style>
