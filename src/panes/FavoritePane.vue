<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { commands, events, GetFavoriteRespData, FavoriteSort } from '../bindings.ts'
import { MessageReactive, useMessage } from 'naive-ui'
import ComicCard from '../components/ComicCard.vue'
import { SelectProps } from 'naive-ui'
import { useStore } from '../store.ts'

const store = useStore()

const message = useMessage()

const sortOptions: SelectProps['options'] = [
  { label: '收藏时间', value: 'FavoriteTime' },
  { label: '更新时间', value: 'UpdateTime' },
]

const getFavoriteRespData = ref<GetFavoriteRespData>()
const sortSelected = ref<FavoriteSort>('FavoriteTime')
const pageSelected = ref<number>(1)
const folderIdSelected = ref<number>(0)

const favoritePageCount = computed(() => {
  const PAGE_SIZE = 20
  if (getFavoriteRespData.value === undefined) {
    return 0
  }
  const total = parseInt(getFavoriteRespData.value.total)
  return Math.ceil(total / PAGE_SIZE)
})
const folderOptions = computed<SelectProps['options']>(() => [
  { label: '全部', value: 0 },
  ...(getFavoriteRespData.value?.folder_list || []).map((folder) => ({
    label: folder.name,
    value: parseInt(folder.FID),
  })),
])

watch(
  () => store.userProfile,
  async () => {
    if (store.userProfile === undefined) {
      getFavoriteRespData.value = undefined
      return
    }
    await getFavourite(0, 1, 'FavoriteTime')
  },
  { immediate: true },
)

async function getFavourite(folderId: number, page: number, sort: FavoriteSort) {
  console.log(folderId, page, sort)
  folderIdSelected.value = folderId
  sortSelected.value = sort
  pageSelected.value = page
  const result = await commands.getFavoriteFolder(folderId, page, sort)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  getFavoriteRespData.value = result.data
}

async function syncFavoriteFolder() {
  const result = await commands.syncFavoriteFolder()
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  await getFavourite(0, 1, 'FavoriteTime')
  message.success('收藏夹已同步')
}

async function updateDownloadedFavoriteComic() {
  const result = await commands.updateDownloadedFavoriteComic()
  if (result.status === 'error') {
    updateMessage?.destroy()
    console.error(result.error)
    return
  }
}

let updateMessage: MessageReactive | undefined
onMounted(async () => {
  await events.updateDownloadedFavoriteComicEvent.listen(({ payload: updateEvent }) => {
    if (updateEvent.event === 'GettingFolders') {
      updateMessage = message.loading('正在获取收藏夹', { duration: 0 })
    } else if (updateEvent.event === 'GettingComics' && updateMessage !== undefined) {
      const { total } = updateEvent.data
      updateMessage.content = `正在获取收藏夹中的漫画(0/${total})`
    } else if (updateEvent.event === 'ComicGot' && updateMessage !== undefined) {
      const { current, total } = updateEvent.data
      updateMessage.content = `正在获取收藏夹中的漫画(${current}/${total})`
    } else if (updateEvent.event === 'DownloadTaskCreated' && updateMessage !== undefined) {
      updateMessage.type = 'success'
      updateMessage.content = '已为需要更新的章节创建下载任务'
      setTimeout(() => {
        updateMessage?.destroy()
        updateMessage = undefined
      }, 3000)
    }
  })
})
</script>

<template>
  <div class="h-full flex flex-col gap-2">
    <div class="flex box-border px-2 pt-2">
      <n-select
        v-model:value="folderIdSelected"
        :options="folderOptions"
        :show-checkmark="false"
        size="small"
        @update-value="getFavourite($event, 1, sortSelected)" />
      <n-select
        v-model:value="sortSelected"
        :options="sortOptions"
        :show-checkmark="false"
        size="small"
        @update-value="getFavourite(folderIdSelected, 1, $event)" />
      <n-button size="small" @click="updateDownloadedFavoriteComic">更新漫画</n-button>
      <n-button size="small" type="primary" secondary @click="syncFavoriteFolder">收藏不对点我</n-button>
    </div>

    <div v-if="getFavoriteRespData !== undefined" class="flex flex-col gap-row-2 overflow-auto box-border px-2">
      <comic-card
        v-for="comicInFavorite in getFavoriteRespData?.list"
        :key="comicInFavorite.id"
        :comic-id="parseInt(comicInFavorite.id)"
        :comic-title="comicInFavorite.name"
        :comic-author="comicInFavorite.author"
        :comic-category="comicInFavorite.category"
        :comic-category-sub="comicInFavorite.category_sub" />
    </div>

    <n-pagination
      class="box-border p-2 pt-0 mt-auto"
      :page-count="favoritePageCount"
      :page="pageSelected"
      @update:page="getFavourite(folderIdSelected, $event, sortSelected)" />
  </div>
</template>
