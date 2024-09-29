<script setup lang="ts">
import {SelectionArea, SelectionEvent, SelectionOptions} from "@viselect/vue";
import {nextTick, ref, watch} from "vue";
import {Album, commands} from "../bindings.ts";

const selectedAlbum = defineModel<Album | undefined>("selectedAlbum", {required: true});

const dropdownX = ref<number>(0);
const dropdownY = ref<number>(0);
const showDropdown = ref<boolean>(false);
const dropdownOptions = [
  {label: "勾选", key: "check"},
  {label: "取消勾选", key: "uncheck"},
  {label: "全选", key: "check all"},
  {label: "取消全选", key: "uncheck all"},
];
const checkedIds = ref<number[]>([]);
const selectedIds = ref<Set<number>>(new Set());
//记录这次框选是否改动了选中的元素
const selectedChanged = ref<boolean>(false);
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>();

watch(selectedAlbum, () => {
  checkedIds.value = [];
  selectedIds.value.clear();
  selectionAreaRef.value?.selection?.clearSelection();
});

watch(selectedIds.value, () => {
  selectedChanged.value = true;
});

function extractIds(elements: Element[]): number[] {
  return elements.map(element => element.getAttribute("data-key"))
      .filter(Boolean)
      .map(Number)
      .filter(id => {
        const chapterInfo = selectedAlbum.value?.chapterInfos.find(c => c.chapterId === id);
        if (chapterInfo === undefined) {
          return false;
        }
        return !chapterInfo.isDownloaded;
      });
}

function onMouseDown(event: MouseEvent) {
  if (event.ctrlKey || event.metaKey) {
    return;
  }
  if (event?.button === 0) {
    selectedChanged.value = false;
  }
}

function onMouseUp(event: MouseEvent) {
  // 如果是左键点击，且没有改动选中的元素，则清空选中
  if (event?.button === 0 && !selectedChanged.value) {
    selectedIds.value.clear();
    selectionAreaRef.value?.selection?.clearSelection();
  }
}

function onDragStart({event, selection}: SelectionEvent) {
  if (!event?.ctrlKey && !event?.metaKey) {
    selection.clearSelection();
    selectedIds.value.clear();
  }
}

function onDragMove({store: {changed: {added, removed}}}: SelectionEvent) {
  extractIds(added).forEach(id => selectedIds.value.add(id));
  extractIds(removed).forEach(id => selectedIds.value.delete(id));
}

function onDropdownSelect(key: "check" | "uncheck" | "check all" | "uncheck all") {
  showDropdown.value = false;
  if (key === "check") {
    // 只有未勾选的才会被勾选
    [...selectedIds.value]
        .filter(id => !checkedIds.value.includes(id))
        .forEach(id => checkedIds.value.push(id));
  } else if (key === "uncheck") {
    checkedIds.value = checkedIds.value.filter(id => !selectedIds.value.has(id));
  } else if (key === "check all") {
    // 只有未锁定的才会被勾选
    selectedAlbum.value?.chapterInfos
        ?.filter(c => !c.isDownloaded && !checkedIds.value.includes(c.chapterId))
        .forEach(c => checkedIds.value.push(c.chapterId));
  } else if (key === "uncheck all") {
    checkedIds.value.length = 0;
  }
}

async function onContextMenu(e: MouseEvent) {
  showDropdown.value = false;
  await nextTick();
  showDropdown.value = true;
  dropdownX.value = e.clientX;
  dropdownY.value = e.clientY;
}

async function downloadChapters() {
  const chapterToDownload = selectedAlbum.value?.chapterInfos.filter(c => !c.isDownloaded && checkedIds.value.includes(c.chapterId));
  if (chapterToDownload === undefined) {
    return;
  }
  await commands.downloadChapters(chapterToDownload);

  for (const downloadedChapter of chapterToDownload) {
    const chapter = selectedAlbum.value?.chapterInfos.find(c => c.chapterId === downloadedChapter.chapterId);
    if (chapter !== undefined) {
      chapter.isDownloaded = true;
      checkedIds.value = checkedIds.value.filter(id => id !== downloadedChapter.chapterId);
    }
  }
}

async function refreshChapters() {
}

</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex flex-justify-around">
      <span>总章数：{{ selectedAlbum?.chapterInfos.length }}</span>
      <n-divider vertical></n-divider>
      <span>已下载：{{ selectedAlbum?.chapterInfos.filter(c => c.isDownloaded).length }}</span>
      <n-divider vertical></n-divider>
      <span>已勾选：{{ checkedIds.length }}</span>
    </div>
    <div class="flex justify-between">
      左键拖动进行框选，右键打开菜单
      <n-button size="tiny" :disabled="selectedAlbum===undefined" @click="refreshChapters" class="w-1/6">刷新</n-button>
      <n-button size="tiny" :disabled="selectedAlbum===undefined" type="primary" @click="downloadChapters"
                class="w-1/4">
        下载勾选章节
      </n-button>
    </div>
    <n-empty v-if="selectedAlbum === undefined" description="请先进行漫画搜索">
    </n-empty>
    <SelectionArea v-else
                   ref="selectionAreaRef"
                   class="selection-container"
                   :options="{selectables: '.selectable'} as SelectionOptions"
                   @contextmenu="onContextMenu"
                   @mousedown="onMouseDown"
                   @mouseup="onMouseUp"
                   @move="onDragMove"
                   @start="onDragStart">
      <n-checkbox-group v-model:value="checkedIds" class="grid grid-cols-3 gap-1.5 w-full">
        <n-checkbox v-for="{chapterId, chapterTitle, isDownloaded} in selectedAlbum.chapterInfos"
                    :key="chapterId"
                    :data-key="chapterId"
                    class="selectable hover:bg-gray-200!"
                    :value="chapterId"
                    :label="chapterTitle"
                    :disabled="isDownloaded"
                    :class="{ selected: selectedIds.has(chapterId), downloaded: isDownloaded }"/>
      </n-checkbox-group>
    </SelectionArea>

    <n-dropdown
        placement="bottom-start"
        trigger="manual"
        :x="dropdownX"
        :y="dropdownY"
        :options="dropdownOptions"
        :show="showDropdown"
        :on-clickoutside="()=>showDropdown=false"
        @select="onDropdownSelect"
    />
  </div>
</template>

<style scoped>
.selection-container {
  @apply user-select-none overflow-auto;
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