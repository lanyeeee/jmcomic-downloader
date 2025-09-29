<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue'
import { GetWeeklyInfoRespData, commands } from '../bindings.ts'
import { SelectProps } from 'naive-ui'
import ComicCard from '../components/ComicCard.vue'
import { useStore } from '../store.ts'

const store = useStore()

const weeklyInfo = ref<GetWeeklyInfoRespData>()

const selectedCategoryId = ref<string>('')
const currentWeeklyTypeId = ref<string>('')

const categoryOptions = computed<SelectProps['options']>(() =>
  weeklyInfo.value?.categories.map((category) => ({
    label: category.time,
    value: category.id,
  })),
)

onMounted(async () => {
  const result = await commands.getWeeklyInfo()
  if (result.status === 'error') {
    console.error(result.error)
    return
  }

  weeklyInfo.value = result.data

  selectedCategoryId.value = result.data.categories[0].id
  currentWeeklyTypeId.value = result.data.type[result.data.type.length - 1].id
})

watch(
  () => [selectedCategoryId.value, currentWeeklyTypeId.value],
  () => {
    store.getWeeklyResult = undefined
    getWeekly()
  },
)

async function getWeekly() {
  const result = await commands.getWeekly(selectedCategoryId.value, currentWeeklyTypeId.value)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }

  store.getWeeklyResult = result.data
}
</script>

<template>
  <div v-if="weeklyInfo !== undefined" class="h-full flex flex-col">
    <n-select
      v-if="categoryOptions !== undefined"
      class="flex box-border px-2 pt-2"
      v-model:value="selectedCategoryId"
      :options="categoryOptions"
      :show-checkmark="false"
      size="small" />
    <n-tabs class="h-full overflow-auto mb-2" v-model:value="currentWeeklyTypeId" type="line" size="small">
      <n-tab-pane
        class="h-full overflow-auto"
        v-for="weeklyType in weeklyInfo.type"
        :key="weeklyType.id"
        :name="weeklyType.id"
        :tab="weeklyType.title">
        <div v-if="store.getWeeklyResult !== undefined" class="flex flex-col gap-row-2 overflow-auto box-border px-2">
          <ComicCard
            v-for="comicInWeekly in store.getWeeklyResult.list"
            :key="comicInWeekly.id"
            :comic-id="comicInWeekly.id"
            :comic-title="comicInWeekly.name"
            :comic-author="comicInWeekly.author"
            :comic-category="comicInWeekly.category"
            :comic-category-sub="comicInWeekly.category_sub"
            :comic-downloaded="comicInWeekly.is_downloaded"
            :comic-download-dir="comicInWeekly.comic_download_dir" />
        </div>
      </n-tab-pane>
    </n-tabs>
  </div>
</template>
