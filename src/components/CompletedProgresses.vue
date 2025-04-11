<script setup lang="ts">
import { ProgressData } from '../types.ts'
import { computed } from 'vue'
import { useStore } from '../store.ts'

const store = useStore()

const completedProgresses = computed<[number, ProgressData][]>(() =>
  Array.from(store.progresses.entries())
    .filter(([, { state }]) => state === 'Completed')
    .sort((a, b) => {
      return b[1].totalImgCount - a[1].totalImgCount
    }),
)
</script>

<template>
  <div class="h-full flex flex-col gap-row-2 px-2 overflow-auto">
    <div class="grid grid-cols-[1fr_1fr] py-2 px-4 bg-gray-100 rounded-lg" v-for="[chapterId, { chapterInfo }] in completedProgresses" :key="chapterId">
      <span class="text-ellipsis whitespace-nowrap overflow-hidden" :title="chapterInfo.comicTitle">
        {{ chapterInfo.comicTitle }}
      </span>
      <span class="text-ellipsis whitespace-nowrap overflow-hidden" :title="chapterInfo.chapterTitle">
        {{ chapterInfo.chapterTitle }}
      </span>
    </div>
  </div>
</template>
