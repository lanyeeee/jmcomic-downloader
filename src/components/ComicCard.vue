<script setup lang="ts">
import { CategoryRespData, CategorySubRespData, commands } from '../bindings.ts'
import { useStore } from '../store.ts'

const store = useStore()

const props = defineProps<{
  comicId: number
  comicTitle: string
  comicAuthor: string
  comicCategory: CategoryRespData
  comicCategorySub: CategorySubRespData
  comicDownloaded: boolean
  comicDownloadDir: string
}>()

async function pickComic() {
  const result = await commands.getComic(props.comicId)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  store.pickedComic = result.data
  store.currentTabName = 'chapter'
}

async function downloadComic() {
  const result = await commands.downloadComic(props.comicId)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
}

async function showComicDownloadDirInFileManager() {
  if (store.config === undefined) {
    return
  }
  const result = await commands.showPathInFileManager(props.comicDownloadDir)
  if (result.status === 'error') {
    console.error(result.error)
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
        @click="pickComic" />
      <div class="flex flex-col w-full justify-between">
        <div class="flex flex-col">
          <span
            class="font-bold text-xl line-clamp-3 cursor-pointer transition-colors duration-200 hover:text-blue-5"
            @click="pickComic">
            {{ comicTitle }}
          </span>
          <span class="text-red">作者：{{ comicAuthor }}</span>
          <span class="text-gray">分类：{{ comicCategory.title }} {{ comicCategorySub.title }}</span>
        </div>
        <div class="flex">
          <n-button v-if="comicDownloaded" size="tiny" @click="showComicDownloadDirInFileManager">
            打开下载目录
          </n-button>
          <n-button size="tiny" class="ml-auto" @click="downloadComic">一键下载所有章节</n-button>
        </div>
      </div>
    </div>
  </n-card>
</template>
