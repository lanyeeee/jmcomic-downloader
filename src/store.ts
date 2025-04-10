import { defineStore } from 'pinia'
import { CurrentTabName, ProgressData } from './types.ts'
import { Comic, Config, GetUserProfileRespData } from './bindings.ts'
import { ref } from 'vue'

export const useStore = defineStore('store', () => {
  const config = ref<Config>()
  const userProfile = ref<GetUserProfileRespData>()
  const pickedComic = ref<Comic>()
  const currentTabName = ref<CurrentTabName>('search')
  const progresses = ref<Map<number, ProgressData>>(new Map())

  return { config, userProfile, pickedComic, currentTabName, progresses }
})
