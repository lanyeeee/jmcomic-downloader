<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { Comic, commands, Config, GetUserProfileRespData } from './bindings.ts'
import { useMessage, useNotification } from 'naive-ui'
import LoginDialog from './components/LoginDialog.vue'
import SearchPane from './panes/SearchPane.vue'
import ChapterPane from './panes/ChapterPane.vue'
import DownloadingPane from './panes/DownloadingPane.vue'
import FavoritePane from './panes/FavoritePane.vue'
import AboutDialog from './components/AboutDialog.vue'
import { QuestionCircleOutlined, UserOutlined, SettingOutlined } from '@vicons/antd'
import SettingsDialog from './components/SettingsDialog.vue'
import { CurrentTabName } from './types.ts'
import DownloadedPane from './panes/DownloadedPane.vue'

const message = useMessage()
const notification = useNotification()

const config = ref<Config>()
const userProfile = ref<GetUserProfileRespData>()
const loginDialogShowing = ref<boolean>(false)
const settingsDialogShowing = ref<boolean>(false)
const aboutDialogShowing = ref<boolean>(false)
const currentTabName = ref<CurrentTabName>('search')
const pickedComic = ref<Comic>()

watch(
  config,
  async () => {
    if (config.value === undefined) {
      return
    }

    const result = await commands.saveConfig(config.value)
    if (result.status === 'error') {
      notification.error({ title: '保存配置失败', description: result.error })
      return
    }
    message.success('保存配置成功')
  },
  { deep: true },
)

onMounted(async () => {
  // 屏蔽浏览器右键菜单
  document.oncontextmenu = (event) => {
    event.preventDefault()
  }
  // 获取配置
  config.value = await commands.getConfig()
  // 如果username和password不为空，尝试登录
  if (config.value.username !== '' && config.value.password !== '') {
    const result = await commands.login(config.value.username, config.value.password)
    if (result.status === 'error') {
      notification.error({ title: '自动登录失败', description: result.error })
      return
    }
    userProfile.value = result.data
    message.success('自动登录成功')
  }
})
</script>

<template>
  <div v-if="config !== undefined" class="h-screen flex overflow-hidden">
    <n-tabs class="h-full w-1/2" v-model:value="currentTabName" type="line" size="small" animated>
      <n-tab-pane class="h-full overflow-auto p-0!" name="search" tab="漫画搜索" display-directive="show">
        <search-pane v-model:picked-comic="pickedComic" v-model:current-tab-name="currentTabName" />
      </n-tab-pane>
      <n-tab-pane class="h-full overflow-auto p-0!" name="favorite" tab="漫画收藏" display-directive="show">
        <favorite-pane
          :user-profile="userProfile"
          v-model:picked-comic="pickedComic"
          v-model:current-tab-name="currentTabName" />
      </n-tab-pane>
      <n-tab-pane class="h-full overflow-auto p-0!" name="downloaded" tab="本地库存" display-directive="show">
        <downloaded-pane v-model:picked-comic="pickedComic" v-model:current-tab-name="currentTabName" />
      </n-tab-pane>
      <n-tab-pane class="h-full overflow-auto p-0!" name="chapter" tab="章节详情" display-directive="show">
        <chapter-pane v-model:picked-comic="pickedComic" />
      </n-tab-pane>
    </n-tabs>
    <div class="w-1/2 overflow-auto flex flex-col">
      <div class="flex px-2 gap-1">
        <n-button type="primary" @click="loginDialogShowing = true">
          <template #icon>
            <n-icon>
              <UserOutlined />
            </n-icon>
          </template>
          登录
        </n-button>
        <n-button @click="settingsDialogShowing = true">
          <template #icon>
            <n-icon>
              <SettingOutlined />
            </n-icon>
          </template>
          配置
        </n-button>
        <n-button @click="aboutDialogShowing = true">
          <template #icon>
            <n-icon>
              <QuestionCircleOutlined />
            </n-icon>
          </template>
          关于
        </n-button>
        <div v-if="userProfile !== undefined" class="flex items-center ml-auto overflow-hidden">
          <n-avatar
            class="flex-shrink-0"
            round
            :size="32"
            :src="userProfile.photo"
            fallback-src="https://cdn-msp.18comic.vip/templates/frontend/airav/img/title-png/more-ms-jm.webp?v=2" />
          <span class="whitespace-nowrap text-ellipsis overflow-hidden" :title="userProfile.username">
            {{ userProfile.username }}
          </span>
        </div>
      </div>
      <downloading-pane v-model:config="config" />
    </div>
    <login-dialog v-model:showing="loginDialogShowing" v-model:config="config" v-model:user-profile="userProfile" />
    <settings-dialog v-model:showing="settingsDialogShowing" v-model:config="config" />
    <about-dialog v-model:showing="aboutDialogShowing" />
  </div>
</template>

<style scoped>
:global(.n-notification-main__header) {
  @apply break-words;
}

:global(.n-tabs-pane-wrapper) {
  @apply h-full;
}

:deep(.n-tabs-nav) {
  @apply px-2;
}
</style>
