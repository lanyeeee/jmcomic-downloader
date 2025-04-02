<script setup lang="ts">
import { Comic, commands, Config, events } from '../bindings.ts'
import { CurrentTabName } from '../types.ts'
import { computed, ref, watch, onMounted } from 'vue'
import { useNotification, MessageReactive, useMessage } from 'naive-ui'
import DownloadedComicCard from '../components/DownloadedComicCard.vue'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpenOutlined } from '@vicons/antd'

interface ProgressData {
  comicTitle: string
  current: number
  total: number
  progressMessage: MessageReactive
}

const notification = useNotification()
const message = useMessage()

const config = defineModel<Config>('config', { required: true })
const pickedComic = defineModel<Comic | undefined>('pickedComic', { required: true })
const currentTabName = defineModel<CurrentTabName>('currentTabName', { required: true })

const { currentPage, pageCount, currentPageComics } = useDownloadedComics()
useProgressTracking()

function useDownloadedComics() {
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
    () => currentTabName.value,
    async () => {
      if (currentTabName.value !== 'downloaded') {
        return
      }

      const result = await commands.getDownloadedComics()
      if (result.status === 'error') {
        notification.error({ title: '获取本地库存失败', description: result.error })
        return
      }
      downloadedComics.value = result.data
    },
    { immediate: true },
  )

  return { currentPage, pageCount, currentPageComics }
}

function useProgressTracking() {
  const progresses = new Map<string, ProgressData>(new Map())

  // 处理导出CBZ事件
  async function handleExportCbzEvents() {
    await events.exportCbzEvent.listen(async ({ payload: exportEvent }) => {
      if (exportEvent.event === 'Start') {
        const { uuid, comicTitle, total } = exportEvent.data
        progresses.set(uuid, {
          comicTitle,
          current: 0,
          total,
          progressMessage: message.loading(
            () => {
              const progressData = progresses.get(uuid)
              if (progressData === undefined) return ''
              return `${progressData.comicTitle} 正在导出cbz(${progressData.current}/${progressData.total})`
            },
            { duration: 0 },
          ),
        })
      } else if (exportEvent.event === 'Progress') {
        const { uuid, current } = exportEvent.data
        const progressData = progresses.get(uuid)
        if (progressData) {
          progressData.current = current
        }
      } else if (exportEvent.event === 'Error') {
        const { uuid } = exportEvent.data
        const progressData = progresses.get(uuid)
        if (progressData) {
          progressData.progressMessage.type = 'error'
          progressData.progressMessage.content = `${progressData.comicTitle} 导出cbz失败(${progressData.current}/${progressData.total})`
          setTimeout(() => {
            progressData.progressMessage.destroy()
            progresses.delete(uuid)
          }, 3000)
        }
      } else if (exportEvent.event === 'End') {
        const { uuid } = exportEvent.data
        const progressData = progresses.get(uuid)
        if (progressData) {
          progressData.progressMessage.type = 'success'
          progressData.progressMessage.content = `${progressData.comicTitle} 导出cbz完成(${progressData.current}/${progressData.total})`
          setTimeout(() => {
            progressData.progressMessage.destroy()
            progresses.delete(uuid)
          }, 3000)
        }
      }
    })
  }

  // 处理导出PDF事件
  async function handleExportPdfEvents() {
    await events.exportPdfEvent.listen(async ({ payload: exportEvent }) => {
      if (exportEvent.event === 'CreateStart') {
        const { uuid, comicTitle, total } = exportEvent.data
        progresses.set(uuid, {
          comicTitle,
          current: 0,
          total,
          progressMessage: message.loading(
            () => {
              const progressData = progresses.get(uuid)
              if (progressData === undefined) return ''
              return `${progressData.comicTitle} 正在创建pdf(${progressData.current}/${progressData.total})`
            },
            { duration: 0 },
          ),
        })
      } else if (exportEvent.event === 'CreateProgress') {
        const { uuid, current } = exportEvent.data
        const progressData = progresses.get(uuid)
        if (progressData) {
          progressData.current = current
        }
      } else if (exportEvent.event === 'CreateError') {
        const { uuid } = exportEvent.data
        const progressData = progresses.get(uuid)
        if (progressData) {
          progressData.progressMessage.type = 'error'
          progressData.progressMessage.content = `${progressData.comicTitle} 创建pdf失败(${progressData.current}/${progressData.total})`
          setTimeout(() => {
            progressData.progressMessage.destroy()
            progresses.delete(uuid)
          }, 3000)
        }
      } else if (exportEvent.event === 'CreateEnd') {
        const { uuid } = exportEvent.data
        const progressData = progresses.get(uuid)
        if (progressData) {
          progressData.progressMessage.type = 'success'
          progressData.progressMessage.content = `${progressData.comicTitle} 创建pdf完成(${progressData.current}/${progressData.total})`
          setTimeout(() => {
            progressData.progressMessage.destroy()
            progresses.delete(uuid)
          }, 3000)
        }
      } else if (exportEvent.event === 'MergeStart') {
        const { uuid, comicTitle } = exportEvent.data
        progresses.set(uuid, {
          comicTitle,
          current: 0,
          total: 1,
          progressMessage: message.loading(
            () => {
              const progressData = progresses.get(uuid)
              if (progressData === undefined) return ''
              return `${progressData.comicTitle} 正在合并cbz(${progressData.current}/${progressData.total})`
            },
            { duration: 0 },
          ),
        })
      } else if (exportEvent.event === 'MergeError') {
        const { uuid } = exportEvent.data
        const progressData = progresses.get(uuid)
        if (progressData) {
          progressData.progressMessage.type = 'error'
          progressData.progressMessage.content = `${progressData.comicTitle} 合并pdf失败(${progressData.current}/${progressData.total})`
          setTimeout(() => {
            progressData.progressMessage.destroy()
            progresses.delete(uuid)
          }, 3000)
        }
      } else if (exportEvent.event === 'MergeEnd') {
        const { uuid } = exportEvent.data
        const progressData = progresses.get(uuid)
        if (progressData) {
          progressData.current = 1
          progressData.progressMessage.type = 'success'
          progressData.progressMessage.content = `${progressData.comicTitle} 合并pdf完成(${progressData.current}/${progressData.total})`
          setTimeout(() => {
            progressData.progressMessage.destroy()
            progresses.delete(uuid)
          }, 3000)
        }
      }
    })
  }

  // 监听导出事件
  onMounted(async () => {
    await handleExportCbzEvents()
    await handleExportPdfEvents()
  })
}

