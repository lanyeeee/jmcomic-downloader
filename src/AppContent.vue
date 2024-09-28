<script setup lang="ts">
import {onMounted, ref, watch} from "vue";
import {commands, Config, UserProfileRespData} from "./bindings.ts";
import {useMessage, useNotification} from "naive-ui";
import LoginDialog from "./components/LoginDialog.vue";

const message = useMessage();
const notification = useNotification();

const config = ref<Config>();
const userProfile = ref<UserProfileRespData>();
const loginDialogShowing = ref<boolean>(false);

watch(config, async () => {
  if (config.value === undefined) {
    return;
  }
  await commands.saveConfig(config.value);
  message.success("保存配置成功");
}, {deep: true});

watch(() => config.value?.avs, async () => {
  const result = await commands.getUserProfile();
  if (result.status === "error") {
    notification.error({title: "获取用户信息失败", description: result.error});
    userProfile.value = undefined;
    return;
  }
  userProfile.value = result.data;
  message.success("获取用户信息成功");
});

onMounted(async () => {
  config.value = await commands.getConfig();
});

async function test() {
  const result = await commands.getUserProfile();
  if (result.status === "error") {
    message.error(result.error);
    return;
  }
  console.log(result.data);
}

</script>

<template>
  <div v-if="config!==undefined" class="h-full flex flex-col">
    <div class="flex">
      <n-input v-model:value="config.avs" placeholder="" clearable>
        <template #prefix>
          AVS：
        </template>
      </n-input>
      <n-button type="primary" @click="loginDialogShowing=true">账号登录</n-button>
      <n-button @click="test">测试用</n-button>
      <div v-if="userProfile!==undefined" class="flex flex-col">
        <!--    TODO: 显示头像    -->
        <span class="whitespace-nowrap">{{ userProfile.username }} Lv{{ userProfile.level }}</span>
      </div>
    </div>

    <n-modal v-model:show="loginDialogShowing">
      <login-dialog v-model:showing="loginDialogShowing" v-model:config="config" v-model:userProfile="userProfile"/>
    </n-modal>
  </div>
</template>
