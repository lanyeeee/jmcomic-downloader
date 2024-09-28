<script setup lang="ts">
import {onMounted, ref, watch} from "vue";
import {commands, Config, UserProfileRespData} from "./bindings.ts";
import {useMessage} from "naive-ui";
import LoginDialog from "./components/LoginDialog.vue";

const message = useMessage();

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
    </div>

    <n-modal v-model:show="loginDialogShowing">
      <login-dialog v-model:showing="loginDialogShowing" v-model:config="config" v-model:userProfile="userProfile"/>
    </n-modal>
  </div>
</template>
