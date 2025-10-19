#![feature(error_generic_member_access)]

mod backup;
mod errors;

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use crate::backup::MinecraftInstanceRoot;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct AppStateInner {
    instance_roots: Vec<MinecraftInstanceRoot>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let store = app.store("store.json")?;
            let state: AppStateInner =
                serde_json::from_value(store.get("settings").unwrap_or_default())
                    .unwrap_or_default();
            store.close_resource();
            app.manage(Mutex::new(state));
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
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let store = window.store("store.json").expect("Failed to save settings");
                store.set(
                    "settings",
                    serde_json::to_value(
                        window
                            .state::<Mutex<AppStateInner>>()
                            .lock()
                            .expect("Failed to lock state")
                            .to_owned(),
                    )
                    .expect("App state serialization failed"),
                );
                store.save().expect("Failed to save to storge");
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
