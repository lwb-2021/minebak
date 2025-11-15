use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::errors::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum BackupType {
    #[default]
    Full,
    FileDelta,
    DataDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Version {
    #[serde(skip)]
    path: PathBuf,

    name: String,
    target: PathBuf,
    backup_type: BackupType,
    file_hash: HashMap<String, String>,
}
impl Version {
    pub fn read(path: &Path) -> Result<Self> {
        let mut path: PathBuf = path.to_path_buf();
        path.push("metadata.json");
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }
    pub fn read_compressed(path: &Path) -> Result<Self> {
        todo!()
    }
    pub fn create_full(src: PathBuf, dst: PathBuf) -> Result<Self> {
        let mut file_hash = HashMap::new();
        let mut dst = dst.clone();
        dst.push("0");
        for entry in WalkDir::new(&src).min_depth(1) {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let relative = path.strip_prefix(&src).unwrap().to_path_buf();
            let mut target = dst.clone();
            target.push(&relative);

            file_hash.insert(
                relative.to_string_lossy().to_string(),
                sha256::try_digest(&path)?,
            );
            log::debug!("Hash info generated for {:?}", relative);
            log::debug!("Copying {:?} to {:?}", relative, target);

            fs::create_dir_all(target.parent().as_ref().unwrap())?;
            fs::copy(path, target)?;
        }

        let res = Version {
            path: dst,
            target: src,
            backup_type: BackupType::Full,
            file_hash,
            ..Default::default()
        };
        res.write_meta()?;
        Ok(res)
    }
    pub fn create_version(prev: &Self, src: PathBuf, dst: PathBuf) -> Result<Self> {
        todo!()
    }
    pub fn write_meta(&self) -> Result<()> {
        let mut meta_file = self.path.clone();
        meta_file.push("metadata.json");
        serde_json::to_writer(fs::File::create_new(meta_file)?, &self)?;
        Ok(())
    }
    pub fn restore(&self) -> Result<()> {
        todo!()
    }
    pub fn merge(self, prev: &Self) -> Result<()> {
        todo!()
    }
}
