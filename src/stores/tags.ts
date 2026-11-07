import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { notesApi } from '@/services/notes'
import { tagsApi } from '@/services/tags'
import type { NoteSummary, Tag, TagSummary } from '@/types'

function errorText(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return String(error)
}

/**
 * 标签管理数据
 *
 * - 左侧标签列表带关联笔记数量，支持按名称搜索；
 * - 选中标签后加载该标签下的笔记，可直接打开对应笔记；
 * - 增删改后立即刷新列表与当前标签的笔记。
 */
export const useTagsStore = defineStore('tags', () => {
  const tags = ref<TagSummary[]>([])
  const keyword = ref('')
  const selectedTagId = ref<string | null>(null)
  const notes = ref<NoteSummary[]>([])

  const isLoading = ref(false)
  const isLoadingNotes = ref(false)
  const errorMessage = ref<string | null>(null)
  const notesError = ref<string | null>(null)
  const initialized = ref(false)

  /** 笔记请求序号：避免切换标签时旧响应覆盖新列表 */
  let notesRequest = 0

  const selectedTag = computed(
    () => tags.value.find((tag) => tag.id === selectedTagId.value) ?? null,
  )

  /** 所有标签关联的笔记数量之和 */
  const totalTaggedNotes = computed(() =>
    tags.value.reduce((sum, tag) => sum + tag.note_count, 0),
  )

  async function loadTags(searchKeyword: string): Promise<void> {
    const trimmed = searchKeyword.trim()
    tags.value = trimmed ? await tagsApi.search(trimmed) : await tagsApi.overview()
  }

  async function loadNotes(tagId: string): Promise<void> {
    const requestId = ++notesRequest
    isLoadingNotes.value = true
    notesError.value = null

    try {
      const result = await notesApi.listNotes(tagId)
      if (requestId === notesRequest) {
        notes.value = result
      }
    } catch (error) {
      if (requestId === notesRequest) {
        notes.value = []
        notesError.value = errorText(error)
      }
    } finally {
      if (requestId === notesRequest) {
        isLoadingNotes.value = false
      }
    }
  }

  async function selectTag(tagId: string): Promise<void> {
    selectedTagId.value = tagId
    await loadNotes(tagId)
  }

  /** 刷新标签列表，并尽量保持（或切换到指定）选中标签 */
  async function refreshTags(preferredTagId?: string | null): Promise<void> {
    try {
      await loadTags(keyword.value)

      const target =
        (preferredTagId && tags.value.some((tag) => tag.id === preferredTagId)
          ? preferredTagId
          : null) ??
        (selectedTagId.value && tags.value.some((tag) => tag.id === selectedTagId.value)
          ? selectedTagId.value
          : null) ??
        tags.value[0]?.id ??
        null

      if (target) {
        await selectTag(target)
      } else {
        selectedTagId.value = null
        notes.value = []
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    }
  }

  async function initialize(force = false): Promise<void> {
    if (initialized.value && !force) return

    isLoading.value = true
    errorMessage.value = null

    try {
      await loadTags('')
      if (tags.value[0]) {
        await selectTag(tags.value[0].id)
      }
      initialized.value = true
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  /** 顶部刷新 / 错误重试 */
  async function refresh(): Promise<void> {
    isLoading.value = true
    errorMessage.value = null

    try {
      await refreshTags(selectedTagId.value)
      initialized.value = true
    } finally {
      isLoading.value = false
    }
  }

  /** 按名称搜索标签（防抖由调用方控制） */
  async function search(value: string): Promise<void> {
    keyword.value = value
    isLoading.value = true
    errorMessage.value = null

    try {
      await loadTags(value)

      const target =
        (selectedTagId.value && tags.value.some((tag) => tag.id === selectedTagId.value)
          ? selectedTagId.value
          : null) ??
        tags.value[0]?.id ??
        null

      if (target) {
        await selectTag(target)
      } else {
        selectedTagId.value = null
        notes.value = []
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      isLoading.value = false
    }
  }

  async function createTag(name: string): Promise<Tag> {
    const tag = await tagsApi.create(name)
    await refreshTags(tag.id)
    return tag
  }

  async function renameTag(tagId: string, name: string): Promise<Tag> {
    const tag = await tagsApi.update(tagId, name)
    await refreshTags(tagId)
    return tag
  }

  async function deleteTag(tagId: string): Promise<number> {
    const detached = await tagsApi.remove(tagId)
    await refreshTags(null)
    return detached
  }

  function dismissError(): void {
    errorMessage.value = null
  }

  return {
    // 数据
    tags,
    notes,
    keyword,
    selectedTagId,
    selectedTag,
    totalTaggedNotes,
    // 状态
    isLoading,
    isLoadingNotes,
    errorMessage,
    notesError,
    initialized,
    // 行为
    initialize,
    refresh,
    search,
    selectTag,
    createTag,
    renameTag,
    deleteTag,
    dismissError,
  }
})
