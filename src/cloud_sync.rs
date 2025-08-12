use std::{
    collections::HashMap,
    env,
    fs::{self},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Ok, Result};
use rustydav::{client::Client, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{config::Config, utils::compare_hash};

pub fn run_sync(config: &Config) -> Result<()> {
    log::info!("Sync started");
    if !config.cloud_services.is_empty() {
        for (name, service) in &config.cloud_services {
            log::info!("Sync started to {}", name);
            service.sync(&config.backup_root, "minebak".to_string(), false, false)?;
            log::info!("Sync finished to {}", name);
        }
    }
    log::info!("Sync finished");
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CloudService {
    RClone,
    WebDAV {
        endpoint: String,
        username: String,
        password: String,
        init: bool,
        #[serde(skip_serializing, skip_deserializing)]
        client: Option<Client>,
    },
}

impl CloudService {
    pub fn push_file(&self, file: &Path, remote: String) -> Result<Response> {
        match &self {
            Self::WebDAV {
                endpoint,
                username: _,
                password: _,
                client,
                init: _,
            } => {
                log::debug!(
                    "Pushing {} to {}",
                    file.to_str().unwrap(),
                    &(endpoint.clone() + &remote),
                );
                Ok(client
                    .as_ref()
                    .unwrap()
                    .put(fs::read(file)?, &(endpoint.clone() + &remote))?
                    .error_for_status()?)
            }
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
                endpoint,
                username: _,
                password: _,
                client,
                init: _,
            } => Ok(client
                .as_ref()
                .unwrap()
                .put(
                    ron::to_string(&content)?,
                    &format!("{}{}/hash.ron", endpoint, remote_root),
                )?
                .error_for_status()?),
            s => panic!("Unimplemented sync method {:?}", s),
        }
    }

    pub fn pull_file(&self, remote: String, to: &Path) -> Result<()> {
        match &self {
            Self::WebDAV {
                endpoint,
                username: _,
                password: _,
                client,
                init: _,
            } => {
                log::debug!(
                    "Pulling {} from {}",
                    to.to_str().unwrap(),
                    &(endpoint.clone() + &remote),
                );
                fs::write(
                    to,
                    client
                        .as_ref()
                        .unwrap()
                        .get(&(endpoint.clone() + &remote))?
                        .error_for_status()?
                        .bytes()?,
                )?;
            }
            s => panic!("Unimplemented sync method {:?}", s),
        }
        Ok(())
    }

    pub fn delete_file(&self, remote: String) -> Result<Response> {
        match &self {
            Self::WebDAV {
                endpoint,
                username: _,
                password: _,
                client,
                init: _,
            } => {
                log::debug!("Deleting {}", remote);
                Ok(client
                    .as_ref()
                    .unwrap()
                    .delete(&(endpoint.clone() + &remote))?
                    .error_for_status()?)
            }
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
        let mut hash_tmp = env::temp_dir().to_path_buf();
        hash_tmp.push("hash.pull.ron");
        self.pull_file("minebak/hash.ron".to_string(), &hash_tmp)?;
        let mut hashs = ron::from_str(&fs::read_to_string(&hash_tmp)?)?;
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
        fs::remove_file(hash_tmp)?;
        Ok(())
    }
    pub fn pull(
        &self,
        folder: &Path,
        remote_root: String,
        force: bool,
        skip_conflict: bool,
    ) -> Result<()> {
        let mut hash_tmp = env::temp_dir().to_path_buf();
        hash_tmp.push("hash.pull.ron");
        self.pull_file("minebak/hash.ron".to_string(), &hash_tmp)?;
        let hashs = ron::from_str(&fs::read_to_string(&hash_tmp)?)?;
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
        fs::remove_dir(&hash_tmp)?;
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
                endpoint,
                username,
                password,
                client,
                init,
            } => {
                if client.is_none() {
                    *client = Some(Client::init(&username, &password));
                }
                if !*init {
                    log::info!("Initalizating WebDAV");
                    client
                        .as_ref()
                        .unwrap()
                        .mkcol(&(endpoint.to_string() + "minebak"))?
                        .error_for_status()?;
                    let hashs_empty: HashMap<PathBuf, String> = HashMap::new();
                    client
                        .as_ref()
                        .unwrap()
                        .put(
                            ron::to_string(&hashs_empty)?,
                            &(endpoint.to_string() + "minebak/hash.ron"),
                        )?
                        .error_for_status()?;
                    *init = true;
                }
            }
            s => todo!("{:?} sync not implemented", s),
        }
        Ok(())
    }
}
