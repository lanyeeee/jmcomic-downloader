<script setup lang="ts">
import {ref} from "vue";
import {commands, Config, UserProfileRespData} from "../bindings.ts";
import {useMessage, useNotification} from "naive-ui";

const message = useMessage();
const notification = useNotification();

const showing = defineModel<boolean>("showing", {required: true});
const config = defineModel<Config>("config", {required: true});
const userProfile = defineModel<UserProfileRespData>("userProfile", {required: false});

const username = ref<string>("");
const password = ref<string>("");

async function onLogin() {
  if (username.value === "") {
    message.error("请输入用户名");
    return;
  }
  if (password.value === "") {
    message.error("请输入密码");
    return;
  }
  const result = await commands.login(username.value, password.value);
  if (result.status === "error") {
    notification.error({title: "登录失败", description: result.error});
    return;
  }
  message.success("登录成功");
  userProfile.value = result.data;
  config.value.avs = result.data.s;
  showing.value = false;
}
</script>

<template>
  <n-dialog class="flex flex-col"
            :showIcon="false"
            title="账号登录"
            positive-text="登录"
            @positive-click="onLogin"
            @close="showing=false"
            @keydown.enter="onLogin">
    <n-input v-model:value="username" placeholder="">
      <template #prefix>
        用户名:
      </template>
    </n-input>
    <n-input v-model:value="password" type="password" placeholder="" show-password-on="mousedown">
      <template #prefix>
        密码:
      </template>
    </n-input>
  </n-dialog>
</template>
