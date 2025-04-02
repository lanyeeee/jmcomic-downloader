<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { commands } from './bindings.ts'
import { useMessage, useNotification } from 'naive-ui'
import LoginDialog from './components/LoginDialog.vue'
import SearchPane from './panes/SearchPane.vue'
import ChapterPane from './panes/ChapterPane.vue'
import DownloadingPane from './panes/DownloadingPane.vue'
import FavoritePane from './panes/FavoritePane.vue'
import AboutDialog from './components/AboutDialog.vue'
import { QuestionCircleOutlined, UserOutlined, SettingOutlined } from '@vicons/antd'
import SettingsDialog from './components/SettingsDialog.vue'
import DownloadedPane from './panes/DownloadedPane.vue'
import { useStore } from './store.ts'

const store = useStore()

const message = useMessage()
const notification = useNotification()

const loginDialogShowing = ref<boolean>(false)
const settingsDialogShowing = ref<boolean>(false)
const aboutDialogShowing = ref<boolean>(false)

watch(
  () => store.config,
  async () => {
    if (store.config === undefined) {
      return
    }

    const result = await commands.saveConfig(store.config)
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
  store.config = await commands.getConfig()
  // 如果username和password不为空，尝试登录
  if (store.config.username !== '' && store.config.password !== '') {
    const result = await commands.login(store.config.username, store.config.password)
    if (result.status === 'error') {
      notification.error({ title: '自动登录失败', description: result.error })
      return
    }
    store.userProfile = result.data
    message.success('自动登录成功')
  }
})
</script>

<template>
  <div v-if="store.config !== undefined" class="h-screen flex overflow-hidden">
    <n-tabs class="h-full w-1/2" v-model:value="store.currentTabName" type="line" size="small" animated>
      <n-tab-pane class="h-full overflow-auto p-0!" name="search" tab="漫画搜索" display-directive="show">
        <search-pane />
      </n-tab-pane>
      <n-tab-pane class="h-full overflow-auto p-0!" name="favorite" tab="漫画收藏" display-directive="show">
        <favorite-pane />
      </n-tab-pane>
      <n-tab-pane class="h-full overflow-auto p-0!" name="downloaded" tab="本地库存" display-directive="show">
        <downloaded-pane />
      </n-tab-pane>
      <n-tab-pane class="h-full overflow-auto p-0!" name="chapter" tab="章节详情" display-directive="show">
        <chapter-pane />
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
        <div v-if="store.userProfile !== undefined" class="flex items-center ml-auto overflow-hidden">
          <n-avatar
            class="flex-shrink-0"
            round
            :size="32"
            :src="store.userProfile.photo"
            fallback-src="https://cdn-msp.18comic.vip/templates/frontend/airav/img/title-png/more-ms-jm.webp?v=2" />
          <span class="whitespace-nowrap text-ellipsis overflow-hidden" :title="store.userProfile.username">
            {{ store.userProfile.username }}
          </span>
        </div>
      </div>
      <downloading-pane />
    </div>
    <login-dialog v-model:showing="loginDialogShowing" />
    <settings-dialog v-model:showing="settingsDialogShowing" />
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
