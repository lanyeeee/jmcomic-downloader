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
  await commands.saveConfig(config.value);
  message.success("保存配置成功");
}, {deep: true});

watch(() => config.value?.avs, async () => {
  const profileResult = await commands.getUserProfile();
  if (profileResult.status === "ok") {
    userProfile.value = profileResult.data;
    message.success("获取用户信息成功");
    return;
  }
  const profileNotification = notification.error({title: "获取用户信息失败", description: profileResult.error});
  if (config.value !== undefined && config.value.username !== "" && config.value.password !== "") {
    const loginMessage = message.loading("获取用户信息失败，正在尝试用配置文件中的用户名和密码登录");
    const loginResult = await commands.login(config.value.username, config.value.password);
    if (loginResult.status === "ok") {
      config.value.avs = loginResult.data.s;
      loginMessage.content = "登录成功";
      loginMessage.type = "success";
      profileNotification.type = "success";
      profileNotification.title = "获取用户信息成功";
      profileNotification.description = "使用配置文件中的用户名和密码登录成功";
      return;
    }
    notification.error({title: "登录失败", description: loginResult.error});
  }

  userProfile.value = undefined;
});

onMounted(async () => {
  // 屏蔽浏览器右键菜单
  document.oncontextmenu = (event) => {
    event.preventDefault();
  };
  // 获取配置
  config.value = await commands.getConfig();
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
    <div class="flex">
      <n-input v-model:value="config.avs" placeholder="" clearable>
        <template #prefix>
          AVS：
        </template>
      </n-input>
      <n-button type="primary" @click="loginDialogShowing=true">账号登录</n-button>
      <n-button @click="showConfigInFileManager">打开配置目录</n-button>
      <n-button @click="test">测试用</n-button>
      <div v-if="userProfile!==undefined" class="flex flex-col">
        <!--    TODO: 显示头像    -->
        <span class="whitespace-nowrap">{{ userProfile.username }} Lv{{ userProfile.level }}</span>
      </div>
    </div>
    <div class="flex overflow-hidden">
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
      <downloading-list class="basis-1/2 overflow-auto" v-model:config="config"></downloading-list>
    </div>
    <n-modal v-model:show="loginDialogShowing">
      <login-dialog v-model:showing="loginDialogShowing" v-model:config="config"/>
    </n-modal>
  </div>
</template>
