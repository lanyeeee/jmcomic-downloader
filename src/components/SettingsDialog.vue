<script setup lang="ts">
import {Config} from "../bindings.ts";
import {computed, ref} from "vue";

const config = defineModel<Config>("config", {required: true});
const showing = defineModel<boolean>("showing", {required: true});

const proxyHost = ref<string>(config.value.proxyHost);

const disableProxyHostAndPort = computed(() => config.value.proxyMode !== "Custom");

</script>

<template>
  <n-dialog :showIcon="false"
            title="设置"
            content-style=""
            @close="showing=false">
    <div class="flex flex-col gap-row-2">
      <n-radio-group v-model:value="config.downloadFormat">
        图片下载格式：
        <n-tooltip placement="top" trigger="hover">
          <template #trigger>
            <n-radio value="Jpeg">jpg</n-radio>
          </template>
          1. 有损<span class="text-red">(肉眼看不出)</span><br/>
          2. 文件体积小<br/>
          3. 编码速度最快<br/>
        </n-tooltip>
        <n-tooltip placement="top" trigger="hover">
          <template #trigger>
            <n-radio value="Png">png</n-radio>
          </template>
          1. 无损<br/>
          2. 文件体积大<span class="text-red">(约为jpg的5倍)</span><br/>
          3. 编码速度最慢<br/>
        </n-tooltip>
        <n-tooltip placement="top" trigger="hover">
          <template #trigger>
            <n-radio value="Webp">webp</n-radio>
          </template>
          1. 无损<br/>
          <span class="text-red">2. 这是jm图片原本的格式</span><br/>
          3. 文件体积大<span class="text-red">(约为jpg的4倍)</span><br/>
          4. 编码速度较慢<br/>
        </n-tooltip>
      </n-radio-group>
      <n-radio-group v-model:value="config.archiveFormat">
        章节保存格式：
        <n-radio value="Image">文件夹-图片</n-radio>
        <n-radio value="Pdf">pdf</n-radio>
      </n-radio-group>
      <n-radio-group v-model:value="config.proxyMode">
        代理类型：
        <n-radio value="System">系统代理</n-radio>
        <n-radio value="NoProxy">直连</n-radio>
        <n-radio value="Custom">自定义</n-radio>
      </n-radio-group>
      <div class="flex">
        <n-input :disabled=disableProxyHostAndPort
                 v-model:value="proxyHost"
                 size="tiny"
                 placeholder=""
                 @blur="config.proxyHost=proxyHost"
                 @keydown.enter="config.proxyHost=proxyHost">
          <template #prefix>
            主机:
          </template>
        </n-input>
        <n-input-number :disabled=disableProxyHostAndPort
                        v-model:value="config.proxyPort"
                        size="tiny"
                        placeholder=""
                        :parse="(x:string) => parseInt(x)">
          <template #prefix>
            端口:
          </template>
        </n-input-number>
      </div>
    </div>
  </n-dialog>

</template>
