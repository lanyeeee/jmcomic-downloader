use std::path::PathBuf;
use std::sync::atomic::AtomicI64;
use std::sync::Arc;

// TODO: 用`#![allow(clippy::used_underscore_binding)]`来消除警告
use anyhow::anyhow;
use parking_lot::{Mutex, RwLock};
use path_slash::PathBufExt;
use tauri::{AppHandle, State};
use tauri_specta::Event;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::config::Config;
use crate::download_manager::DownloadManager;
use crate::errors::CommandResult;
use crate::events::UpdateDownloadedFavoriteAlbumEvent;
use crate::jm_client::JmClient;
use crate::responses::{ChapterRespData, FavoriteRespData, UserProfileRespData};
use crate::types::{Album, ChapterInfo, FavoriteSort, SearchResult, SearchSort};

#[tauri::command]
#[specta::specta]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn get_config(config: State<RwLock<Config>>) -> Config {
    config.read().clone()
}

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn save_config(
    app: AppHandle,
    jm_client: State<'_, RwLock<JmClient>>,
    download_manager: State<'_, RwLock<DownloadManager>>,
    config_state: State<RwLock<Config>>,
    config: Config,
) -> CommandResult<()> {
    let need_recreate = {
        let config_state = config_state.read();
        config_state.proxy_mode != config.proxy_mode
            || config_state.proxy_host != config.proxy_host
            || config_state.proxy_port != config.proxy_port
    };

    let mut config_state = config_state.write();
    *config_state = config;
    config_state.save(&app)?;
    drop(config_state);

    if need_recreate {
        jm_client.write().recreate_http_client();
        download_manager.write().recreate_http_client();
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn login(
    jm_client: State<'_, RwLock<JmClient>>,
    username: String,
    password: String,
) -> CommandResult<UserProfileRespData> {
    let jm_client = jm_client.read().clone();
    let user_profile = jm_client.login(&username, &password).await?;
    Ok(user_profile)
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_profile(
    jm_client: State<'_, RwLock<JmClient>>,
) -> CommandResult<UserProfileRespData> {
    let jm_client = jm_client.read().clone();
    let user_profile = jm_client.get_user_profile().await?;
    Ok(user_profile)
}

#[tauri::command]
#[specta::specta]
pub async fn search(
    app: AppHandle,
    jm_client: State<'_, RwLock<JmClient>>,
    keyword: String,
    page: i64,
    sort: SearchSort,
) -> CommandResult<SearchResult> {
    let jm_client = jm_client.read().clone();
    let search_resp = jm_client.search(&keyword, page, sort).await?;
    let search_result = SearchResult::from_search_resp(&app, search_resp);
    Ok(search_result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_album(
    app: AppHandle,
    jm_client: State<'_, RwLock<JmClient>>,
    aid: i64,
) -> CommandResult<Album> {
    let jm_client = jm_client.read().clone();
    let album_resp_data = jm_client.get_album(aid).await?;
    let album = Album::from_album_resp_data(&app, album_resp_data);
    Ok(album)
}

#[tauri::command]
#[specta::specta]
pub async fn get_chapter(
    jm_client: State<'_, RwLock<JmClient>>,
    id: i64,
) -> CommandResult<ChapterRespData> {
    let jm_client = jm_client.read().clone();
    // TODO: 变量名改为chapter_resp_data
    let chapter = jm_client.get_chapter(id).await?;
    Ok(chapter)
}

#[tauri::command]
#[specta::specta]
pub async fn get_scramble_id(
    jm_client: State<'_, RwLock<JmClient>>,
    id: i64,
) -> CommandResult<i64> {
    let jm_client = jm_client.read().clone();
    let scramble_id = jm_client.get_scramble_id(id).await?;
    Ok(scramble_id)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_favorite_folder(
    jm_client: State<'_, RwLock<JmClient>>,
    folder_id: i64,
    page: i64,
    sort: FavoriteSort,
) -> CommandResult<FavoriteRespData> {
    let jm_client = jm_client.read().clone();
    let favorite_resp_data = jm_client.get_favorite_folder(folder_id, page, sort).await?;
    Ok(favorite_resp_data)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn download_chapters(
    download_manager: State<'_, RwLock<DownloadManager>>,
    chapter_infos: Vec<ChapterInfo>,
) -> CommandResult<()> {
    let download_manager = download_manager.read().clone();
    for chapter_info in chapter_infos {
        download_manager.submit_chapter(chapter_info).await?;
    }
    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub async fn download_album(
    app: AppHandle,
    jm_client: State<'_, RwLock<JmClient>>,
    download_manager: State<'_, RwLock<DownloadManager>>,
    aid: i64,
) -> CommandResult<()> {
    let album = get_album(app, jm_client, aid).await?;
    let chapter_infos: Vec<ChapterInfo> = album
        .chapter_infos
        .into_iter()
        .filter(|chapter_info| !chapter_info.is_downloaded)
        .collect();
    if chapter_infos.is_empty() {
        let album_title = album.name;
        return Err(
            anyhow!("漫画`{album_title}`的所有章节都已存在于下载目录，无需重复下载").into(),
        );
    }
    download_chapters(download_manager, chapter_infos).await?;
    Ok(())
}

#[allow(clippy::cast_possible_wrap)]
#[tauri::command(async)]
#[specta::specta]
pub async fn update_downloaded_favorite_album(
    app: AppHandle,
    jm_client: State<'_, RwLock<JmClient>>,
    download_manager: State<'_, RwLock<DownloadManager>>,
) -> CommandResult<()> {
    let jm_client = jm_client.read().clone();
    let favorite_albums = Arc::new(Mutex::new(vec![]));
    // 发送正在获取收藏夹事件
    let _ = UpdateDownloadedFavoriteAlbumEvent::GettingFolders.emit(&app);
    // 获取收藏夹第一页
    let first_page = jm_client
        .get_favorite_folder(0, 1, FavoriteSort::FavoriteTime)
        .await?;
    favorite_albums.lock().extend(first_page.list);
    // 计算总页数
    let count = first_page.count;
    let total = first_page.total.parse::<i64>().map_err(|e| anyhow!(e))?;
    let page_count = (total / count) + 1;
    // 获取收藏夹剩余页
    let mut join_set = JoinSet::new();
    for page in 2..=page_count {
        let jm_client = jm_client.clone();
        let favorite_albums = favorite_albums.clone();
        join_set.spawn(async move {
            let page = jm_client
                .get_favorite_folder(0, page, FavoriteSort::FavoriteTime)
                .await?;
            favorite_albums.lock().extend(page.list);
            Ok::<(), anyhow::Error>(())
        });
    }
    // 等待所有请求完成
    while let Some(Ok(get_favorite_result)) = join_set.join_next().await {
        // 如果有请求失败，直接返回错误
        get_favorite_result?;
    }
    // 至此，收藏夹已经全部获取完毕
    let favorite_albums = std::mem::take(&mut *favorite_albums.lock());
    let albums = Arc::new(Mutex::new(vec![]));
    // 限制并发数为10
    let sem = Arc::new(Semaphore::new(10));
    let current = Arc::new(AtomicI64::new(0));
    // 发送正在获取收藏夹漫画详情事件
    let total = favorite_albums.len() as i64;
    let _ = UpdateDownloadedFavoriteAlbumEvent::GettingAlbums { total }.emit(&app);
    // 获取收藏夹漫画的详细信息
    for favorite_album in favorite_albums {
        let sem = sem.clone();
        let aid = favorite_album.id.parse::<i64>().map_err(|e| anyhow!(e))?;
        let jm_client = jm_client.clone();
        let app = app.clone();
        let albums = albums.clone();
        let current = current.clone();
        join_set.spawn(async move {
            let permit = sem.acquire().await?;
            let album_resp_data = jm_client.get_album(aid).await?;
            drop(permit);
            let album = Album::from_album_resp_data(&app, album_resp_data);
            albums.lock().push(album);
            let current = current.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            // 发送获取到收藏夹漫画详情事件
            let _ = UpdateDownloadedFavoriteAlbumEvent::AlbumGot { current, total }.emit(&app);
            Ok::<(), anyhow::Error>(())
        });
    }
    // 等待所有请求完成
    while let Some(Ok(get_album_result)) = join_set.join_next().await {
        // 如果有请求失败，直接返回错误
        get_album_result?;
    }
    // 至此，收藏夹漫画的详细信息已经全部获取完毕
    let albums = std::mem::take(&mut *albums.lock());
    // 过滤出已下载的漫画(至少有一个章节已下载)
    let downloaded_album = albums
        .into_iter()
        .filter(|album| {
            album
                .chapter_infos
                .iter()
                .any(|chapter_info| chapter_info.is_downloaded)
        })
        .collect::<Vec<_>>();
    // 获取已下载的漫画中的未下载章节
    let chapters_to_download = downloaded_album
        .iter()
        .flat_map(|album| {
            album
                .chapter_infos
                .iter()
                .filter(|chapter_info| !chapter_info.is_downloaded)
                .cloned()
        })
        .collect::<Vec<_>>();
    // 下载未下载章节
    download_chapters(download_manager, chapters_to_download).await?;
    // 发送下载任务创建完成事件
    let _ = UpdateDownloadedFavoriteAlbumEvent::DownloadTaskCreated.emit(&app);

    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub fn show_path_in_file_manager(path: &str) -> CommandResult<()> {
    let path = PathBuf::from_slash(path);
    if !path.exists() {
        return Err(anyhow!("路径`{path:?}`不存在").into());
    }
    showfile::show_path_in_file_manager(path);
    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub async fn sync_favorite_folder(jm_client: State<'_, RwLock<JmClient>>) -> CommandResult<()> {
    // 同步收藏夹的方式是随便收藏一个漫画
    // 调用两次toggle是因为要把新收藏的漫画取消收藏
    let jm_client = jm_client.read().clone();
    let task1 = jm_client.toggle_favorite_album(468_984);
    let task2 = jm_client.toggle_favorite_album(468_984);
    let (resp1, resp2) = tokio::try_join!(task1, task2)?;
    if resp1.toggle_type == resp2.toggle_type {
        let toggle_type = resp1.toggle_type;
        return Err(anyhow!("同步收藏夹失败，两个请求都是`{toggle_type:?}`操作，请重试").into());
    }

    Ok(())
}
