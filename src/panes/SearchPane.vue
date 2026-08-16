<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { commands, SearchSort } from '../bindings.ts'
import { DropdownOption, SelectProps, useMessage } from 'naive-ui'
import ComicCard from '../components/ComicCard.vue'
import FloatLabelInput from '../components/FloatLabelInput.vue'
import { PhMagnifyingGlass } from '@phosphor-icons/vue'
import { useStore } from '../store.ts'

const store = useStore()

const message = useMessage()

const sortOptions: SelectProps['options'] = [
  { label: '最新', value: 'Latest' },
  { label: '最多点击', value: 'View' },
  { label: '最多图片', value: 'Picture' },
  { label: '最多爱心', value: 'Like' },
]

const searchInput = ref<string>('')
const searching = ref<boolean>(false)
const sortSelected = ref<SearchSort>('Latest')
const searchPage = ref<number>(1)
const contextMenuX = ref<number>(0)
const contextMenuY = ref<number>(0)
const contextMenuShowing = ref<boolean>(false)
const searchInputElement = ref<HTMLInputElement>()
let selectionStart = 0
let selectionEnd = 0

const contextMenuOptions: DropdownOption[] = [
  {
    label: '粘贴',
    key: 'paste',
    props: {
      onClick: pasteFromClipboard,
    },
  },
]

const searchPageCount = computed(() => {
  const PAGE_SIZE = 80
  if (store.searchResult === undefined) {
    return 0
  }
  const total = store.searchResult.total
  return Math.ceil(total / PAGE_SIZE)
})

async function showContextMenu(event: MouseEvent) {
  const target = event.target
  if (!(target instanceof Element)) {
    return
  }
  const inputElement = target.closest('.n-input')?.querySelector('input')
  if (!(inputElement instanceof HTMLInputElement)) {
    return
  }

  searchInputElement.value = inputElement
  selectionStart = inputElement.selectionStart ?? inputElement.value.length
  selectionEnd = inputElement.selectionEnd ?? inputElement.value.length
  contextMenuShowing.value = false
  await nextTick()
  contextMenuX.value = event.clientX
  contextMenuY.value = event.clientY
  contextMenuShowing.value = true
}

async function pasteFromClipboard() {
  contextMenuShowing.value = false
  try {
    const clipboardText = await navigator.clipboard.readText()
    searchInput.value =
      searchInput.value.slice(0, selectionStart) + clipboardText + searchInput.value.slice(selectionEnd)
    const caretPosition = selectionStart + clipboardText.length
    await nextTick()
    searchInputElement.value?.focus()
    searchInputElement.value?.setSelectionRange(caretPosition, caretPosition)
  } catch (error) {
    console.error(error)
    message.error('无法读取剪贴板，请检查系统剪贴板权限')
  }
}

async function search(keyword: string, page: number, sort: SearchSort) {
  if (searching.value) {
    message.warning('有搜索正在进行，请稍后再试')
    return
  }

  searching.value = true
  console.log(keyword, page, sort)
  searchPage.value = page

  const result = await commands.search(keyword, page, sort)
  if (result.status === 'error') {
    console.error(result.error)
    searching.value = false
    return
  }
  const searchResultVariant = result.data
  if ('SearchResult' in searchResultVariant) {
    const respData = searchResultVariant.SearchResult
    if (respData.content.length === 0) {
      message.warning('什么都没有搜到，请尝试其他关键词')
      searching.value = false
      return
    }
    store.searchResult = respData
    console.log(respData)
  } else if ('Comic' in searchResultVariant) {
    const comic = searchResultVariant.Comic
    store.pickedComic = comic
    console.log(comic)
    store.currentTabName = 'chapter'
  }

  searching.value = false
}
</script>

<template>
  <div class="h-full flex flex-col gap-2">
    <n-input-group class="box-border px-2 pt-2">
      <FloatLabelInput
        label="关键词(jm号也可以)"
        size="small"
        v-model:value="searchInput"
        clearable
        @contextmenu="showContextMenu"
        @keydown.enter="search(searchInput.trim(), 1, sortSelected)" />
      <n-select
        class="w-45%"
        v-model:value="sortSelected"
        :options="sortOptions"
        :show-checkmark="false"
        size="small"
        @update-value="search(searchInput.trim(), 1, $event)" />
      <n-button
        :loading="searching"
        type="primary"
        size="small"
        class="w-15%"
        @click="search(searchInput.trim(), 1, sortSelected)">
        <template #icon>
          <n-icon size="22">
            <PhMagnifyingGlass />
          </n-icon>
        </template>
      </n-button>
    </n-input-group>

    <div v-if="store.searchResult !== undefined" class="flex flex-col gap-row-2 overflow-auto box-border px-2">
      <ComicCard
        v-for="comicInSearch in store.searchResult.content"
        :key="comicInSearch.id"
        :comic-id="comicInSearch.id"
        :comic-title="comicInSearch.name"
        :comic-author="comicInSearch.author"
        :comic-category="comicInSearch.category"
        :comic-category-sub="comicInSearch.categorySub"
        :comic-downloaded="comicInSearch.isDownloaded"
        :comic-download-dir="comicInSearch.comicDownloadDir" />
    </div>

    <n-pagination
      v-if="searchPageCount > 0"
      class="box-border p-2 pt-0 mt-auto"
      :page-count="searchPageCount"
      :page="searchPage"
      @update:page="search(searchInput.trim(), $event, sortSelected)" />

    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="contextMenuX"
      :y="contextMenuY"
      :options="contextMenuOptions"
      :show="contextMenuShowing"
      :on-clickoutside="() => (contextMenuShowing = false)" />
  </div>
</template>
