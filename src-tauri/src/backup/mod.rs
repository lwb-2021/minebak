mod base;
mod instance;
mod nbt;
mod save;

#[allow(unused_imports)]
pub use base::MinecraftInstanceMetadata;
pub use base::MinecraftInstanceType;
pub use instance::MinecraftInstance;
pub use instance::MinecraftInstanceRoot;
use tauri::State;

use crate::errors::MyError;
use crate::errors::Result;
use crate::AppStateInner;
use std::path::PathBuf;
use std::sync::Mutex;

#[tauri::command]
pub async fn add_root(
    path: String,
    instance_type: u8,
    state: State<'_, Mutex<AppStateInner>>,
) -> Result<()> {
    let root: PathBuf = PathBuf::from(path);
    state
        .lock()
        .unwrap()
        .instance_roots
        .push(MinecraftInstanceRoot::new(
            root,
            match instance_type {
                0 => MinecraftInstanceType::Normal,
                1 => MinecraftInstanceType::VersionIsolated,
                2 => MinecraftInstanceType::MultiMC,
                i => {
                    return Err(MyError::Other(format!(
                        "Unsupported Minecraft instance type {}",
                        i
                    )))
                }
            },
        ));
    rescan_saves(state).await
}

#[tauri::command]
pub async fn rescan_saves(state: State<'_, Mutex<AppStateInner>>) -> Result<()> {
    for item in state.lock().unwrap().instance_roots.iter_mut() {
        item.scan()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn list_instances(
    state: State<'_, Mutex<AppStateInner>>,
) -> Result<Vec<MinecraftInstance>> {
    let mut result = Vec::new();
    for item in state.lock().unwrap().instance_roots.iter() {
        println!("{:?}", result);
        result.extend(item.instances.clone());
    }
    Ok(result)
}
