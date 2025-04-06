// @ts-nocheck
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

/** user-defined commands **/


export const commands = {
async greet(name: string) : Promise<string> {
    return await TAURI_INVOKE("greet", { name });
},
async getConfig() : Promise<Config> {
    return await TAURI_INVOKE("get_config");
},
async saveConfig(config: Config) : Promise<Result<null, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("save_config", { config }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async login(username: string, password: string) : Promise<Result<GetUserProfileRespData, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("login", { username, password }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async search(keyword: string, page: number, sort: SearchSort) : Promise<Result<SearchResult, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("search", { keyword, page, sort }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getComic(aid: number) : Promise<Result<Comic, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_comic", { aid }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getChapter(id: number) : Promise<Result<GetChapterRespData, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_chapter", { id }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getScrambleId(id: number) : Promise<Result<number, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_scramble_id", { id }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getFavoriteFolder(folderId: number, page: number, sort: FavoriteSort) : Promise<Result<GetFavoriteRespData, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_favorite_folder", { folderId, page, sort }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getUserProfile() : Promise<Result<GetUserProfileRespData, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_user_profile") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async downloadChapters(chapterInfos: ChapterInfo[]) : Promise<Result<null, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("download_chapters", { chapterInfos }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async downloadComic(aid: number) : Promise<Result<null, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("download_comic", { aid }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async updateDownloadedFavoriteComic() : Promise<Result<null, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("update_downloaded_favorite_comic") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async showPathInFileManager(path: string) : Promise<Result<null, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("show_path_in_file_manager", { path }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async syncFavoriteFolder() : Promise<Result<null, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("sync_favorite_folder") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async saveMetadata(comic: Comic) : Promise<Result<null, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("save_metadata", { comic }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getDownloadedComics() : Promise<Result<Comic[], CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_downloaded_comics") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async exportCbz(comic: Comic) : Promise<Result<null, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("export_cbz", { comic }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async exportPdf(comic: Comic) : Promise<Result<null, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("export_pdf", { comic }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async getLogsDirSize() : Promise<Result<number, CommandError>> {
    try {
    return { status: "ok", data: await TAURI_INVOKE("get_logs_dir_size") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
}
}

/** user-defined events **/


export const events = __makeEvents__<{
downloadEvent: DownloadEvent,
exportCbzEvent: ExportCbzEvent,
exportPdfEvent: ExportPdfEvent,
logEvent: LogEvent,
updateDownloadedFavoriteComicEvent: UpdateDownloadedFavoriteComicEvent
}>({
downloadEvent: "download-event",
exportCbzEvent: "export-cbz-event",
exportPdfEvent: "export-pdf-event",
logEvent: "log-event",
updateDownloadedFavoriteComicEvent: "update-downloaded-favorite-comic-event"
})

/** user-defined constants **/



/** user-defined types **/

export type CategoryRespData = { id: string | null; title: string | null }
export type CategorySubRespData = { id: string | null; title: string | null }
export type ChapterInfo = { chapterId: number; chapterTitle: string; comicId: number; comicTitle: string; author: string[]; isDownloaded?: boolean | null; order: number }
export type Comic = { id: number; name: string; addtime: string; description: string; total_views: string; likes: string; chapterInfos: ChapterInfo[]; series_id: string; comment_total: string; author: string[]; tags: string[]; works: string[]; actors: string[]; related_list: RelatedListRespData[]; liked: boolean; is_favorite: boolean; is_aids: boolean; isDownloaded?: boolean | null }
export type ComicInFavoriteRespData = { id: string; author: string; description: string | null; name: string; latest_ep: string | null; latest_ep_aid: string | null; image: string; category: CategoryRespData; category_sub: CategorySubRespData }
export type ComicInSearchRespData = { id: string; author: string; name: string; image: string; category: CategoryRespData; category_sub: CategorySubRespData; liked: boolean; is_favorite: boolean; update_at: number }
export type CommandError = { err_title: string; err_message: string }
export type Config = { username: string; password: string; downloadDir: string; exportDir: string; downloadFormat: DownloadFormat; proxyMode: ProxyMode; proxyHost: string; proxyPort: number; enableFileLogger: boolean }
export type DownloadEvent = { event: "ChapterPending"; data: { chapterId: number; comicTitle: string; chapterTitle: string } } | { event: "ChapterStart"; data: { chapterId: number; total: number } } | { event: "ChapterEnd"; data: { chapterId: number; errMsg: string | null } } | { event: "ImageSuccess"; data: { chapterId: number; url: string; current: number } } | { event: "ImageError"; data: { chapterId: number; url: string; errMsg: string } } | { event: "OverallUpdate"; data: { downloadedImageCount: number; totalImageCount: number; percentage: number } } | { event: "OverallSpeed"; data: { speed: string } }
export type DownloadFormat = "Jpeg" | "Png" | "Webp"
export type ExportCbzEvent = { event: "Start"; data: { uuid: string; comicTitle: string; total: number } } | { event: "Progress"; data: { uuid: string; current: number } } | { event: "Error"; data: { uuid: string } } | { event: "End"; data: { uuid: string } }
export type ExportPdfEvent = { event: "CreateStart"; data: { uuid: string; comicTitle: string; total: number } } | { event: "CreateProgress"; data: { uuid: string; current: number } } | { event: "CreateError"; data: { uuid: string } } | { event: "CreateEnd"; data: { uuid: string } } | { event: "MergeStart"; data: { uuid: string; comicTitle: string } } | { event: "MergeError"; data: { uuid: string } } | { event: "MergeEnd"; data: { uuid: string } }
export type FavoriteFolderRespData = { FID: string; UID: string; name: string }
export type FavoriteSort = "FavoriteTime" | "UpdateTime"
export type GetChapterRespData = { id: number; series: SeriesRespData[]; tags: string; name: string; images: string[]; addtime: string; series_id: string; is_favorite: boolean; liked: boolean }
export type GetFavoriteRespData = { list: ComicInFavoriteRespData[]; folder_list: FavoriteFolderRespData[]; total: string; count: number }
export type GetUserProfileRespData = { uid: string; username: string; email: string; emailverified: string; photo: string; fname: string; gender: string; message: string | null; coin: number; album_favorites: number; s: string; level_name: string; level: number; nextLevelExp: number; exp: string; expPercent: number; album_favorites_max: number; ad_free: boolean; charge: string; jar: string; invitation_qrcode: string; invitation_url: string; invited_cnt: string }
export type JsonValue = null | boolean | number | string | JsonValue[] | { [key in string]: JsonValue }
export type LogEvent = { timestamp: string; level: LogLevel; fields: { [key in string]: JsonValue }; target: string; filename: string; line_number: number }
export type LogLevel = "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"
export type ProxyMode = "System" | "NoProxy" | "Custom"
export type RelatedListRespData = { id: string; author: string; name: string; image: string }
export type SearchRespData = { search_query: string; total: number; content: ComicInSearchRespData[] }
export type SearchResult = { SearchRespData: SearchRespData } | { Comic: Comic }
export type SearchSort = "Latest" | "View" | "Picture" | "Like"
export type SeriesRespData = { id: string; name: string; sort: string }
export type UpdateDownloadedFavoriteComicEvent = { event: "GettingFolders" } | { event: "GettingComics"; data: { total: number } } | { event: "ComicGot"; data: { current: number; total: number } } | { event: "DownloadTaskCreated" }

/** tauri-specta globals **/

import {
	invoke as TAURI_INVOKE,
	Channel as TAURI_CHANNEL,
} from "@tauri-apps/api/core";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";

type __EventObj__<T> = {
	listen: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
	once: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
	emit: null extends T
		? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
		: (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

export type Result<T, E> =
	| { status: "ok"; data: T }
	| { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
	mappings: Record<keyof T, string>,
) {
	return new Proxy(
		{} as unknown as {
			[K in keyof T]: __EventObj__<T[K]> & {
				(handle: __WebviewWindow__): __EventObj__<T[K]>;
			};
		},
		{
			get: (_, event) => {
				const name = mappings[event as keyof T];

				return new Proxy((() => {}) as any, {
					apply: (_, __, [window]: [__WebviewWindow__]) => ({
						listen: (arg: any) => window.listen(name, arg),
						once: (arg: any) => window.once(name, arg),
						emit: (arg: any) => window.emit(name, arg),
					}),
					get: (_, command: keyof __EventObj__<any>) => {
						switch (command) {
							case "listen":
								return (arg: any) => TAURI_API_EVENT.listen(name, arg);
							case "once":
								return (arg: any) => TAURI_API_EVENT.once(name, arg);
							case "emit":
								return (arg: any) => TAURI_API_EVENT.emit(name, arg);
						}
					},
				});
			},
		},
	);
}
