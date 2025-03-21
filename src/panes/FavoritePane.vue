<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { Comic, commands, events, GetFavoriteRespData, FavoriteSort, GetUserProfileRespData } from '../bindings.ts'
import { MessageReactive, useMessage, useNotification } from 'naive-ui'
import ComicCard from '../components/ComicCard.vue'

const message = useMessage()
const notification = useNotification()

const props = defineProps<{
  userProfile: GetUserProfileRespData | undefined
}>()

const selectedComic = defineModel<Comic | undefined>('selectedComic', { required: true })
const currentTabName = defineModel<'search' | 'favorite' | 'chapter'>('currentTabName', { required: true })

const sortOptions: { label: string; value: string }[] = [
  { label: '收藏时间', value: 'FavoriteTime' },
  { label: '更新时间', value: 'UpdateTime' }
]

const getFavoriteRespData = ref<GetFavoriteRespData>()
const sortSelected = ref<FavoriteSort>('FavoriteTime')
const pageSelected = ref<number>(1)
const folderIdSelected = ref<number>(0)

const favoritePageCount = computed(() => {
  if (getFavoriteRespData.value === undefined) {
    return 0
  }
  const total = parseInt(getFavoriteRespData.value.total)
  return Math.floor(total / 20) + 1
})
const folderOptions = computed<{ label: string; value: number }[]>(() => [
  { label: '全部', value: 0 },
  ...(getFavoriteRespData.value?.folder_list || []).map((folder) => ({
    label: folder.name,
    value: parseInt(folder.FID)
  }))
])

async function getFavourite(folderId: number, page: number, sort: FavoriteSort) {
  console.log(folderId, page, sort)
  folderIdSelected.value = folderId
  sortSelected.value = sort
  pageSelected.value = page
  const result = await commands.getFavoriteFolder(folderId, page, sort)
  if (result.status === 'error') {
    notification.error({ title: '获取收藏失败', description: result.error })
    return
  }
  getFavoriteRespData.value = result.data
}

async function syncFavoriteFolder() {
  const result = await commands.syncFavoriteFolder()
  if (result.status === 'error') {
    notification.error({ title: '获取收藏失败', description: result.error })
    return
  }
  await getFavourite(0, 1, 'FavoriteTime')
  message.success('收藏夹已同步')
}

async function updateDownloadedFavoriteComic() {
  const result = await commands.updateDownloadedFavoriteComic()
  if (result.status === 'error') {
    updateMessage?.destroy()
    notification.error({ title: '更新收藏夹中已下载的漫画失败', description: result.error })
    return
  }
}

// TODO: 把这个watch移到上面去
watch(
  () => props.userProfile,
  async () => {
    if (props.userProfile === undefined) {
      getFavoriteRespData.value = undefined
      return
    }
    await getFavourite(0, 1, 'FavoriteTime')
  },
  { immediate: true }
)

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
  <div class="h-full flex flex-col">
    <div class="flex">
      <n-select
        v-model:value="folderIdSelected"
        :options="folderOptions"
        :show-checkmark="false"
        size="tiny"
        @update-value="getFavourite($event, 1, sortSelected)" />
      <n-select
        v-model:value="sortSelected"
        :options="sortOptions"
        :show-checkmark="false"
        size="tiny"
        @update-value="getFavourite(folderIdSelected, 1, $event)" />
      <n-button size="tiny" @click="updateDownloadedFavoriteComic">更新已下载的漫画</n-button>
      <n-button size="tiny" type="primary" secondary @click="syncFavoriteFolder">收藏夹不对请点我</n-button>
    </div>
    <div v-if="getFavoriteRespData !== undefined" class="flex flex-col gap-row-1 overflow-auto p-2">
      <div class="flex flex-col gap-row-2 overflow-auto">
        <comic-card
          v-for="comicInFavorite in getFavoriteRespData?.list"
          :key="comicInFavorite.id"
          :comic-info="comicInFavorite"
          v-model:selected-comic="selectedComic"
          v-model:current-tab-name="currentTabName" />
      </div>
      <n-pagination
        :page-count="favoritePageCount"
        :page="pageSelected"
        @update:page="getFavourite(folderIdSelected, $event, sortSelected)" />
    </div>
  </div>
</template>
