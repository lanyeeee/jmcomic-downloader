<script setup lang="ts">
import { ref } from 'vue'
import { useStore } from '../../../store.ts'
import { NCheckbox, NInput, NInputGroup, NInputGroupLabel, NInputNumber, NSelect, NTooltip } from 'naive-ui'

const store = useStore()

const exportDirFmt = ref<string>(store.config?.exportDirFmt ?? '')
const mergePdfFmt = ref<string>(store.config?.mergePdfFmt ?? '')

const exportSkipModeOptions = [
  { label: '不跳过，每次都重新导出', value: 'None' },
  { label: '跳过已存在的文件', value: 'SkipExisting' },
  { label: '跳过曾导出过的章节', value: 'SkipExported' },
]
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col">
    <div class="flex gap-1 items-center">
      <n-input-group class="w-70">
        <n-input-group-label size="small">创建pdf并发数</n-input-group-label>
        <n-input-number
          class="w-full"
          v-model:value="store.config.createPdfConcurrency"
          size="small"
          :min="1"
          :parse="(x: string) => Number(x)" />
      </n-input-group>
      <n-tooltip placement="top" trigger="hover">
        <div>
          <span>在</span>
          <span class="rounded bg-gray-500 px-1">章节详情</span>
          <span>里手动勾选导出的PDF一律不会自动合并</span>
        </div>
        <div>
          <span>只有在</span>
          <span class="rounded bg-gray-500 px-1">本地库存</span>
          <span>里直接导出整部作品为PDF</span>
        </div>
        <div>
          <span>且导出策略不为</span>
          <span class="rounded bg-gray-500 px-1">跳过曾导出过的章节</span>
          <span>时才会触发自动合并</span>
        </div>
        <template #trigger>
          <n-checkbox class="ml-4 w-fit" v-model:checked="store.config.enableMergePdf">创建完成后自动合并</n-checkbox>
        </template>
      </n-tooltip>
    </div>

    <n-tooltip placement="top" trigger="hover">
      <div>
        <span>只影响</span>
        <span class="rounded bg-gray-500 px-1">本地库存</span>
        <span>里直接导出整部作品时的行为</span>
      </div>
      <div>
        <span>在</span>
        <span class="rounded bg-gray-500 px-1">章节详情</span>
        <span>里手动勾选导出时一律以</span>
        <span class="rounded bg-gray-500 px-1">不跳过，每次都重新导出</span>
        <span>处理</span>
      </div>
      <template #trigger>
        <n-input-group class="mt-2 w-fit">
          <n-input-group-label size="small">导出策略</n-input-group-label>
          <n-select
            v-model:value="store.config.exportSkipMode"
            :options="exportSkipModeOptions"
            size="small"
            class="w-50" />
        </n-input-group>
      </template>
    </n-tooltip>

    <span class="font-bold mt-2">导出目录格式</span>
    <n-tooltip placement="top" trigger="hover" :width="550">
      <div>
        可以用斜杠
        <span class="rounded bg-gray-500 px-1 text-white">/</span>
        来分隔目录层级，最后一层会作为导出文件名
      </div>
      <div class="text-orange">章节字段只能出现在最后一层，且最后一层至少要有一个章节字段</div>
      <div class="grid grid-cols-2">
        <div class="flex flex-col">
          <div class="font-semibold mt-2">通用字段：</div>
          <div>
            <span class="rounded bg-gray-500 px-1">comic_id</span>
            <span class="ml-2">漫画ID</span>
          </div>
          <div>
            <span class="rounded bg-gray-500 px-1">comic_title</span>
            <span class="ml-2">漫画标题</span>
          </div>
          <div>
            <span class="rounded bg-gray-500 px-1">author</span>
            <span class="ml-2">作者</span>
          </div>
          <div>
            <span class="rounded bg-gray-500 px-1">export_format</span>
            <span class="ml-2">导出格式(cbz/pdf)</span>
          </div>
        </div>
        <div class="flex flex-col">
          <div class="font-semibold mt-2">章节字段：</div>
          <div>
            <span class="rounded bg-gray-500 px-1">chapter_id</span>
            <span class="ml-2">章节ID</span>
          </div>
          <div>
            <span class="rounded bg-gray-500 px-1">chapter_title</span>
            <span class="ml-2">章节标题</span>
          </div>
          <div>
            <span class="rounded bg-gray-500 px-1">order</span>
            <span class="ml-2">章节在漫画里对应的序号</span>
          </div>
        </div>
      </div>
      <div class="font-semibold mt-2">例如格式</div>
      <div class="bg-gray-200 rounded-md p-1 text-black w-fit">
        {export_format}/{comic_title}/{order:0>3} {chapter_title}
      </div>
      <div class="font-semibold">
        <span>导出《蓦然回首》第1话为cbz时，会产生两层文件夹</span>
      </div>
      <div class="flex gap-1 text-black">
        <span class="bg-gray-200 rounded-md px-2 w-fit">cbz</span>
        <span class="rounded bg-gray-500 px-1 text-white">/</span>
        <span class="bg-gray-200 rounded-md px-2 w-fit">蓦然回首</span>
      </div>
      <div class="font-semibold">
        <span>并在里面生成</span>
        <span class="bg-gray-200 rounded-md px-2 ml-1 text-black font-normal">001 第1话.cbz</span>
        <span class="ml-1">({order:0>3} 表示用0把序号补齐到3位)</span>
      </div>
      <template #trigger>
        <n-input
          v-model:value="exportDirFmt"
          size="small"
          @blur="store.config.exportDirFmt = exportDirFmt"
          @keydown.enter="store.config.exportDirFmt = exportDirFmt" />
      </template>
    </n-tooltip>

    <span class="font-bold mt-2">合并pdf目录格式</span>
    <n-tooltip placement="top" trigger="hover" :width="550">
      <div>
        可以用斜杠
        <span class="rounded bg-gray-500 px-1 text-white">/</span>
        来分隔目录层级，最后一层会作为合并后的PDF文件名
      </div>
      <div class="font-semibold mt-2">可用字段：</div>
      <div>
        <span class="rounded bg-gray-500 px-1">comic_id</span>
        <span class="ml-2">漫画ID</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1">comic_title</span>
        <span class="ml-2">漫画标题</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1">author</span>
        <span class="ml-2">作者</span>
      </div>
      <div class="font-semibold mt-2">例如格式</div>
      <div class="bg-gray-200 rounded-md p-1 text-black w-fit">合并pdf/{comic_title}</div>
      <div class="font-semibold">
        <span>导出《蓦然回首》并触发自动合并时，会在目录</span>
        <span class="bg-gray-200 rounded-md px-2 mx-1 text-black font-normal">合并pdf</span>
        <span>下生成</span>
        <span class="bg-gray-200 rounded-md px-2 ml-1 text-black font-normal">蓦然回首.pdf</span>
      </div>
      <template #trigger>
        <n-input
          v-model:value="mergePdfFmt"
          size="small"
          @blur="store.config.mergePdfFmt = mergePdfFmt"
          @keydown.enter="store.config.mergePdfFmt = mergePdfFmt" />
      </template>
    </n-tooltip>
  </div>
</template>
