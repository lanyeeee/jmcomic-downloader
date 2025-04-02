<script setup lang="ts">
import { Comic, commands } from '../bindings.ts'
import { CurrentTabName } from '../types.ts'
import { useNotification } from 'naive-ui'

const notification = useNotification()

const props = defineProps<{
  comic: Comic
}>()

const pickedComic = defineModel<Comic | undefined>('pickedComic', { required: true })
const currentTabName = defineModel<CurrentTabName>('currentTabName', { required: true })

function pickComic() {
  pickedComic.value = props.comic
  currentTabName.value = 'chapter'
}

async function exportCbz() {
  const result = await commands.exportCbz(props.comic)
  if (result.status === 'error') {
    notification.error({ title: '导出cbz失败', description: result.error })
    return
  }
}

async function exportPdf() {
  const result = await commands.exportPdf(props.comic)
  if (result.status === 'error') {
    notification.error({ title: '导出pdf失败', description: result.error })
    return
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
        <div class="flex ml-auto mt-auto gap-col-2">
          <n-button size="tiny" @click="exportCbz">导出cbz</n-button>
          <n-button size="tiny" @click="exportPdf">导出pdf</n-button>
        </div>
      </div>
    </div>
  </n-card>
</template>
