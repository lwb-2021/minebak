mod instance;
mod save;

use std::{fs, thread::sleep, time::Duration};

#[allow(unused_imports)]
pub use instance::{MinecraftInstance, MinecraftInstanceRoot};
pub use save::MinecraftSave;

#[warn(unused_imports)]
use crate::config::Config;


use anyhow::{Ok, Result};

pub fn rescan_instances(config: &mut Config) -> Result<()> {
    for item in config.instance_roots.iter_mut() {
        item.rescan()?
    }
    Ok(())
}

pub fn run_backup(config: &Config) -> Result<bool> {
    let mut res = false;
    log::info!("Starting backup");
    
    let mut lock_file = config.backup_root.clone();
    lock_file.push("backup.lock");
    while lock_file.exists() {
        log::info!("Waiting for lock {:?}", lock_file);
        sleep(Duration::from_secs(10));
    }
    fs::write(&lock_file, "")?;
    
    for save in config
        .instance_roots
        .iter()
        .map(|v| v.instances.iter().map(|v| v.saves.iter()))
        .flatten()
        .flatten()
    {
        res = save.run_backup(config.backup_root.clone(), config.compress_level)? | res;
    }
    log::info!("Backup finished");

    fs::remove_file(lock_file)?;

    Ok(res)
}
