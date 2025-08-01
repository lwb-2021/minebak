mod instance;
mod save;

#[allow(unused_imports)]
pub use instance::{MinecraftInstance, MinecraftInstanceRoot};
pub use save::MinecraftSave;

#[warn(unused_imports)]
use crate::config::Config;


use anyhow::Result;

pub fn rescan_instances(config: &mut Config) -> Result<()> {
    for item in config.instance_roots.iter_mut() {
        item.rescan()?
    }
    Ok(())
}

pub fn run_backup(config: &Config) -> Result<()> {
    log::info!("Starting backup");
    for save in config
        .instance_roots
        .iter()
        .map(|v| v.instances.iter().map(|v| v.saves.iter()))
        .flatten()
        .flatten()
    {
        save.run_backup(config.backup_root.clone(), 3)?;
    }
    log::info!("Backup finished");
    Ok(())
}
