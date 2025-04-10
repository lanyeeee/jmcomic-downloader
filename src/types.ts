import { CategoryRespData, CategorySubRespData, DownloadTaskEvent } from './bindings.ts'

export type CurrentTabName = 'search' | 'favorite' | 'downloaded' | 'chapter'

export type ComicInfo = {
  id: string
  author: string
  name: string
  category: CategoryRespData
  category_sub: CategorySubRespData
}

export type ProgressData = Extract<DownloadTaskEvent, { event: 'Create' }>['data'] & {
  percentage: number
  indicator: string
}
