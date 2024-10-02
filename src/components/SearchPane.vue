<script setup lang="ts">
import {computed, ref} from "vue";
import {Album, commands, SearchRespData, SearchSort} from "../bindings.ts";
import {useMessage, useNotification} from "naive-ui";
import AlbumCard from "./AlbumCard.vue";

const message = useMessage();
const notification = useNotification();

const sortOptions = [
  {label: "最新", value: "Latest"},
  {label: "最多点击", value: "View"},
  {label: "最多图片", value: "Picture"},
  {label: "最多爱心", value: "Like"},
];

const selectedAlbum = defineModel<Album | undefined>("selectedAlbum", {required: true});
const currentTabName = defineModel<"search" | "favorite" | "chapter">("currentTabName", {required: true});

const searchInput = ref<string>("");
const sortSelected = ref<SearchSort>("Latest");
const searchPage = ref<number>(1);
const searchRespData = ref<SearchRespData>();

const searchPageCount = computed(() => {
  if (searchRespData.value === undefined) {
    return 0;
  }
  const total = searchRespData.value.total;
  return Math.floor(total / 80) + 1;
});

async function search(keyword: string, page: number, sort: SearchSort) {
  console.log(keyword, page, sort);
  searchPage.value = page;
  const result = await commands.search(keyword, page, sort);
  if (result.status === "error") {
    notification.error({title: "搜索失败", description: result.error});
    return;
  }
  const searchResult = result.data;
  if ("SearchRespData" in searchResult) {
    const respData = searchResult.SearchRespData;
    if (respData.content.length === 0) {
      message.warning("什么都没有搜到，请尝试其他关键词");
      return;
    }
    searchRespData.value = respData;
    console.log(respData);
  } else if ("Album" in searchResult) {
    const album = searchResult.Album;
    selectedAlbum.value = album;
    console.log(album);
    currentTabName.value = "chapter";
  }
}
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex flex-col">
      <div class="grid grid-cols-[4fr_1fr]">
        <div class="flex">
          <n-input class="text-align-left"
                   size="tiny"
                   v-model:value="searchInput"
                   placeholder="jm号也可以"
                   clearable
                   @keydown.enter="search(searchInput.trim(), 1, sortSelected)">
            <template #prefix>
              关键词:
            </template>
          </n-input>
          <n-button size="tiny" @click="search(searchInput.trim(), 1, sortSelected)">搜索</n-button>
        </div>
        <n-select class="flex"
                  v-model:value="sortSelected"
                  :options="sortOptions"
                  :show-checkmark="false"
                  size="tiny"
                  @update-value="search(searchInput.trim(), 1, $event)"/>
      </div>
    </div>

    <div v-if="searchRespData!==undefined" class="flex flex-col gap-row-1 overflow-auto p-2">
      <div class="flex flex-col gap-row-2 overflow-auto">
        <album-card v-for="albumInSearch in searchRespData.content"
                    :key="albumInSearch.id"
                    :album-info="albumInSearch"
                    v-model:selected-album="selectedAlbum"
                    v-model:current-tab-name="currentTabName"/>
      </div>
      <n-pagination :page-count="searchPageCount"
                    :page="searchPage"
                    @update:page="search(searchInput.trim(), $event, sortSelected)"/>
    </div>
  </div>
</template>