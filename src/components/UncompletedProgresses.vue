<script setup lang="tsx">
import { ProgressData } from '../types.ts'
import { ref, watchEffect, computed, nextTick } from 'vue'
import { SelectionArea, SelectionEvent } from '@viselect/vue'
import { commands, DownloadTaskState } from '../bindings.ts'
import { DropdownOption, NIcon, ProgressProps } from 'naive-ui'
import { useStore } from '../store.ts'
import {
  CheckOutlined,
  DeleteOutlined,
  PauseOutlined,
  RightOutlined,
  LoadingOutlined,
  ClockCircleOutlined,
  ExclamationCircleOutlined,
} from '@vicons/antd'

const store = useStore()

const selectedIds = ref<Set<number>>(new Set())
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()
const selectableRefs = ref<HTMLDivElement[]>([])
const { dropdownX, dropdownY, dropdownShowing, dropdownOptions, showDropdown } = useDropdown()

const uncompletedProgresses = computed<[number, ProgressData][]>(() =>
  Array.from(store.progresses.entries())
    .filter(([, { state }]) => state !== 'Completed' && state !== 'Cancelled')
    .sort((a, b) => b[1].totalImgCount - a[1].totalImgCount),
)

watchEffect(() => {
  // 只保留未完成的章节id
  const uncompletedIds = new Set(uncompletedProgresses.value.map(([chapterId]) => chapterId))
  selectedIds.value = new Set([...selectedIds.value].filter((chapterId) => uncompletedIds.has(chapterId)))
})

function extractIds(elements: Element[]): number[] {
  return elements
    .map((element) => element.getAttribute('data-key'))
    .filter(Boolean)
    .map(Number)
}

function updateSelectedIds({
  store: {
    changed: { added, removed },
  },
}: SelectionEvent) {
  extractIds(added).forEach((chapterId) => selectedIds.value.add(chapterId))
  extractIds(removed).forEach((chapterId) => selectedIds.value.delete(chapterId))
}

function unselectAll({ event, selection }: SelectionEvent) {
  if (!event?.ctrlKey && !event?.metaKey) {
    selection.clearSelection()
    selectedIds.value.clear()
  }
}

async function handleProgressDoubleClick(state: DownloadTaskState, chapterId: number) {
  if (state === 'Downloading' || state === 'Pending') {
    const result = await commands.pauseDownloadTask(chapterId)
    if (result.status === 'error') {
      console.error(result.error)
    }
  } else if (state === 'Paused') {
    const result = await commands.resumeDownloadTask(chapterId)
    if (result.status === 'error') {
      console.error(result.error)
    }
  } else {
    const progressData = store.progresses.get(chapterId)
    if (progressData === undefined) {
      return
    }
    const { comic } = progressData
    const result = await commands.createDownloadTask(comic, chapterId)
    if (result.status === 'error') {
      console.error(result.error)
    }
  }
}

function handleProgressContextMenu(chapterId: number) {
  if (selectedIds.value.has(chapterId)) {
    return
  }
  selectedIds.value.clear()
  selectedIds.value.add(chapterId)
}

function useDropdown() {
  const dropdownX = ref<number>(0)
  const dropdownY = ref<number>(0)
  const dropdownShowing = ref<boolean>(false)
  const dropdownOptions: DropdownOption[] = [
    {
      label: '全选',
      key: 'check all',
      icon: () => (
        <NIcon>
          <CheckOutlined />
        </NIcon>
      ),
      props: {
        onClick: () => {
          if (selectionAreaRef.value === undefined) {
            return
          }
          const selection = selectionAreaRef.value.selection
          if (selection === undefined) {
            return
          }
          selection.select(selectableRefs.value)
          dropdownShowing.value = false
        },
      },
    },
    {
      label: '继续',
      key: 'resume',
      icon: () => (
        <NIcon>
          <RightOutlined />
        </NIcon>
      ),
      props: {
        onClick: () => {
          selectedIds.value.forEach(async (chapterId) => {
            const progressData = store.progresses.get(chapterId)
            if (progressData === undefined) {
              return
            }
            const { state, comic } = progressData
            if (state === 'Cancelled' || state === 'Completed' || state === 'Failed') {
              const result = await commands.createDownloadTask(comic, chapterId)
              if (result.status === 'error') {
                console.error(result.error)
              }
              return
            }

            const result = await commands.resumeDownloadTask(chapterId)
            if (result.status === 'error') {
              console.error(result.error)
            }
          })
          dropdownShowing.value = false
        },
      },
    },
    {
      label: '暂停',
      key: 'pause',
      icon: () => (
        <NIcon>
          <PauseOutlined />
        </NIcon>
      ),
      props: {
        onClick: () => {
          selectedIds.value.forEach(async (chapterId) => {
            const progressData = store.progresses.get(chapterId)
            if (progressData === undefined) {
              return
            }
            const { state } = progressData
            if (state === 'Cancelled' || state === 'Completed' || state === 'Failed') {
              return
            }

            const result = await commands.pauseDownloadTask(chapterId)
            if (result.status === 'error') {
              console.error(result.error)
            }
          })
          dropdownShowing.value = false
        },
      },
    },
    {
      label: '取消',
      key: 'cancel',
      icon: () => (
        <NIcon>
          <DeleteOutlined />
        </NIcon>
      ),
      props: {
        onClick: () => {
          selectedIds.value.forEach(async (chapterId) => {
            const progressData = store.progresses.get(chapterId)
            if (progressData === undefined) {
              return
            }
            const { state } = progressData
            if (state === 'Cancelled' || state === 'Completed' || state === 'Failed') {
              return
            }

            const result = await commands.cancelDownloadTask(chapterId)
            if (result.status === 'error') {
              console.error(result.error)
            }
          })
          dropdownShowing.value = false
        },
      },
    },
  ]

  async function showDropdown(e: MouseEvent) {
    dropdownShowing.value = false
    await nextTick()
    dropdownShowing.value = true
    dropdownX.value = e.clientX
    dropdownY.value = e.clientY
  }

  return {
    dropdownX,
    dropdownY,
    dropdownShowing,
    dropdownOptions,
    showDropdown,
  }
}

