use crate::{
    backup::{self, increasement::Version},
    errors::{MyError, Result},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinecraftSave {
    path: PathBuf,
    backup_path: PathBuf,
}
// TODO

impl MinecraftSave {
    pub fn new(path: PathBuf, backup_path: PathBuf) -> Result<Self> {
        Ok(Self {
            path,
            backup_path,
            ..Default::default()
        })
    }
    pub fn backup(&self, mode: u8) -> Result<()> {
        let versions = self.load_backups()?;
        if versions.is_empty() {
            Version::create_full(self.path.clone(), self.backup_path.clone())?;
            return Ok(());
        }
        match mode {
            0 => {
                // File
                Version::create_version(
                    versions.last().unwrap(),
                    self.path.clone(),
                    self.backup_path.clone(),
                )?;
                Ok(())
            }
            1 => {
                todo!()
            }
            i => Err(MyError::Other(format!("Unexpected backup mode"))),
        }
    }
    fn load_backups(&self) -> Result<Vec<Version>> {
        if !self.backup_path.exists() {
            return Ok(vec![]);
        }
        let mut versions = Vec::new();
        for backup in fs::read_dir(&self.backup_path)? {
            let backup = backup?;
            let path = backup.path();
            if path.is_dir() {
                versions.push(Version::read(&path)?);
            } else {
                versions.push(Version::read_compressed(&path)?);
            }
        }
        Ok(versions)
    }
}
