use std::{fs, path::PathBuf};

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backup::MinecraftSave;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftInstanceRoot {
    pub name: String,
    pub path: PathBuf,
    multimc: bool,
    version_isolated: bool,
    pub instances: Vec<MinecraftInstance>,
}

impl MinecraftInstanceRoot {
    pub fn new(name: String, path: PathBuf, multimc: bool, version_isolated: bool) -> Result<Self> {
        Ok(Self {
            name: name.clone(),
            path: path.clone(),
            multimc,
            version_isolated,
            instances: if multimc {
                MinecraftInstance::search_multimc(path)?
            } else {
                if version_isolated {
                    MinecraftInstance::search_version_isolated(path)?
                } else {
                    vec![MinecraftInstance::search_normal(path, name)?]
                }
            },
        })
    }
    pub fn rescan(&mut self) -> Result<()> {
        if self.multimc {
            self.instances = MinecraftInstance::search_multimc(self.path.clone())?
        } else {
            if self.version_isolated {
                self.instances = MinecraftInstance::search_version_isolated(self.path.clone())?;
            } else {
                self.instances = vec![MinecraftInstance::search_normal(
                    self.path.clone(),
                    self.name.clone(),
                )?]
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftInstance {
    pub name: String,
    dot_minecraft: PathBuf,
    pub saves: Vec<MinecraftSave>,
}

impl MinecraftInstance {
    pub fn search_multimc(path: PathBuf) -> Result<Vec<Self>> {
        let mut result: Vec<Self> = Vec::new();

        for entry in fs::read_dir(path)? {
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

            result.push(Self {
                name: name.clone(),
                dot_minecraft: child.clone(),
                saves: MinecraftSave::search_instance(child, name)?,
            });
        }
        Ok(result)
    }
    pub fn search_normal(path: PathBuf, name: String) -> Result<Self> {
        Ok(Self {
            name: name.clone(),
            dot_minecraft: path.clone(),
            saves: MinecraftSave::search_instance(path, name)?,
        })
    }
    pub fn search_version_isolated(mut path: PathBuf) -> Result<Vec<Self>> {
        let mut result: Vec<Self> = Vec::new();
        path.push("versions");
        for entry in fs::read_dir(&path)? {
            let mut child = entry?.path();
            if child.is_file() {
                continue;
            }
            let name = child.file_name().unwrap().to_os_string();
            child.push(name);
            child.add_extension("json");
            if !child.exists() {
                child.pop();
                log::warn!("Invaild instance: {:?}. Skipping", child);
                continue;
            }

            let info = fs::read_to_string(&child)?;
            let name = serde_json::from_str::<Value>(&info)?["id"]
                .as_str()
                .map(|s| s.to_string());
            child.pop();

            if name.is_none() {
                log::warn!("Invaild instance: {:?}. Skipping", child);
                continue;
            }
            let name = name.unwrap();

            log::debug!("Instance detected: {}", name);

            result.push(Self {
                name: name.clone(),
                dot_minecraft: child.clone(),
                saves: MinecraftSave::search_instance(child, name)?,
            });
        }
        Ok(result)
    }
}
