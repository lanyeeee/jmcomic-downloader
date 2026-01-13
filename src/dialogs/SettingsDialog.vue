<script setup lang="ts">
import { commands } from '../bindings.ts'
import { ref, watch } from 'vue'
import { path } from '@tauri-apps/api'
import { appDataDir } from '@tauri-apps/api/path'
import { useStore } from '../store.ts'
import { useMessage } from 'naive-ui'

const store = useStore()

const message = useMessage()

const showing = defineModel<boolean>('showing', { required: true })

const dirFmt = ref<string>(store.config?.dirFmt ?? '')
const proxyHost = ref<string>(store.config?.proxyHost ?? '')
const customApiDomain = ref<string>(store.config?.customApiDomain ?? '')

watch([() => store.config?.apiDomainMode, () => store.config?.customApiDomain], () => {
  message.warning('切换线路后可能需要重新登录')
})

async function showConfigInFileManager() {
  const configName = 'config.json'
  const configPath = await path.join(await appDataDir(), configName)
  const result = await commands.showPathInFileManager(configPath)
  if (result.status === 'error') {
    console.error(result.error)
  }
}
</script>

<template>
  <n-modal v-if="store.config !== undefined" v-model:show="showing">
    <n-dialog class="w-140!" :showIcon="false" title="配置" @close="showing = false">
      <div class="flex flex-col">
        <span class="font-bold">下载速度</span>
        <div class="flex flex-col gap-1">
          <div class="flex gap-1">
            <n-input-group class="w-35%">
              <n-input-group-label size="small">章节并发数</n-input-group-label>
              <n-input-number
                class="w-full"
                v-model:value="store.config.chapterConcurrency"
                size="small"
                @update:value="message.warning('对章节并发数的修改需要重启才能生效')"
                :min="1"
                :parse="(x: string) => Number(x)" />
            </n-input-group>
            <n-input-group class="w-65%">
              <n-input-group-label size="small">每个章节下载完成后休息</n-input-group-label>
              <n-input-number
                class="w-full"
                v-model:value="store.config.chapterDownloadIntervalSec"
                size="small"
                :min="0"
                :parse="(x: string) => Number(x)" />
              <n-input-group-label size="small">秒</n-input-group-label>
            </n-input-group>
          </div>
          <div class="flex gap-1">
            <n-input-group class="w-35%">
              <n-input-group-label size="small">图片并发数</n-input-group-label>
              <n-input-number
                class="w-full"
                v-model:value="store.config.imgConcurrency"
                size="small"
                @update-value="message.warning('对图片并发数的修改需要重启才能生效')"
                :min="1"
                :parse="(x: string) => Number(x)" />
            </n-input-group>
            <n-input-group class="w-65%">
              <n-input-group-label size="small">每张图片下载完成后休息</n-input-group-label>
              <n-input-number
                class="w-full"
                v-model:value="store.config.imgDownloadIntervalSec"
                size="small"
                :min="0"
                :parse="(x: string) => Number(x)" />
              <n-input-group-label size="small">秒</n-input-group-label>
            </n-input-group>
          </div>
          <n-input-group>
            <n-input-group-label size="small">下载整个收藏夹时，每处理完一个收藏夹中的漫画后休息</n-input-group-label>
            <n-input-number
              class="w-full"
              v-model:value="store.config.downloadAllFavoritesIntervalSec"
              size="small"
              :min="0"
              :parse="(x: string) => Number(x)" />
            <n-input-group-label size="small">秒</n-input-group-label>
          </n-input-group>
          <n-input-group>
            <n-input-group-label size="small">更新库存时，每处理完一个已下载的漫画后休息</n-input-group-label>
            <n-input-number
              class="w-full"
              v-model:value="store.config.updateDownloadedComicsIntervalSec"
              size="small"
              :min="0"
              :parse="(x: string) => Number(x)" />
            <n-input-group-label size="small">秒</n-input-group-label>
          </n-input-group>
        </div>

        <span class="font-bold mt-2">下载格式</span>
        <n-radio-group v-model:value="store.config.downloadFormat">
          <n-tooltip placement="top" trigger="hover">
            <template #trigger>
              <n-radio value="Jpeg">jpg</n-radio>
            </template>
            1. 有损
            <span class="text-red">(肉眼看不出)</span>
            <br />
            2. 文件体积小
            <br />
            4. 宽高的上限为65534
            <span class="text-red">(某些条漫可能会超过这个上限导致报错)</span>
            <br />
            3. 编码速度最快
            <br />
          </n-tooltip>
          <n-tooltip placement="top" trigger="hover">
            <template #trigger>
              <n-radio value="Png">png</n-radio>
            </template>
            1. 无损
            <br />
            2. 文件体积大
            <span class="text-red">(约为jpg的5倍)</span>
            <br />
            3. 编码速度最慢
            <br />
          </n-tooltip>
          <n-tooltip placement="top" trigger="hover">
            <template #trigger>
              <n-radio value="Webp">webp</n-radio>
            </template>
            1. 无损
            <br />
            2. 文件体积大
            <span class="text-red">(约为jpg的4倍)</span>
            <br />
            3. 宽高的上限为16383
            <span class="text-red">(某些条漫可能会超过这个上限导致报错)</span>
            <br />
            4. 编码速度较慢
            <br />
          </n-tooltip>
        </n-radio-group>

        <span class="font-bold mt-2">API域名</span>
        <n-radio-group v-model:value="store.config.apiDomainMode" size="small">
          <n-radio-button value="Domain1">线路1</n-radio-button>
          <n-radio-button value="Domain2">线路2</n-radio-button>
          <n-radio-button value="Domain3">线路3</n-radio-button>
          <n-radio-button value="Domain4">线路4</n-radio-button>
          <n-radio-button value="Domain5">线路5</n-radio-button>
          <n-radio-button value="Custom">自定义</n-radio-button>
        </n-radio-group>
        <n-input-group v-if="store.config.apiDomainMode === 'Custom'" class="mt-1">
          <n-input-group-label size="small">自定义API域名</n-input-group-label>
          <n-input
            v-model:value="customApiDomain"
            size="small"
            placeholder=""
            @blur="store.config.customApiDomain = customApiDomain"
            @keydown.enter="store.config.customApiDomain = customApiDomain" />
        </n-input-group>

        <span class="font-bold mt-2">代理类型</span>
        <n-radio-group v-model:value="store.config.proxyMode" size="small">
          <n-radio-button value="System">系统代理</n-radio-button>
          <n-radio-button value="NoProxy">直连</n-radio-button>
          <n-radio-button value="Custom">自定义</n-radio-button>
        </n-radio-group>
        <n-input-group v-if="store.config.proxyMode === 'Custom'" class="mt-1">
          <n-input-group-label size="small">http://</n-input-group-label>
          <n-input
            v-model:value="proxyHost"
            size="small"
            placeholder=""
            @blur="store.config.proxyHost = proxyHost"
            @keydown.enter="store.config.proxyHost = proxyHost" />
          <n-input-group-label size="small">:</n-input-group-label>
          <n-input-number
            v-model:value="store.config.proxyPort"
            size="small"
            placeholder=""
            :parse="(x: string) => parseInt(x)" />
        </n-input-group>

        <span class="font-bold mt-2">下载目录格式</span>
        <n-tooltip placement="top" trigger="hover" width="550">
          <div>
            可以用斜杠
            <span class="rounded bg-gray-500 px-1 text-white">/</span>
            来分隔目录层级
          </div>
          <div class="text-orange">至少要有两个层级，最后一层存放章节元数据，倒数第二层存放漫画元数据</div>
          <div class="font-semibold mt-2">可用字段：</div>
          <div class="grid grid-cols-2">
            <div>
              <span class="rounded bg-gray-500 px-1">comic_id</span>
              <span class="ml-2">漫画ID</span>
            </div>
            <div>
              <span class="rounded bg-gray-500 px-1">chapter_id</span>
              <span class="ml-2">章节ID</span>
            </div>
            <div>
              <span class="rounded bg-gray-500 px-1">comic_title</span>
              <span class="ml-2">漫画标题</span>
            </div>
            <div>
              <span class="rounded bg-gray-500 px-1">chapter_title</span>
              <span class="ml-2">章节标题</span>
            </div>
            <div>
              <span class="rounded bg-gray-500 px-1">author</span>
              <span class="ml-2">作者</span>
            </div>
            <div>
              <span class="rounded bg-gray-500 px-1">order</span>
              <span class="ml-2">章节在漫画里对应的序号</span>
            </div>
          </div>
          <div class="font-semibold mt-2">例如格式</div>
          <div class="bg-gray-200 rounded-md p-1 text-black w-fit">
            {author}/[{author}] {comic_title}({comic_id})/{order} - {chapter_title}
          </div>
          <div class="font-semibold">下载《蓦然回首》第1话会产生三层文件夹，分别是</div>
          <div class="flex gap-1 text-black">
            <span class="bg-gray-200 rounded-md px-2 w-fit">藤本树, 藤本タツキ</span>
            <span class="rounded bg-gray-500 px-1 text-white">/</span>
            <span class="bg-gray-200 rounded-md px-2 w-fit">[藤本树, 藤本タツキ] 蓦然回首(384524)</span>
            <span class="rounded bg-gray-500 px-1 text-white">/</span>
            <span class="bg-gray-200 rounded-md px-2 w-fit">1 - 第1话</span>
          </div>
          <template #trigger>
            <n-input
              v-model:value="dirFmt"
              size="small"
              @blur="store.config.dirFmt = dirFmt"
              @keydown.enter="store.config.dirFmt = dirFmt" />
          </template>
        </n-tooltip>

        <span class="font-bold mt-2">其他</span>
        <n-checkbox class="w-fit" v-model:checked="store.config.shouldDownloadCover">下载封面</n-checkbox>

        <n-button class="ml-auto mt-4" size="small" @click="showConfigInFileManager">打开配置目录</n-button>
      </div>
    </n-dialog>
  </n-modal>
</template>
