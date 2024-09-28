use std::sync::RwLock;

use tauri::{AppHandle, State};

use crate::config::Config;
use crate::errors::CommandResult;
use crate::extensions::IgnoreRwLockPoison;
use crate::jm_client::JmClient;
use crate::responses::{AlbumRespData, ChapterRespData, SearchResp, UserProfileRespData};
use crate::types::SearchSort;

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
pub async fn search(
    jm_client: State<'_, JmClient>,
    keyword: String,
    page: i64,
    sort: SearchSort,
) -> CommandResult<SearchResp> {
    let search_result = jm_client.search(&keyword, page, sort).await?;
    Ok(search_result)
}

#[tauri::command]
#[specta::specta]
pub async fn get_album(jm_client: State<'_, JmClient>, aid: i64) -> CommandResult<AlbumRespData> {
    let album = jm_client.get_album(aid).await?;
    Ok(album)
}

#[tauri::command]
#[specta::specta]
pub async fn get_chapter(
    jm_client: State<'_, JmClient>,
    id: i64,
) -> CommandResult<ChapterRespData> {
    let chapter = jm_client.get_chapter(id).await?;
    Ok(chapter)
}

#[tauri::command]
#[specta::specta]
pub async fn get_scramble_id(jm_client: State<'_, JmClient>, id: i64) -> CommandResult<i64> {
    let scramble_id = jm_client.get_scramble_id(id).await?;
    Ok(scramble_id)
}
