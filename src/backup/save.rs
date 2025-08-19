use std::{
    collections::HashMap, fs::{self, File}, io::{Read, Write}, path::PathBuf, thread, time::{Duration, SystemTime, UNIX_EPOCH}
};

use anyhow::{Ok, Result, anyhow};
use data_encoding::HEXLOWER;
use ring::digest::{Context, Digest, SHA256};
use serde::{Deserialize, Serialize};
use tar::Archive;
use walkdir::WalkDir;

use crate::utils::hash;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MinecraftSave {
    pub instance_name: String,
    pub name: String,
    pub image: Option<PathBuf>,
    path: PathBuf,
    pub description: String,
}

impl MinecraftSave {
    pub fn search_instance(path: PathBuf, instance_name: String) -> Result<Vec<Self>> {
        let mut result = Vec::new();
        let mut path = path.clone();
        path.push("saves");
        if !path.exists() {
            return Ok(result);
        }
        for entry in fs::read_dir(path)? {
            let child = entry?.path();
            let mut icon = child.clone();
            let mut image = None;
            icon.push("icon.png");
            if icon.exists() {
                image = Some(icon);
            }
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
                image,
                description: String::new(),
                path: child,
                instance_name: instance_name.clone(),
            });
        }
        Ok(result)
    }
    pub fn run_backup(&self, backup_root: PathBuf, compress_level: i32) -> Result<bool> {
        let mut backup_file = backup_root;
        backup_file.push(self.instance_name.clone());
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
            for item in WalkDir::new(&self.path).sort_by_file_name() {
                let entry = item?;
                let file_path = entry.path();
                let relative = file_path.strip_prefix(&self.path)?;
                log::debug!("Adding file {}", relative.to_string_lossy());
                if file_path.is_file() {
                    let mut header = tar::Header::new_gnu();
                    header.set_mode(0o777);
                    header.set_size(fs::metadata(file_path)?.len());
                    header.set_mtime(fs::metadata(file_path)?.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as u64);
                    header.set_cksum();
                    let mut writer = archive.append_writer(&mut header, relative)?;
                    let hash = Self::hash_and_write(&mut File::open(file_path)?, &mut writer)?;
                    hashs.insert(relative.to_path_buf(), HEXLOWER.encode(hash.as_ref()));
                }
            }
            
            fs::write(last_hash, ron::to_string(&hashs)?.as_bytes())?;
            log::info!("Compressing");
            zstd::stream::copy_encode(
                &mut File::open(&backup_file)?,
                &mut File::create(backup_file.with_added_extension("zst"))?,
                compress_level,
            )?;
            fs::remove_file(backup_file)?;
            return Ok(true);
        }
        let mut hashs: HashMap<PathBuf, String> = ron::from_str(&fs::read_to_string(&last_hash)?)?;
        let mut archive = tar::Builder::new(File::create(&backup_file)?);
        let mut changed = false;
        for item in WalkDir::new(&self.path).sort_by_file_name() {
            let entry = item?;
            let file_path = entry.path();
            let relative = file_path.strip_prefix(&self.path)?;
            if file_path.is_file() {
                let hash = HEXLOWER.encode(hash(&mut File::open(file_path)?)?.as_ref());
                if hashs.get(relative) == Some(&hash) {
                    continue;
                }
                changed = true;
                archive.append_file(relative, &mut File::open(file_path)?)?;
                hashs.insert(relative.to_path_buf(), hash);
                
            }
        }
        if changed {
            log::info!("File changed, compressing");
            zstd::stream::copy_encode(
                &mut File::open(&backup_file)?,
                &mut File::create(backup_file.with_added_extension("zst"))?,
                compress_level,
            )?;
            fs::write(last_hash, ron::to_string(&hashs)?.as_bytes())?;
        } else {
            log::info!("File not changed");
        }
        fs::remove_file(&backup_file)?;
        Ok(changed)
    }
    pub fn list_backups(&self, backup_root: PathBuf) -> Result<Vec<String>> {
        let mut backup_folder = backup_root;
        backup_folder.push(self.instance_name.clone());
        backup_folder.push(self.name.clone());
        let mut res = Vec::new();
        for entry in fs::read_dir(backup_folder)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains(".tar.zst") {
                    log::info!("Backup dectected: {}", name);
                    res.push(name.replace(".tar.zst", ""));
                }
            }
        }
        Ok(res)
    }
    pub fn recover(&self, backup_root: PathBuf, timestamp: String) -> Result<()> {
        let mut backup_folder = backup_root.clone();
        backup_folder.push(self.instance_name.clone());
        backup_folder.push(self.name.clone());

        let mut backups = self.list_backups(backup_root)?;
        backups.sort();

        let mut recover_root = self.path.clone();
        recover_root.pop();
        recover_root.push(
            self.name.clone()
                + "-recover-"
                + &SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_millis()
                    .to_string(),
        );

        log::info!("Recovery started");
        for item in backups {
            if item == timestamp {
                break;
            }

            log::info!("Recovering: {} to {}", item, recover_root.to_string_lossy());
            backup_folder.push(timestamp.clone() + ".tar.zst");

            let file = File::open(backup_folder.clone())?;
            let decoder = zstd::Decoder::new(file)?;
            let mut archive = Archive::new(decoder);
            archive.set_overwrite(true);
            archive.unpack(recover_root.clone())?;
            backup_folder.pop();
        }
        log::info!("Recovery finished");
        Ok(())
    }
    fn hash_and_write<R: Read, W: Write>(source: &mut R, target: &mut W) -> Result<Digest> {
        let mut context = Context::new(&SHA256);
        let mut buf = [0; 1024];

        loop {
            let count = source.read(&mut buf)?;
            if count == 0 {
                break;
            }
            context.update(&buf[..count]);
            target.write(&buf[..count])?;
        }

        Ok(context.finish())
    }
    
}
