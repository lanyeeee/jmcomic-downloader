<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { commands, events } from '../bindings.ts'
import { open } from '@tauri-apps/plugin-dialog'
import { NProgress, useNotification } from 'naive-ui'
import { FolderOpenOutlined } from '@vicons/antd'
import { useStore } from '../store.ts'

const store = useStore()

type ProgressData = {
  comicTitle: string
  chapterTitle: string
  downloadedCount: number
  total: number
  percentage: number
  indicator: string
}

const notification = useNotification()

const progresses = ref<Map<number, ProgressData>>(new Map())
const overallProgress = ref<ProgressData>({
  comicTitle: '总进度',
  chapterTitle: '总进度',
  downloadedCount: 0,
  total: 0,
  percentage: 0,
  indicator: '',
})

const sortedProgresses = computed(() => {
  const progressesArray = Array.from(progresses.value.entries())
  progressesArray.sort((a, b) => {
    return b[1].total - a[1].total
  })
  return progressesArray
})

onMounted(async () => {
  await events.downloadEvent.listen(({ payload: downloadEvent }) => {
    if (downloadEvent.event == 'ChapterPending') {
      const { chapterId, comicTitle: comicTitle, chapterTitle } = downloadEvent.data
      const progressData: ProgressData = {
        comicTitle,
        chapterTitle,
        downloadedCount: 0,
        total: 0,
        percentage: 0,
        indicator: '',
      }
      progresses.value.set(chapterId, progressData)
    } else if (downloadEvent.event == 'ChapterStart') {
      const { chapterId, total } = downloadEvent.data
      const progressData = progresses.value.get(chapterId) as ProgressData | undefined
      if (progressData === undefined) {
        return
      }
      progressData.total = total
    } else if (downloadEvent.event == 'ChapterEnd') {
      const { chapterId, errMsg } = downloadEvent.data
      const progressData = progresses.value.get(chapterId) as ProgressData | undefined
      if (progressData === undefined) {
        return
      }
      if (errMsg !== null) {
        notification.warning({
          title: '下载章节失败',
          content: errMsg,
          meta: `${progressData.comicTitle} - ${progressData.chapterTitle}`,
        })
      }
      progresses.value.delete(chapterId)
    } else if (downloadEvent.event == 'ImageSuccess') {
      const { chapterId, current } = downloadEvent.data
      const progressData = progresses.value.get(chapterId) as ProgressData | undefined
      if (progressData === undefined) {
        return
      }
      progressData.downloadedCount = current
      progressData.percentage = Math.round((progressData.downloadedCount / progressData.total) * 100)
    } else if (downloadEvent.event == 'ImageError') {
      const { chapterId, url, errMsg } = downloadEvent.data
      const progressData = progresses.value.get(chapterId) as ProgressData | undefined
      if (progressData === undefined) {
        return
      }
      notification.warning({
        title: '下载图片失败',
        description: url,
        content: errMsg,
        meta: `${progressData.comicTitle} - ${progressData.chapterTitle}`,
      })
    } else if (downloadEvent.event == 'OverallSpeed') {
      const { speed } = downloadEvent.data
      overallProgress.value.indicator = speed
    } else if (downloadEvent.event == 'OverallUpdate') {
      const { percentage, downloadedImageCount, totalImageCount } = downloadEvent.data
      overallProgress.value.percentage = percentage
      overallProgress.value.downloadedCount = downloadedImageCount
      overallProgress.value.total = totalImageCount
    }
  })

  await events.setProxyEvent.listen(({ payload }) => {
    if (payload.event === 'Error') {
      notification.error({ title: '设置代理失败', description: payload.data.errMsg })
    }
  })
})

async function showDownloadDirInFileManager() {
  if (store.config === undefined) {
    return
  }
  const result = await commands.showPathInFileManager(store.config.downloadDir)
  if (result.status === 'error') {
    notification.error({ title: '打开下载目录失败', description: result.error })
  }
}

async function selectDownloadDir() {
  if (store.config === undefined) {
    return
  }

  const selectedDirPath = await open({ directory: true })
  if (selectedDirPath === null) {
    return
  }

  store.config.downloadDir = selectedDirPath
}
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col gap-2 flex-1 overflow-auto">
    <n-input-group class="box-border px-2 pt-2">
      <n-input-group-label size="small">下载目录</n-input-group-label>
      <n-input v-model:value="store.config.downloadDir" size="small" readonly @click="selectDownloadDir" />
      <n-button size="small" @click="showDownloadDirInFileManager">
        <template #icon>
          <n-icon>
            <FolderOpenOutlined />
          </n-icon>
        </template>
      </n-button>
    </n-input-group>
    <div class="grid grid-cols-[1fr_4fr_1fr]">
      <span class="text-ellipsis whitespace-nowrap overflow-hidden">{{ overallProgress.chapterTitle }}</span>
      <n-progress :percentage="overallProgress.percentage" indicator-placement="inside" :height="21">
        {{ overallProgress.indicator }}
      </n-progress>
      <span>{{ overallProgress.downloadedCount }}/{{ overallProgress.total }}</span>
    </div>
    <div class="h-full overflow-auto">
      <div
        class="grid grid-cols-[1fr_1fr_2fr]"
        v-for="[chapterId, { comicTitle, chapterTitle, percentage, downloadedCount, total }] in sortedProgresses"
        :key="chapterId">
        <span class="mb-1! text-ellipsis whitespace-nowrap overflow-hidden">{{ comicTitle }}</span>
        <span class="mb-1! text-ellipsis whitespace-nowrap overflow-hidden">{{ chapterTitle }}</span>
        <span v-if="total === 0">等待中</span>
        <n-progress v-else :percentage="percentage">{{ downloadedCount }}/{{ total }}</n-progress>
      </div>
    </div>
  </div>
</template>
