<script setup lang="ts">
import { SelectionArea, SelectionEvent } from '@viselect/vue'
import { nextTick, ref, watch } from 'vue'
import { commands } from '../bindings.ts'
import { useStore } from '../store.ts'

const store = useStore()

const dropdownX = ref<number>(0)
const dropdownY = ref<number>(0)
const showDropdown = ref<boolean>(false)
const dropdownOptions = [
  {
    label: '勾选',
    key: 'check',
    props: {
      onClick: () => {
        // 只有未勾选的才会被勾选
        ;[...selectedIds.value]
          .filter((id) => !checkedIds.value.includes(id))
          .forEach((id) => checkedIds.value.push(id))
        showDropdown.value = false
      },
    },
  },
  {
    label: '取消勾选',
    key: 'uncheck',
    props: {
      onClick: () => {
        checkedIds.value = checkedIds.value.filter((id) => !selectedIds.value.has(id))
        showDropdown.value = false
      },
    },
  },
  {
    label: '全选',
    key: 'check all',
    props: {
      onClick: () => {
        // 只有未锁定的才会被勾选
        store.pickedComic?.chapterInfos
          ?.filter((c) => c.isDownloaded !== true && !checkedIds.value.includes(c.chapterId))
          .forEach((c) => checkedIds.value.push(c.chapterId))
        showDropdown.value = false
      },
    },
  },
  {
    label: '取消全选',
    key: 'uncheck all',
    props: {
      onClick: () => {
        checkedIds.value.length = 0
        showDropdown.value = false
      },
    },
  },
]
const checkedIds = ref<number[]>([])
const selectedIds = ref<Set<number>>(new Set())
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()

watch(
  () => store.pickedComic,
  () => {
    checkedIds.value = []
    selectedIds.value.clear()
    selectionAreaRef.value?.selection?.clearSelection()
  },
)

function extractIds(elements: Element[]): number[] {
  return elements
    .map((element) => element.getAttribute('data-key'))
    .filter(Boolean)
    .map(Number)
    .filter((id) => {
      const chapterInfo = store.pickedComic?.chapterInfos.find((c) => c.chapterId === id)
      if (chapterInfo === undefined) {
        return false
      }
      return chapterInfo.isDownloaded !== true
    })
}

function unselectAll({ event, selection }: SelectionEvent) {
  if (!event?.ctrlKey && !event?.metaKey) {
    selection.clearSelection()
    selectedIds.value.clear()
  }
}

function updateSelectedIds({
  store: {
    changed: { added, removed },
  },
}: SelectionEvent) {
  extractIds(added).forEach((id) => selectedIds.value.add(id))
  extractIds(removed).forEach((id) => selectedIds.value.delete(id))
}

async function onContextMenu(e: MouseEvent) {
  showDropdown.value = false
  await nextTick()
  showDropdown.value = true
  dropdownX.value = e.clientX
  dropdownY.value = e.clientY
}

async function downloadChapters() {
  if (store.pickedComic === undefined) {
    return
  }
  // 创建下载任务前，先创建元数据
  const result = await commands.saveMetadata(store.pickedComic!)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  // 下载勾选的章节
  const chapterIdsToDownload = store.pickedComic.chapterInfos
    .filter((c) => c.isDownloaded !== true && checkedIds.value.includes(c.chapterId))
    .map((c) => c.chapterId)
  for (const chapterId of chapterIdsToDownload) {
    // 创建下载任务
    const result = await commands.createDownloadTask(store.pickedComic, chapterId)
    if (result.status === 'error') {
      console.error(result.error)
      continue
    }
    // 更新勾选状态
    const chapter = store.pickedComic.chapterInfos.find((chapter) => chapter.chapterId === chapterId)
    if (chapter !== undefined) {
      chapter.isDownloaded = true
      checkedIds.value = checkedIds.value.filter((id) => id !== chapterId)
    }
  }
}

async function refreshChapters() {
  if (store.pickedComic === undefined) {
    return
  }
  const result = await commands.getComic(store.pickedComic.id)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  store.pickedComic = result.data
}

async function showComicDownloadDirInFileManager() {
  if (store.pickedComic === undefined) {
    return
  }
  const result = await commands.showComicDownloadDirInFileManager(store.pickedComic.name)
  if (result.status === 'error') {
    console.error(result.error)
  }
}
</script>

<template>
  <div class="h-full flex flex-col gap-2 box-border">
    <div v-if="store.pickedComic !== undefined" class="flex items-center select-none pt-2 gap-1 px-2">
      左键拖动进行框选，右键打开菜单
      <n-button class="ml-auto" size="small" @click="refreshChapters">刷新</n-button>
      <n-button size="small" type="primary" @click="downloadChapters">下载勾选章节</n-button>
    </div>
    <n-empty v-if="store.pickedComic === undefined" description="请先进行漫画搜索"></n-empty>
    <SelectionArea
      v-else
      ref="selectionAreaRef"
      class="selection-container flex flex-col flex-1 px-2 pt-0 overflow-auto"
      :options="{ selectables: '.selectable', features: { deselectOnBlur: true } }"
      @contextmenu="onContextMenu"
      @move="updateSelectedIds"
      @start="unselectAll">
      <n-checkbox-group v-model:value="checkedIds" class="grid grid-cols-3 gap-1.5">
        <n-checkbox
          v-for="{ chapterId, chapterTitle, isDownloaded } in store.pickedComic.chapterInfos"
          :key="chapterId"
          :data-key="chapterId"
          class="selectable hover:bg-gray-200!"
          :value="chapterId"
          :label="chapterTitle"
          :disabled="isDownloaded"
          :class="{ selected: selectedIds.has(chapterId), downloaded: isDownloaded === true }" />
      </n-checkbox-group>
    </SelectionArea>

    <div v-if="store.pickedComic !== undefined" class="flex p-2 pt-0">
      <img
        class="w-24 mr-4"
        :src="`https://cdn-msp3.18comic.vip/media/albums/${store.pickedComic.id}_3x4.jpg`"
        alt=""
        referrerpolicy="no-referrer" />
      <div class="flex flex-col w-full justify-between">
        <div class="flex flex-col">
          <span class="font-bold text-xl line-clamp-2">{{ store.pickedComic.name }}</span>
          <span class="text-red">作者：{{ store.pickedComic.author }}</span>
          <span class="text-gray">标签：{{ store.pickedComic.tags }}</span>
          <n-button
            v-if="store.pickedComic.isDownloaded"
            class="flex mt-auto mr-auto gap-col-2"
            size="tiny"
            @click="showComicDownloadDirInFileManager">
            打开下载目录
          </n-button>
        </div>
      </div>
    </div>

    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="dropdownX"
      :y="dropdownY"
      :options="dropdownOptions"
      :show="showDropdown"
      :on-clickoutside="() => (showDropdown = false)" />
  </div>
</template>

<style scoped>
.selection-container {
  @apply select-none overflow-auto;
}

.selection-container .selected {
  @apply bg-[rgb(204,232,255)];
}

.selection-container .downloaded {
  @apply bg-[rgba(24,160,88,0.16)];
}

:deep(.n-checkbox__label) {
  @apply overflow-hidden whitespace-nowrap text-ellipsis;
}

:global(.selection-area) {
  @apply bg-[rgba(46,115,252,0.5)];
}
</style>