async function selectExportDir() {
  const selectedDirPath = await open({ directory: true })
  if (selectedDirPath === null) {
    return
  }
  config.value.exportDir = selectedDirPath
}

async function showExportDirInFileManager() {
  if (config.value === undefined) {
    return
  }
  const result = await commands.showPathInFileManager(config.value.exportDir)
  if (result.status === 'error') {
    notification.error({ title: '打开下载目录失败', description: result.error })
  }
}
</script>

<template>
  <div class="h-full flex flex-col gap-2">
    <n-input-group class="box-border px-2 pt-2">
      <n-input-group-label size="small">导出目录</n-input-group-label>
      <n-input v-model:value="config.exportDir" size="small" readonly @click="selectExportDir" />
      <n-button size="small" @click="showExportDirInFileManager">
        <template #icon>
          <n-icon>
            <FolderOpenOutlined />
          </n-icon>
        </template>
      </n-button>
    </n-input-group>
    <div class="flex flex-col gap-row-2 overflow-auto box-border px-2">
      <downloaded-comic-card
        v-for="comic in currentPageComics"
        :key="comic.id"
        :comic="comic"
        v-model:picked-comic="pickedComic"
        v-model:current-tab-name="currentTabName" />
    </div>
    <n-pagination
      class="box-border p-2 pt-0 mt-auto"
      :page-count="pageCount"
      :page="currentPage"
      @update:page="currentPage = $event" />
  </div>
</template>
