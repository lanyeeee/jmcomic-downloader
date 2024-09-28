use std::sync::RwLock;

use tauri::State;

use crate::config::Config;
use crate::errors::CommandResult;
use crate::jm_client::JmClient;
use crate::responses::UserProfileRespData;

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
