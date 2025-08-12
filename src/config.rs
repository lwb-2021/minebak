use std::{
    env, fs::{self, File}, io::Write, path::PathBuf, time::Duration
};

use anyhow::{Ok, Result};
use eframe::egui::ahash::HashMap;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Deserialize, Serialize};

use crate::{backup::MinecraftInstanceRoot, cloud_sync::CloudService};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    
    pub autostart: bool,
    pub cron: bool,
    pub autostart_installed: bool,
    pub cron_installed: bool,

    #[serde(default = "default_duration")]
    pub duration: Duration,


    pub cloud_services: HashMap<String, CloudService>,

    pub instance_roots: Vec<MinecraftInstanceRoot>,
    #[serde(default = "default_backup_root")]
    pub backup_root: PathBuf,
}

impl Config {
    pub fn save(&self, path: PathBuf) -> Result<()> {
        if !path.exists() {
            log::warn!(
                "Config file {} doesn't exist, creating",
                path.to_string_lossy()
            )
        }
        let mut file = File::create(path)?;
        let content = to_string_pretty(self, PrettyConfig::default())?;
        file.write(content.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}

pub fn read_config(config_path: PathBuf) -> Result<Config> {
    let content = fs::read_to_string(config_path)?;
    Ok(ron::from_str(&content)?)
}

pub const fn default_duration() -> Duration {
    Duration::from_hours(1)
}

pub fn default_backup_root() -> PathBuf {
    let mut res = env::home_dir().expect("Cannot get home dir, please set --backup-root").to_path_buf();
    res.push(".minebak");
    res.push("backup");
    res
}
