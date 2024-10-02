use anyhow::Context;
use tauri::{Manager, Wry};

// TODO: 用prelude来消除警告
use crate::commands::*;
use crate::config::Config;
use crate::events::prelude::*;

mod commands;
mod config;
mod download_manager;
mod errors;
mod events;
mod extensions;
mod jm_client;
mod responses;
mod types;
mod utils;

fn generate_context() -> tauri::Context<Wry> {
    tauri::generate_context!()
}

// TODO: 添加Panic Doc
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<Wry>::new()
        .commands(tauri_specta::collect_commands![
            greet,
            get_config,
            save_config,
            login,
            search,
            get_album,
            get_chapter,
            get_scramble_id,
            get_favorite_folder,
            get_user_profile,
            download_chapters,
            show_path_in_file_manager,
            sync_favorite_folder,
        ])
        .events(tauri_specta::collect_events![
            DownloadChapterEndEvent,
            DownloadChapterPendingEvent,
            DownloadChapterStartEvent,
            DownloadImageErrorEvent,
            DownloadImageSuccessEvent,
            DownloadSpeedEvent,
            UpdateOverallDownloadProgressEvent,
        ]);

    #[cfg(debug_assertions)]
    builder
        .export(
            specta_typescript::Typescript::default()
                .bigint(specta_typescript::BigIntExportBehavior::Number)
                .formatter(specta_typescript::formatter::prettier)
                .header("// @ts-nocheck"), // 跳过检查
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            let app_data_dir = app
                .path()
                .app_data_dir()
                .context("failed to get app data dir")?;

            std::fs::create_dir_all(&app_data_dir)
                .context(format!("failed to create app data dir: {app_data_dir:?}"))?;
            println!("app data dir: {app_data_dir:?}");

            let config = std::sync::RwLock::new(Config::new(app.handle())?);
            let jm_client = jm_client::JmClient::new(app.handle().clone());
            let download_manager = download_manager::DownloadManager::new(app.handle().clone());

            app.manage(config);
            app.manage(jm_client);
            app.manage(download_manager);

            Ok(())
        })
        .run(generate_context())
        .expect("error while running tauri application");
}
