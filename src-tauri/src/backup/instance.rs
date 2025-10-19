use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::{
    backup::{
        base::{MinecraftInstanceMetadata, MinecraftInstanceType},
        save::MinecraftSave,
    },
    errors::{MyError, Result},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftInstanceRoot {
    root: PathBuf,
    instances_type: MinecraftInstanceType,
    pub(super) instances: Vec<MinecraftInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftInstance {
    name: String,
    path: PathBuf,

    metadata: MinecraftInstanceMetadata,

    pub(super) saves: Vec<MinecraftSave>,

    instance_type: MinecraftInstanceType,
}

impl MinecraftInstanceRoot {
    pub fn new(root: PathBuf, instances_type: MinecraftInstanceType) -> Self {
        Self {
            root: root,
            instances_type: instances_type,
            instances: Vec::new(),
        }
    }
    pub fn scan(&mut self) -> Result<()> {
        self.instances.append(&mut match &self.instances_type {
            &MinecraftInstanceType::MultiMC => Self::scan_multimc(self.root.clone()),
            _ => Err(MyError::Other(format!(
                "Unsupported instance type {:?}",
                self.instances_type
            ))),
        }?);
        Ok(())
    }
    fn scan_multimc(root: PathBuf) -> Result<Vec<MinecraftInstance>> {
        let mut root = root;
        let mut result = Vec::new();
        root.push("instances");
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

            result.push(MinecraftInstance {
                name,
                path: child.clone(),
                instance_type: MinecraftInstanceType::MultiMC,
                metadata: MinecraftInstanceMetadata::parse(child, MinecraftInstanceType::MultiMC)?,
                saves: Vec::new(),
            });
        }
        Ok(result)
    }
}
