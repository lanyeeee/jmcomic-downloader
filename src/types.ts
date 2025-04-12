import { DownloadTaskEvent } from './bindings.ts'

export type CurrentTabName = 'search' | 'favorite' | 'downloaded' | 'chapter'

export type ProgressData = Extract<DownloadTaskEvent, { event: 'Create' }>['data'] & {
  percentage: number
  indicator: string
}
