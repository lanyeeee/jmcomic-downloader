<script setup lang="ts">
import { Comic, commands } from '../bindings.ts'
import { computed, ref, watch } from 'vue'
import DownloadedComicCard from '../components/DownloadedComicCard.vue'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpenOutlined } from '@vicons/antd'
import { useStore } from '../store.ts'

const store = useStore()

const PAGE_SIZE = 20
// 已下载的漫画
const downloadedComics = ref<Comic[]>([])
// 当前页码
const currentPage = ref<number>(1)
// 总页数
const pageCount = computed<number>(() => {
  return Math.ceil(downloadedComics.value.length / PAGE_SIZE)
})
// 当前页的漫画
const currentPageComics = computed<Comic[]>(() => {
  const start = (currentPage.value - 1) * PAGE_SIZE
  const end = start + PAGE_SIZE
  return downloadedComics.value.slice(start, end)
})

// 监听标签页变化，更新下载的漫画列表
watch(
  () => store.currentTabName,
  async () => {
    if (store.currentTabName !== 'downloaded') {
      return
    }

    downloadedComics.value = await commands.getDownloadedComics()
  },
  { immediate: true },
)

async function selectExportDir() {
  if (store.config === undefined) {
    return
  }

  const selectedDirPath = await open({ directory: true })
  if (selectedDirPath === null) {
    return
  }

  store.config.exportDir = selectedDirPath
}

async function showExportDirInFileManager() {
  if (store.config === undefined) {
    return
  }
  const result = await commands.showPathInFileManager(store.config.exportDir)
  if (result.status === 'error') {
    console.error(result.error)
  }
}
</script>

<template>
  <div v-if="store.config !== undefined" class="h-full flex flex-col gap-2">
    <n-input-group class="box-border px-2 pt-2">
      <n-input-group-label size="small">导出目录</n-input-group-label>
      <n-input v-model:value="store.config.exportDir" size="small" readonly @click="selectExportDir" />
      <n-button size="small" @click="showExportDirInFileManager">
        <template #icon>
          <n-icon>
            <FolderOpenOutlined />
          </n-icon>
        </template>
      </n-button>
    </n-input-group>
    <div class="flex flex-col gap-row-2 overflow-auto box-border px-2">
      <DownloadedComicCard v-for="comic in currentPageComics" :key="comic.id" :comic="comic" />
    </div>
    <n-pagination
      class="box-border p-2 pt-0 mt-auto"
      :page-count="pageCount"
      :page="currentPage"
      @update:page="currentPage = $event" />
  </div>
</template>
