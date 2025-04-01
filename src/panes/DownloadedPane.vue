<script setup lang="ts">
import { Comic, commands } from '../bindings.ts'
import { CurrentTabName } from '../types.ts'
import { computed, ref, watch } from 'vue'
import { useNotification } from 'naive-ui'
import DownloadedComicCard from '../components/DownloadedComicCard.vue'

const notification = useNotification()

const pickedComic = defineModel<Comic | undefined>('pickedComic', { required: true })
const currentTabName = defineModel<CurrentTabName>('currentTabName', { required: true })

const { currentPage, pageCount, currentPageComics } = useDownloadedComics()

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
</script>

<template>
  <div class="h-full flex flex-col gap-2">
    <div class="flex flex-col gap-row-2 overflow-auto box-border px-2 pt-2">
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
