use std::{fs::{self, File}, io::Write, path::PathBuf, time::Duration};

use anyhow::{Ok, Result};
use eframe::egui::ahash::HashMap;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Serialize, Deserialize};

use crate::{backup::MinecraftInstanceRoot, cloud_sync::CloudService};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub autostart: bool,
    pub cron: bool,
    pub autostart_installed: bool,
    pub cron_installed: bool,

    pub duration: Duration,
    
    pub cloud_services: HashMap<String, CloudService>,

    pub instance_roots: Vec<MinecraftInstanceRoot>,
    pub backup_root: PathBuf
}

impl Config {
    pub fn save(&self, path: PathBuf) -> Result<()> {
        if !path.exists() {
            log::warn!("Config file {} doesn't exist, creating", path.to_string_lossy())
        }   
        let mut file = File::create(path)?;
        let content = to_string_pretty(self, PrettyConfig::default())?;
        file.write(content.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}

pub fn read_config(config_path: PathBuf) -> Result<Config>{
    let content = fs::read_to_string(config_path)?;
    Ok(ron::from_str(&content)?)
}

