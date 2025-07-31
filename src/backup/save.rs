use anyhow::{Ok, Result, anyhow};
use data_encoding::HEXLOWER;
use ring::digest::{Context, Digest, SHA256};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftSave {
    pub name: String,
    path: PathBuf,
}

impl MinecraftSave {
    pub fn search_instance(path: PathBuf) -> Result<Vec<Self>> {
        let mut result = Vec::new();
        let mut path = path.clone();
        path.push("saves");
        if !path.exists() {
            return Ok(result);
        }
        for entry in fs::read_dir(path)? {
            let child = entry?.path();
            log::debug!("Find save: {}", child.to_string_lossy());
            result.push(MinecraftSave {
                name: child
                    .file_name()
                    .ok_or(anyhow!(
                        "Error when finding save name: {}",
                        child.to_string_lossy()
                    ))?
                    .to_string_lossy()
                    .to_string(),
                path: child,
            });
        }
        Ok(result)
    }
    pub fn run_backup(&self, backup_root: PathBuf, compress_level: i32) -> Result<()> {
        let mut backup_file = backup_root;
        let parent_name = self
            .path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        backup_file.push(parent_name);    
        backup_file.push(self.name.clone());
        let mut last_hash = backup_file.clone();
        last_hash.push("last_hash.ron");
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        backup_file.push(timestamp.to_string() + ".tar");
        log::info!(
            "Starting backup: '{}' to '{}'",
            self.path.to_string_lossy(),
            backup_file.to_string_lossy()
        );

        if !last_hash.exists() {
            log::info!("First Backup");
            fs::create_dir_all(backup_file.parent().unwrap())?;
            let mut hashs: HashMap<PathBuf, String> = HashMap::new();

            let mut archive = tar::Builder::new(File::create(&backup_file)?);
            let mut header = tar::Header::new_gnu();
            for item in WalkDir::new(&self.path).sort_by_file_name() {
                let entry = item?;
                let file_path = entry.path();
                let relative = file_path.strip_prefix(&self.path)?;
                log::debug!("Adding file {}", relative.to_string_lossy());
                if file_path.is_file() {
                    let mut writer = archive.append_writer(&mut header, relative)?;
                    let hash = Self::hash_and_write(&mut File::open(file_path)?, &mut writer)?;
                    hashs.insert(file_path.to_path_buf(), HEXLOWER.encode(hash.as_ref()));
                }
            }
            fs::write(last_hash, ron::to_string(&hashs)?.as_bytes())?;
            log::debug!("Compressing");
            zstd::stream::copy_encode(
                &mut File::open(backup_file.clone())?,
                &mut File::create(backup_file.with_added_extension("zst"))?,
                compress_level,
            )?;
            fs::remove_file(backup_file)?;
            return Ok(());
        }
        todo!();
        Ok(())
    }
    fn hash_and_write<R: Read, W: Write>(source: &mut R, target: &mut W) -> Result<Digest> {
        let mut context = Context::new(&SHA256);
        let mut buf = [0; 1024];
        source.read(&mut buf)?;
        context.update(&buf);
        target.write(&buf)?;
        Ok(context.finish())
    }
    fn hash<R: Read>(source: &mut R) -> Result<Digest> {
        let mut context = Context::new(&SHA256);
        let mut buf = [0; 1024];
        source.read(&mut buf)?;
        context.update(&buf);
        Ok(context.finish())
    }
}
