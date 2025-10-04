use anyhow::Context;
use events::{
    DownloadAllFavoritesEvent, DownloadSleepingEvent, DownloadSpeedEvent, DownloadTaskEvent,
    ExportCbzEvent, ExportPdfEvent, LogEvent, UpdateDownloadedComicsEvent,
    UpdateDownloadedFavoriteComicEvent,
};
use parking_lot::RwLock;
use tauri::{Manager, Wry};

// TODO: 用prelude来消除警告
use crate::commands::*;
use crate::config::Config;
use crate::download_manager::DownloadManager;
use crate::jm_client::JmClient;

mod commands;
mod config;
mod download_manager;
mod errors;
mod events;
mod export;
mod extensions;
mod jm_client;
mod logger;
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
            get_comic,
            get_favorite_folder,
            get_weekly_info,
            get_weekly,
            get_user_profile,
            create_download_task,
            pause_download_task,
            resume_download_task,
            cancel_download_task,
            download_comic,
            download_all_favorites,
            update_downloaded_comics,
            update_downloaded_favorite_comic,
            show_path_in_file_manager,
            sync_favorite_folder,
            get_downloaded_comics,
            export_cbz,
            export_pdf,
            get_logs_dir_size,
            get_synced_comic,
            get_synced_comic_in_favorite,
            get_synced_comic_in_search,
            get_synced_comic_in_weekly,
        ])
        .events(tauri_specta::collect_events![
            DownloadSpeedEvent,
            DownloadSleepingEvent,
            DownloadTaskEvent,
            DownloadAllFavoritesEvent,
            UpdateDownloadedComicsEvent,
            UpdateDownloadedFavoriteComicEvent,
            ExportCbzEvent,
            ExportPdfEvent,
            LogEvent,
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
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            let app_data_dir = app
                .path()
                .app_data_dir()
                .context("failed to get app data dir")?;

            std::fs::create_dir_all(&app_data_dir).context(format!(
                "failed to create app data dir: {}",
                app_data_dir.display()
            ))?;

            let config = RwLock::new(Config::new(app.handle())?);
            app.manage(config);

            let jm_client = JmClient::new(app.handle().clone());
            app.manage(jm_client);

            let download_manager = DownloadManager::new(app.handle().clone());
            app.manage(download_manager);

            logger::init(app.handle())?;

            Ok(())
        })
        .run(generate_context())
        .expect("error while running tauri application");
}
