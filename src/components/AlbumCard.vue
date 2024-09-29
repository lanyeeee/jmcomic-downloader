<script setup lang="ts">
import {Album, AlbumInSearchRespData, commands} from "../bindings.ts";
import {useNotification} from "naive-ui";

defineProps<{
  albumInSearch: AlbumInSearchRespData
}>();

const selectedAlbum = defineModel<Album | undefined>("selectedAlbum", {required: true});
const currentTabName = defineModel<"search" | "chapter">("currentTabName", {required: true});

const notification = useNotification();

async function onClickItem(aid: number) {
  const result = await commands.getAlbum(aid);
  if (result.status === "error") {
    notification.error({title: "获取漫画失败", description: result.error});
    return;
  }
  selectedAlbum.value = result.data;
  currentTabName.value = "chapter";
}

</script>

<template>
  <n-card class="cursor-pointer"
          content-style="padding: 0.25rem;"
          hoverable
          @click="onClickItem(parseInt(albumInSearch.id))">
    <div class="flex">
      <img class="w-24 object-cover pr-4"
           :src="`https://cdn-msp3.18comic.vip/media/albums/${albumInSearch.id}_3x4.jpg`"
           alt=""
           referrerpolicy="no-referrer"/>
      <div class="flex flex-col h-full">
        <!--   TODO: 解决漫画名字太长导致的排版问题     -->
        <span class="font-bold text-xl">{{ albumInSearch.name }}</span>
        <span class="text-red">作者：{{ albumInSearch.author }}</span>
        <span class="text-gray">分类：{{ albumInSearch.category.title }} {{ albumInSearch.category_sub.title }}</span>
      </div>
    </div>
  </n-card>
</template>