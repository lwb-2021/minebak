use std::{
    collections::HashMap,
    env,
    fs::{self},
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow, bail};
use rustydav::{client::Client, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{config::Config, utils::compare_hash};

pub fn run_sync(config: &Config) -> Result<()> {
    if !config.cloud_services.is_empty() {
        for service in config.cloud_services.values() {
            service.sync(&config.backup_root, "minebak".to_string(), false, false)?;
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CloudService {
    RClone,
    WebDAV {
        endpoint: String,
        username: String,
        password: String,
        #[serde(skip_serializing, skip_deserializing)]
        client: Option<Client>,
    },
}

impl CloudService {
    pub fn push_file(&self, file: &Path, remote: String) -> Result<Response> {
        match &self {
            Self::WebDAV {
                endpoint: _,
                username: _,
                password: _,
                client,
            } => Ok(client
                .as_ref()
                .unwrap()
                .put(fs::read(file)?, &remote)?
                .error_for_status()?),
            s => panic!("Unimplemented sync method {:?}", s),
        }
    }
    pub fn push_hash(
        &self,
        content: &HashMap<PathBuf, String>,
        remote_root: String,
    ) -> Result<Response> {
        match &self {
            Self::WebDAV {
                endpoint: _,
                username: _,
                password: _,
                client,
            } => Ok(client
                .as_ref()
                .unwrap()
                .put(
                    ron::to_string(&content)?,
                    &format!("{}/hash.ron", remote_root),
                )?
                .error_for_status()?),
            s => panic!("Unimplemented sync method {:?}", s),
        }
    }

    pub fn pull_file(&self, remote: String, to: &Path) -> Result<()> {
        match &self {
            Self::WebDAV {
                endpoint: _,
                username: _,
                password: _,
                client,
            } => {
                fs::write(
                    to,
                    client
                        .as_ref()
                        .unwrap()
                        .get(&remote)?
                        .error_for_status()?
                        .bytes()?,
                )?;
            }
            s => panic!("Unimplemented sync method {:?}", s),
        }
        todo!()
    }

    pub fn delete_file(&self, remote: String) -> Result<Response> {
        match &self {
            Self::WebDAV {
                endpoint: _,
                username: _,
                password: _,
                client,
            } => Ok(client
                .as_ref()
                .unwrap()
                .delete(&remote)?
                .error_for_status()?),
            s => panic!("Unimplemented sync method {:?}", s),
        }
    }

    pub fn push(
        &self,
        folder: &Path,
        remote_root: String,

        force: bool,
        skip_conflict: bool,
    ) -> Result<()> {
        let hash_tmp = env::temp_dir()
            .with_file_name("hash.pull.ron")
            .to_path_buf();
        self.pull_file("minebak/hash.ron".to_string(), &hash_tmp)?;
        let mut hashs = ron::from_str(&fs::read_to_string(hash_tmp)?)?;
        for ((item, conflict), (old_hash, new_hash)) in compare_hash(folder.to_path_buf(), &hashs)?
        {
            if conflict && skip_conflict {
                continue;
            }
            if conflict && !force {
                notifica::notify("文件冲突，停止同步", item.to_str().unwrap()).unwrap();
                bail!(anyhow!("File conflicts: {}", item.to_str().unwrap()))
            }
            if let Some(h) = old_hash {
                self.delete_file(h)?;
                hashs.remove(&item);
            }

            self.push_file(&item, format!("{}/{}", remote_root, &new_hash))?;
            hashs.insert(item.to_path_buf(), new_hash);
        }
        self.push_hash(&hashs, remote_root)?;
        Ok(())
    }
    pub fn pull(
        &self,
        folder: &Path,
        remote_root: String,
        force: bool,
        skip_conflict: bool,
    ) -> Result<()> {
        let hash_tmp = env::temp_dir()
            .with_file_name("hash.pull.ron")
            .to_path_buf();
        self.pull_file("minebak/hash.ron".to_string(), &hash_tmp)?;
        let hashs = ron::from_str(&fs::read_to_string(hash_tmp)?)?;
        if !skip_conflict {
            for ((item, conflict), _) in compare_hash(folder.to_path_buf(), &hashs)? {
                if !conflict {
                    continue;
                }
                if !force {
                    notifica::notify("文件冲突，停止同步", item.to_str().unwrap()).unwrap();
                    bail!(anyhow!("File conflicts: {}", item.to_str().unwrap()));
                }
                fs::remove_file(item)?;
            }
        }
        for (path, hash) in hashs {
            self.pull_file(format!("{}/{}", remote_root, hash), &path)?;
        }

        Ok(())
    }

    pub fn sync(
        &self,
        folder: &Path,
        remote_root: String,
        force_push: bool,
        force_pull: bool,
    ) -> Result<()> {
        self.pull(folder, remote_root.clone(), force_pull, false)?;
        self.push(folder, remote_root, force_push, false)?;
        Ok(())
    }

    pub fn open_connection(&mut self) -> Result<()> {
        match self {
            Self::WebDAV {
                endpoint: _,
                username,
                password,
                client,
            } => {
                if client.is_none() {
                    *client = Some(Client::init(&username, &password));
                }
            }
            s => todo!("{:?} sync not implemented", s),
        }
        Ok(())
    }
}
