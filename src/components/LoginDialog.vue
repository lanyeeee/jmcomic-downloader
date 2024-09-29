<script setup lang="ts">
import {ref} from "vue";
import {commands, Config} from "../bindings.ts";
import {useMessage, useNotification} from "naive-ui";

const message = useMessage();
const notification = useNotification();

const showing = defineModel<boolean>("showing", {required: true});
const config = defineModel<Config>("config", {required: true});

const username = ref<string>(config.value.username);
const password = ref<string>(config.value.password);
const remember = ref<boolean>(username.value !== "" && password.value !== "");

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
  config.value.avs = result.data.s;
  if (remember.value) {
    config.value.username = username.value;
    config.value.password = password.value;
  }
  showing.value = false;
}

function clearUsernameAndPasswordInConfig() {
  config.value.username = "";
  config.value.password = "";
}

</script>

<template>
  <n-dialog :showIcon="false"
            title="账号登录"
            positive-text="登录"
            @positive-click="onLogin"
            @close="showing=false"
            @keydown.enter="onLogin">
    <div class="flex flex-col gap-row-2">
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
      <div class="flex justify-between">
        <n-tooltip>
          用户名和密码将以明文保存在配置文件中
          <template #trigger>
            <n-checkbox v-model:checked="remember">
              记住我
            </n-checkbox>
          </template>
        </n-tooltip>
        <n-button type="primary" size="tiny" secondary @click="clearUsernameAndPasswordInConfig">
          清除配置文件中的用户名和密码
        </n-button>
      </div>
    </div>
  </n-dialog>
</template>
