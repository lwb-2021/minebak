#![feature(error_generic_member_access)]

mod backup;
mod errors;

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::backup::MinecraftInstanceRoot;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct AppStateInner {
    instance_roots: Vec<MinecraftInstanceRoot>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(Mutex::new(AppStateInner::default()));
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            backup::add_root,
            backup::rescan_saves,
            backup::list_instances
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
