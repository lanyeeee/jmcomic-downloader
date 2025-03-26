<script setup lang="ts">
import { ref } from 'vue'
import { commands, Config, GetUserProfileRespData } from '../bindings.ts'
import { useMessage, useNotification } from 'naive-ui'
import FloatLabelInput from './FloatLabelInput.vue'

const message = useMessage()
const notification = useNotification()

const showing = defineModel<boolean>('showing', { required: true })
const config = defineModel<Config>('config', { required: true })
const userProfile = defineModel<GetUserProfileRespData | undefined>('userProfile', { required: true })

const username = ref<string>(config.value.username)
const password = ref<string>(config.value.password)
const remember = ref<boolean>(username.value !== '' && password.value !== '')

async function onLogin() {
  if (username.value === '') {
    message.error('请输入用户名')
    return
  }
  if (password.value === '') {
    message.error('请输入密码')
    return
  }
  const result = await commands.login(username.value, password.value)
  if (result.status === 'error') {
    notification.error({ title: '登录失败', description: result.error })
    return
  }
  userProfile.value = result.data
  message.success('登录成功')
  if (remember.value) {
    config.value.username = username.value
    config.value.password = password.value
  }
  showing.value = false
}

function clearUsernameAndPasswordInConfig() {
  config.value.username = ''
  config.value.password = ''
}
</script>

<template>
  <n-modal v-model:show="showing">
    <n-dialog
      :showIcon="false"
      title="账号登录"
      positive-text="登录"
      @positive-click="onLogin"
      @close="showing = false"
      @keydown.enter="onLogin">
      <div class="flex flex-col gap-2">
        <FloatLabelInput label="用户名" v-model:value="username" />
        <FloatLabelInput label="密码" v-model:value="password" type="password" />
        <div class="flex justify-between">
          <n-tooltip>
            用户名和密码将以明文保存在配置文件中
            <template #trigger>
              <n-checkbox v-model:checked="remember">记住我</n-checkbox>
            </template>
          </n-tooltip>
          <n-button type="primary" size="tiny" secondary @click="clearUsernameAndPasswordInConfig">
            清除配置文件中的用户名和密码
          </n-button>
        </div>
      </div>
    </n-dialog>
  </n-modal>
</template>
