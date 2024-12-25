<script setup lang="ts">
import {onMounted, ref, watch} from "vue";
import {Album, commands, Config, UserProfileRespData} from "./bindings.ts";
import {useMessage, useNotification} from "naive-ui";
import LoginDialog from "./components/LoginDialog.vue";
import SearchPane from "./components/SearchPane.vue";
import ChapterPane from "./components/ChapterPane.vue";
import DownloadingList from "./components/DownloadingList.vue";
import {appDataDir} from "@tauri-apps/api/path";
import {path} from "@tauri-apps/api";
import FavoritePane from "./components/FavoritePane.vue";

const message = useMessage();
const notification = useNotification();

const config = ref<Config>();
const userProfile = ref<UserProfileRespData>();
const loginDialogShowing = ref<boolean>(false);
const currentTabName = ref<"search" | "favorite" | "chapter">("search");
const selectedAlbum = ref<Album>();

watch(config, async () => {
  if (config.value === undefined) {
    return;
  }
  // TODO: 这里应该检查result是否为error
  await commands.saveConfig(config.value);
  message.success("保存配置成功");
}, {deep: true});

onMounted(async () => {
  // 屏蔽浏览器右键菜单
  document.oncontextmenu = (event) => {
    event.preventDefault();
  };
  // 获取配置
  config.value = await commands.getConfig();
  // 如果username和password不为空，尝试登录
  if (config.value.username !== "" && config.value.password !== "") {
    const result = await commands.login(config.value.username, config.value.password);
    if (result.status === "error") {
      notification.error({title: "自动登录失败", description: result.error});
      return;
    }
    userProfile.value = result.data;
    message.success("自动登录成功");
  }
});

async function showConfigInFileManager() {
  const configName = "config.json";
  const configPath = await path.join(await appDataDir(), configName);
  const result = await commands.showPathInFileManager(configPath);
  if (result.status === "error") {
    notification.error({title: "打开配置目录失败", description: result.error});
  }
}

async function test() {
  const result = await commands.getFavoriteFolder(0, 1, "FavoriteTime");
  if (result.status === "error") {
    notification.error({title: "出现错误", description: result.error});
    return;
  }
  console.log(result.data);
}

</script>

<template>
  <div v-if="config!==undefined" class="h-screen flex flex-col">
    <div class="h-full flex overflow-hidden">
      <n-tabs class="basis-1/2 overflow-auto" v-model:value="currentTabName" type="line" size="small">
        <n-tab-pane class="h-full overflow-auto p-0!" name="search" tab="漫画搜索" display-directive="show:lazy">
          <search-pane v-model:selected-album="selectedAlbum" v-model:current-tab-name="currentTabName"/>
        </n-tab-pane>
        <n-tab-pane class="h-full overflow-auto p-0!" name="favorite" tab="漫画收藏" display-directive="show:lazy">
          <favorite-pane :user-profile="userProfile"
                         v-model:selected-album="selectedAlbum"
                         v-model:current-tab-name="currentTabName"/>
        </n-tab-pane>
        <n-tab-pane class="h-full overflow-auto p-0!" name="chapter" tab="章节详情" display-directive="show:lazy">
          <chapter-pane v-model:selected-album="selectedAlbum"/>
        </n-tab-pane>
      </n-tabs>
      <div class="basis-1/2 flex flex-col">
        <div class="flex">
          <n-button type="primary" @click="loginDialogShowing=true">账号登录</n-button>
          <n-button @click="showConfigInFileManager">打开配置目录</n-button>
          <n-button @click="test">测试用</n-button>
          <div v-if="userProfile!==undefined" class="flex flex-col">
            <!--    TODO: 显示头像    -->
            <span class="whitespace-nowrap">{{ userProfile.username }} Lv{{ userProfile.level }}</span>
          </div>
        </div>
        <downloading-list class="overflow-auto" v-model:config="config"></downloading-list>
      </div>
    </div>
    <n-modal v-model:show="loginDialogShowing">
      <login-dialog v-model:showing="loginDialogShowing" v-model:config="config" v-model:user-profile="userProfile"/>
    </n-modal>
  </div>
</template>
