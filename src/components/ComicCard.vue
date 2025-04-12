<script setup lang="ts">
import { CategoryRespData, CategorySubRespData, commands } from '../bindings.ts'
import { useStore } from '../store.ts'

const store = useStore()

defineProps<{
  comicId: number
  comicTitle: string
  comicAuthor: string
  comicCategory: CategoryRespData
  comicCategorySub: CategorySubRespData
}>()

async function pickComic(aid: number) {
  const result = await commands.getComic(aid)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  store.pickedComic = result.data
  store.currentTabName = 'chapter'
}

async function downloadComic(aid: number) {
  const result = await commands.downloadComic(aid)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
}
</script>

<template>
  <n-card content-style="padding: 0.25rem;" hoverable>
    <div class="flex">
      <img
        class="w-24 object-cover mr-4 cursor-pointer transition-transform duration-200 hover:scale-106"
        :src="`https://cdn-msp3.18comic.vip/media/albums/${comicId}_3x4.jpg`"
        alt=""
        referrerpolicy="no-referrer"
        @click="pickComic(comicId)" />
      <div class="flex flex-col w-full justify-between">
        <div class="flex flex-col">
          <span
            class="font-bold text-xl line-clamp-3 cursor-pointer transition-colors duration-200 hover:text-blue-5"
            @click="pickComic(comicId)">
            {{ comicTitle }}
          </span>
          <span class="text-red">作者：{{ comicAuthor }}</span>
          <span class="text-gray">分类：{{ comicCategory.title }} {{ comicCategorySub.title }}</span>
        </div>
        <n-button size="tiny" class="ml-auto" @click="downloadComic(comicId)">一键下载所有章节</n-button>
      </div>
    </div>
  </n-card>
</template>
