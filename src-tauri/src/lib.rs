use anyhow::Context;
use tauri::{Manager, Wry};

use crate::commands::*;
use crate::config::Config;

mod commands;
mod config;

fn generate_context() -> tauri::Context<Wry> {
    tauri::generate_context!()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri_specta::Builder::<Wry>::new()
        .commands(tauri_specta::collect_commands![greet, get_config])
        .events(tauri_specta::collect_events![]);

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
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .context("failed to get app data dir")?;

            std::fs::create_dir_all(&app_data_dir)
                .context(format!("failed to create app data dir: {app_data_dir:?}"))?;

            let config = std::sync::RwLock::new(Config::new(app.handle())?);
            app.manage(config);
            Ok(())
        })
        .run(generate_context())
        .expect("error while running tauri application");
}
