use std::path::PathBuf;
use std::sync::RwLock;

// TODO: 用`#![allow(clippy::used_underscore_binding)]`来消除警告
use anyhow::anyhow;
use path_slash::PathBufExt;
use tauri::{AppHandle, State};

use crate::config::Config;
use crate::download_manager::DownloadManager;
use crate::errors::CommandResult;
use crate::extensions::IgnoreRwLockPoison;
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
    config.read().unwrap().clone()
}

#[tauri::command(async)]
#[specta::specta]
#[allow(clippy::needless_pass_by_value)]
pub fn save_config(
    app: AppHandle,
    config_state: State<RwLock<Config>>,
    config: Config,
) -> CommandResult<()> {
    let mut config_state = config_state.write_or_panic();
    *config_state = config;
    config_state.save(&app)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn login(
    jm_client: State<'_, JmClient>,
    username: String,
    password: String,
) -> CommandResult<UserProfileRespData> {
    let user_profile = jm_client.login(&username, &password).await?;
    Ok(user_profile)
}

#[tauri::command]
#[specta::specta]
pub async fn get_user_profile(
    jm_client: State<'_, JmClient>,
) -> CommandResult<UserProfileRespData> {
    let user_profile = jm_client.get_user_profile().await?;
    Ok(user_profile)
}

#[tauri::command]
#[specta::specta]
pub async fn search(
    app: AppHandle,
    jm_client: State<'_, JmClient>,
    keyword: String,
    page: i64,
    sort: SearchSort,
) -> CommandResult<SearchResult> {
    let search_resp = jm_client.search(&keyword, page, sort).await?;
    let search_result = SearchResult::from_search_resp(&app, search_resp);

    Ok(search_result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_album(
    app: AppHandle,
    jm_client: State<'_, JmClient>,
    aid: i64,
) -> CommandResult<Album> {
    let album_resp_data = jm_client.get_album(aid).await?;
    let album = Album::from_album_resp_data(&app, album_resp_data);
    Ok(album)
}

#[tauri::command]
#[specta::specta]
pub async fn get_chapter(
    jm_client: State<'_, JmClient>,
    id: i64,
) -> CommandResult<ChapterRespData> {
    // TODO: 变量名改为chapter_resp_data
    let chapter = jm_client.get_chapter(id).await?;
    Ok(chapter)
}

#[tauri::command]
#[specta::specta]
pub async fn get_scramble_id(jm_client: State<'_, JmClient>, id: i64) -> CommandResult<i64> {
    let scramble_id = jm_client.get_scramble_id(id).await?;
    Ok(scramble_id)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_favorite_folder(
    jm_client: State<'_, JmClient>,
    folder_id: i64,
    page: i64,
    sort: FavoriteSort,
) -> CommandResult<FavoriteRespData> {
    let favorite_resp_data = jm_client.get_favorite_folder(folder_id, page, sort).await?;
    Ok(favorite_resp_data)
}

#[tauri::command(async)]
#[specta::specta]
pub async fn download_chapters(
    download_manager: State<'_, DownloadManager>,
    chapter_infos: Vec<ChapterInfo>,
) -> CommandResult<()> {
    for chapter_info in chapter_infos {
        download_manager.submit_chapter(chapter_info).await?;
    }
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
pub async fn sync_favorite_folder(jm_client: State<'_, JmClient>) -> CommandResult<()> {
    // 同步收藏夹的方式是随便收藏一个漫画
    // 调用两次toggle是因为要把新收藏的漫画取消收藏
    let task1 = jm_client.toggle_favorite_album(468_984);
    let task2 = jm_client.toggle_favorite_album(468_984);
    let (resp1, resp2) = tokio::try_join!(task1, task2)?;
    if resp1.toggle_type == resp2.toggle_type {
        let toggle_type = resp1.toggle_type;
        return Err(anyhow!("同步收藏夹失败，两个请求都是`{toggle_type:?}`操作，请重试").into());
    }

    Ok(())
}
