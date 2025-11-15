use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    backup::{
        base::{MinecraftInstanceMetadata, MinecraftInstanceType},
        save::MinecraftSave,
    },
    errors::{MyError, Result},
    utils::{self, BACKUP_HOME},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftInstanceRoot {
    root: PathBuf,
    instances_type: MinecraftInstanceType,
    pub(super) instances: HashMap<String, MinecraftInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftInstance {
    path: PathBuf,

    metadata: MinecraftInstanceMetadata,

    pub(super) saves: HashMap<String, MinecraftSave>,

    instance_type: MinecraftInstanceType,
}

impl MinecraftInstanceRoot {
    pub fn new(root: PathBuf, instances_type: MinecraftInstanceType) -> Self {
        Self {
            root: root,
            instances_type: instances_type,
            instances: HashMap::new(),
        }
    }
    pub fn scan(&mut self) -> Result<()> {
        self.instances.extend(match &self.instances_type {
            &MinecraftInstanceType::MultiMC => Self::scan_multimc(self.root.clone()),
            _ => Err(MyError::Other(format!(
                "Unsupported instance type {:?}",
                self.instances_type
            ))),
        }?);
        self.scan_saves()?;
        Ok(())
    }
    fn scan_multimc(root: PathBuf) -> Result<HashMap<String, MinecraftInstance>> {
        let mut root = root;
        let mut result = HashMap::new();
        if !root.ends_with("instances") {
            root.push("instances");
        }
        for entry in fs::read_dir(root)? {
            let mut child = entry?.path();
            if child.is_file() {
                continue;
            }
            child.push("instance.cfg");
            if !child.exists() {
                child.pop();
                log::warn!("Invaild instance: {:?}. Skipping", child);
                continue;
            }

            let info = fs::read_to_string(&child)?;
            let mut name = String::new();
            for line in info.split("\n") {
                if line.contains("name") {
                    name = line.replace("name=", "")
                }
            }
            child.pop();
            child.push(".minecraft");
            if !child.exists() {
                child.pop();
                child.push("minecraft");
            }
            if !child.exists() {
                log::warn!("Invaild instance: {:?}. Skipping", child);
            }
            log::debug!("Instance detected: {}", name);

            result.insert(
                name,
                MinecraftInstance {
                    path: child.clone(),
                    instance_type: MinecraftInstanceType::MultiMC,
                    metadata: MinecraftInstanceMetadata::parse(
                        child,
                        MinecraftInstanceType::MultiMC,
                    )?,
                    saves: HashMap::new(),
                },
            );
        }
        Ok(result)
    }
    pub fn scan_saves(&mut self) -> Result<()> {
        for (_, instance) in self.instances.iter_mut() {
            instance.scan_saves()?;
        }
        Ok(())
    }
}
impl MinecraftInstance {
    pub fn scan_saves(&mut self) -> Result<()> {
        let mut root = self.path.clone();
        root.push("saves");

        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let save_path = entry.path();
            let name = entry.file_name();
            let mut backup_path = BACKUP_HOME.clone();
            backup_path.push(&name);
            if !self.saves.contains_key(name.to_str().unwrap()) {
                self.saves.insert(
                    name.to_string_lossy().to_string(),
                    MinecraftSave::new(save_path, backup_path)?,
                );
            }
        }
        Ok(())
    }
    pub fn backup(&self) -> Result<()> {
        for save in self.saves.values() {
            save.backup(0)?;
        }
        Ok(())
    }
}
