<script setup lang="ts">
import { Comic, commands } from '../bindings.ts'
import { useStore } from '../store.ts'

const store = useStore()

const props = defineProps<{
  comic: Comic
}>()

function pickComic() {
  store.pickedComic = props.comic
  store.currentTabName = 'chapter'
}

async function exportCbz() {
  store.progressesPaneTabName = 'export'
  const result = await commands.exportCbz(props.comic)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
}

async function exportPdf() {
  store.progressesPaneTabName = 'export'
  const result = await commands.exportPdf(props.comic)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
}

async function showComicDownloadDirInFileManager() {
  if (store.config === undefined) {
    return
  }

  const comicDownloadDir = props.comic.comicDownloadDir

  if (comicDownloadDir === undefined || comicDownloadDir === null) {
    console.error('comicDownloadDir的值为undefined或null')
    return
  }

  const result = await commands.showPathInFileManager(comicDownloadDir)
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
        :src="`https://cdn-msp3.18comic.vip/media/albums/${comic.id}_3x4.jpg`"
        alt=""
        referrerpolicy="no-referrer"
        @click="pickComic" />
      <div class="flex flex-col w-full">
        <span
          class="font-bold text-xl line-clamp-3 cursor-pointer transition-colors duration-200 hover:text-blue-5"
          @click="pickComic">
          {{ comic.name }}
        </span>
        <span class="text-red">作者：{{ comic.author }}</span>
        <div class="flex mt-auto gap-col-2">
          <n-button size="tiny" @click="showComicDownloadDirInFileManager">打开下载目录</n-button>
          <n-button class="ml-auto" size="tiny" @click="exportCbz">导出cbz</n-button>
          <n-button size="tiny" @click="exportPdf">导出pdf</n-button>
        </div>
      </div>
    </div>
  </n-card>
</template>
