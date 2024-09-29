<script setup lang="ts">

import {onMounted, ref} from "vue";
import {commands, Config, events} from "../bindings.ts";
import {open} from "@tauri-apps/plugin-dialog";
import {NProgress, useNotification} from "naive-ui";

type ProgressData = {
  title: string;
  downloadedCount: number;
  total: number;
  percentage: number;
  indicator: string;
}

const notification = useNotification();

const config = defineModel<Config>("config", {required: true});

const progresses = ref<Map<number, ProgressData>>(new Map());
const overallProgress = ref<ProgressData>({
  title: "总进度",
  downloadedCount: 0,
  total: 0,
  percentage: 0,
  indicator: ""
});

onMounted(async () => {
  await events.downloadChapterPendingEvent.listen(({payload}) => {
    let progressData: ProgressData = {
      title: `等待中 ${payload.title}`,
      downloadedCount: 0,
      total: 0,
      percentage: 0,
      indicator: ""
    };
    progresses.value.set(payload.chapterId, progressData);
  });

  await events.downloadChapterStartEvent.listen(({payload}) => {
    const progressData = progresses.value.get(payload.chapterId) as (ProgressData | undefined);
    if (progressData === undefined) {
      return;
    }
    progressData.total = payload.total;
    progressData.title = payload.title;
  });

  await events.downloadImageSuccessEvent.listen(({payload}) => {
    const progressData = progresses.value.get(payload.chapterId) as (ProgressData | undefined);
    if (progressData === undefined) {
      return;
    }
    progressData.downloadedCount = payload.downloadedCount;
    progressData.percentage = Math.round(progressData.downloadedCount / progressData.total * 100);
  });

  await events.downloadImageErrorEvent.listen(({payload}) => {
    const progressData = progresses.value.get(payload.chapterId) as (ProgressData | undefined);
    if (progressData === undefined) {
      return;
    }
    notification.warning({
      title: "下载图片失败",
      description: payload.url,
      content: payload.errMsg,
      meta: progressData.title
    });
  });

  await events.downloadChapterEndEvent.listen(({payload}) => {
    const progressData = progresses.value.get(payload.chapterId) as (ProgressData | undefined);
    if (progressData === undefined) {
      return;
    }
    if (payload.errMsg !== null) {
      notification.warning({title: "下载章节失败", content: payload.errMsg, meta: progressData.title});
    }
    progresses.value.delete(payload.chapterId);
  });

  await events.updateOverallDownloadProgressEvent.listen(({payload}) => {
    overallProgress.value.percentage = payload.percentage;
    overallProgress.value.downloadedCount = payload.downloadedImageCount;
    overallProgress.value.total = payload.totalImageCount;
    console.log(payload);
  });

  await events.downloadSpeedEvent.listen(({payload}) => {
    overallProgress.value.indicator = payload.speed;
  });
});

async function showDownloadDirInFileManager() {
  if (config.value === undefined) {
    return;
  }
  const result = await commands.showPathInFileManager(config.value.downloadDir);
  if (result.status === "error") {
    notification.error({title: "打开下载目录失败", description: result.error});
  }
}

async function selectDownloadDir() {
  const selectedDirPath = await open({directory: true});
  if (selectedDirPath === null) {
    return;
  }
  config.value.downloadDir = selectedDirPath;
}

</script>

<template>
  <div class="flex flex-col gap-row-1">
    <n-h3 class="m-be-0">下载列表</n-h3>
    <div class="flex">
      <n-input v-model:value="config.downloadDir"
               size="tiny"
               readonly
               placeholder="请选择漫画目录"
               @click="selectDownloadDir">
        <template #prefix>下载目录：</template>
      </n-input>
      <n-button size="tiny" @click="showDownloadDirInFileManager">打开下载目录</n-button>
    </div>
    <n-radio-group v-model:value="config.downloadFormat">
      下载格式：
      <n-tooltip placement="bottom" trigger="hover">
        <template #trigger>
          <n-radio value="Jpeg">jpg</n-radio>
        </template>
        1. 有损<span class="text-red">(肉眼看不出)</span><br/>
        2. 文件体积小<br/>
        3. 编码速度最快<br/>
      </n-tooltip>
      <n-tooltip placement="bottom" trigger="hover">
        <template #trigger>
          <n-radio value="Png">png</n-radio>
        </template>
        1. 无损<br/>
        2. 文件体积大<span class="text-red">(约为jpg的5倍)</span><br/>
        3. 编码速度最慢<br/>
      </n-tooltip>
      <n-tooltip placement="bottom" trigger="hover">
        <template #trigger>
          <n-radio value="Webp">webp</n-radio>
        </template>
        1. 无损<br/>
        2. 文件体积大<span class="text-red">(约为jpg的4倍)</span><br/>
        3. 编码速度较慢<br/>
      </n-tooltip>
    </n-radio-group>
    <div class="grid grid-cols-[1fr_4fr_2fr]">
      <span class="text-ellipsis whitespace-nowrap overflow-hidden">{{ overallProgress.title }}</span>
      <n-progress :percentage="overallProgress.percentage" indicator-placement="inside" :height="21">
        {{ overallProgress.indicator }}
      </n-progress>
      <span>{{ overallProgress.downloadedCount }}/{{ overallProgress.total }}</span>
    </div>
    <div class="grid grid-cols-[1fr_4fr]"
         v-for="[chapterId, {title, percentage, downloadedCount, total}] in progresses"
         :key="chapterId">
      <span class="mb-1! text-ellipsis whitespace-nowrap overflow-hidden">{{ title }}</span>
      <n-progress class="" :percentage="percentage">
        {{ downloadedCount }}/{{ total }}
      </n-progress>
    </div>
  </div>
</template>