function stateToStatus(state: DownloadTaskState): ProgressProps['status'] {
  if (state === 'Completed') {
    return 'success'
  } else if (state === 'Paused') {
    return 'warning'
  } else if (state === 'Failed') {
    return 'error'
  } else {
    return 'default'
  }
}

function stateToColorClass(state: DownloadTaskState) {
  if (state === 'Downloading') {
    return 'text-blue-500'
  } else if (state === 'Pending') {
    return 'text-gray-500'
  } else if (state === 'Paused') {
    return 'text-yellow-500'
  } else if (state === 'Failed') {
    return 'text-red-500'
  } else if (state === 'Completed') {
    return 'text-green-500'
  } else if (state === 'Cancelled') {
    return 'text-stone-500'
  }

  return ''
}
</script>

<template>
  <SelectionArea
    ref="selectionAreaRef"
    class="h-full flex flex-col selection-container px-2"
    :options="{ selectables: '.selectable', features: { deselectOnBlur: true } }"
    @contextmenu="showDropdown"
    @move="updateSelectedIds"
    @start="unselectAll">
    <span class="ml-auto select-none">左键拖动进行框选，右键打开菜单，双击暂停/继续</span>
    <div class="h-full select-none">
      <div
        v-for="[chapterId, { state, comic, chapterInfo, percentage, indicator }] in uncompletedProgresses"
        :key="chapterId"
        ref="selectableRefs"
        :data-key="chapterId"
        :class="[
          'selectable p-3 mb-2 rounded-lg',
          selectedIds.has(chapterId) ? 'selected shadow-md' : 'hover:bg-gray-1',
        ]"
        @dblclick="() => handleProgressDoubleClick(state, chapterId)"
        @contextmenu="() => handleProgressContextMenu(chapterId)">
        <div class="grid grid-cols-[1fr_1fr]">
          <div class="text-ellipsis whitespace-nowrap overflow-hidden" :title="comic.name">
            {{ comic.name }}
          </div>
          <div class="text-ellipsis whitespace-nowrap overflow-hidden" :title="chapterInfo.chapterTitle">
            {{ chapterInfo.chapterTitle }}
          </div>
        </div>
        <div class="flex">
          <n-icon :class="[stateToColorClass(state), 'mr-2']" :size="20">
            <LoadingOutlined v-if="state === 'Downloading'" class="animate-spin" />
            <ClockCircleOutlined v-else-if="state === 'Pending'" />
            <PauseOutlined v-else-if="state === 'Paused'" />
            <ExclamationCircleOutlined v-else-if="state === 'Failed'" />
          </n-icon>
          <div v-if="isNaN(percentage)" class="ml-auto">{{ indicator }}</div>
          <n-progress
            v-else
            :class="stateToColorClass(state)"
            :status="stateToStatus(state)"
            :percentage="percentage"
            :processing="state === 'Downloading'">
            {{ indicator }}
          </n-progress>
        </div>
      </div>
    </div>
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="dropdownX"
      :y="dropdownY"
      :options="dropdownOptions"
      :show="dropdownShowing"
      :on-clickoutside="() => (dropdownShowing = false)" />
  </SelectionArea>
</template>

<style scoped>
.selection-container {
  @apply select-none overflow-auto;
}

.selection-container .selected {
  @apply bg-[rgb(204,232,255)];
}

:global(.selection-area) {
  @apply bg-[rgba(46,115,252,0.5)];
}
</style>
