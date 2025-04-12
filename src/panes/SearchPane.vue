<script setup lang="ts">
import { computed, ref } from 'vue'
import { commands, SearchRespData, SearchSort } from '../bindings.ts'
import { useMessage } from 'naive-ui'
import ComicCard from '../components/ComicCard.vue'
import FloatLabelInput from '../components/FloatLabelInput.vue'
import { SearchOutlined } from '@vicons/antd'
import { SelectProps } from 'naive-ui'
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
const searchRespData = ref<SearchRespData>()

const searchPageCount = computed(() => {
  const PAGE_SIZE = 80
  if (searchRespData.value === undefined) {
    return 0
  }
  const total = searchRespData.value.total
  return Math.ceil(total / PAGE_SIZE)
})

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
  const searchResult = result.data
  if ('SearchRespData' in searchResult) {
    const respData = searchResult.SearchRespData
    if (respData.content.length === 0) {
      message.warning('什么都没有搜到，请尝试其他关键词')
      searching.value = false
      return
    }
    searchRespData.value = respData
    console.log(respData)
  } else if ('Comic' in searchResult) {
    const comic = searchResult.Comic
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
            <SearchOutlined />
          </n-icon>
        </template>
      </n-button>
    </n-input-group>

    <div v-if="searchRespData !== undefined" class="flex flex-col gap-row-2 overflow-auto box-border px-2">
      <comic-card
        v-for="comicInSearch in searchRespData.content"
        :key="comicInSearch.id"
        :comic-id="parseInt(comicInSearch.id)"
        :comic-title="comicInSearch.name"
        :comic-author="comicInSearch.author"
        :comic-category="comicInSearch.category"
        :comic-category-sub="comicInSearch.category_sub" />
    </div>

    <n-pagination
      v-if="searchPageCount > 0"
      class="box-border p-2 pt-0 mt-auto"
      :page-count="searchPageCount"
      :page="searchPage"
      @update:page="search(searchInput.trim(), $event, sortSelected)" />
  </div>
</template>
