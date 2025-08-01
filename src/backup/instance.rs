use std::{fs, path::PathBuf};

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

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
            name: name,
            path: path.clone(),
            multimc,
            version_isolated,
            instances: if multimc {
                MinecraftInstance::search_multimc(path)?
            } else {
                todo!()
            },
        })
    }
    pub fn rescan(&mut self) -> Result<()> {
        if self.multimc {
            self.instances = MinecraftInstance::search_multimc(self.path.clone())?
        } else {
            todo!()
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
                log::warn!("Invaild instance: {}. Skipping", child.to_string_lossy());
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
                log::warn!("Invaild instance: {}. Skipping", child.to_string_lossy());
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
}
