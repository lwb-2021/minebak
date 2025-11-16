use crate::AppStateInner;
use crate::backup::MinecraftInstance;
use crate::backup::MinecraftInstanceRoot;
use crate::backup::MinecraftInstanceType;
use crate::errors::MyError;
use crate::errors::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

impl Serialize for MyError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let err = self.to_string();
        log::error!("{}", err);
        log::error!("Sending error to frontend");
        serializer.serialize_str(&err)
    }
}
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
                    )));
                }
            },
        ));
    rescan_saves(state).await
}

#[tauri::command]
pub async fn rescan_saves(state: State<'_, Mutex<AppStateInner>>) -> Result<()> {
    let instance_roots = &mut state.lock().unwrap().instance_roots;
    for index in 0..instance_roots.len() {
        if let Err(err) = instance_roots.get_mut(index).unwrap().scan() {
            instance_roots.remove(index);
            return Err(err);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn list_instances(
    state: State<'_, Mutex<AppStateInner>>,
) -> Result<HashMap<String, MinecraftInstance>> {
    let mut result = HashMap::new();
    for item in state.lock().unwrap().instance_roots.iter() {
        result.extend(item.instances.clone());
    }
    Ok(result)
}

#[tauri::command]
pub async fn run_instance_backup(
    name: String,
    state: State<'_, Mutex<AppStateInner>>,
) -> Result<()> {
    let instance_roots = &state.lock().unwrap().instance_roots;
    for instance_root in instance_roots.iter() {
        if let Some(instance) = &instance_root.instances.get(&name) {
            return instance.backup();
        }
    }
    Err(MyError::Other(format!("Invaild name: {}", name)))
}
