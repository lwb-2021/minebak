use std::{
    collections::HashMap,
    env,
    fs::{self},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

use anyhow::{Ok, Result, anyhow, bail};
use rustydav::client::Client;
use serde::{Deserialize, Serialize};

use crate::{config::Config, utils::compare_hash};

pub fn run_sync(config: &Config) -> Result<()> {
    log::info!("Sync started");

    let mut lock_file = config.backup_root.clone();
    lock_file.push("sync.lock");
    while lock_file.exists() {
        log::info!("Waiting for lock {:?}", lock_file);
        sleep(Duration::from_secs(10));
    }
    fs::write(&lock_file, "")?;

    let mut res = Ok(());
    if !config.cloud_services.is_empty() {
        for (name, service) in &config.cloud_services {
            log::info!("Sync started to {}", name);
            let result = service.sync(
                &config.backup_root,
                "minebak/backup".to_string(),
                true,
                false,
            );
            if result.is_err() {
                log::error!("Sync failed to {}: {}", name, result.as_ref().unwrap_err());
                notifica::notify(
                    "MineBak: 同步失败",
                    &format!("同步到远程 {} 失败", name.split("@").last().unwrap()),
                )?;
                res = result;
            } else {
                log::info!("Sync finished to {}", name);
            }
        }
    }
    log::info!("Sync finished");

    fs::remove_file(lock_file)?;

    res
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CloudService {
    RClone {
        remote: String,
    },
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
    pub fn push_file(&self, file: &Path, remote_file: String) -> Result<()> {
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
                    &(endpoint.clone() + &remote_file),
                );
                client
                    .as_ref()
                    .unwrap()
                    .put(fs::read(file)?, &(endpoint.clone() + &remote_file))?
                    .error_for_status()?;
                Ok(())
            }
            Self::RClone { remote } => {
                std::process::Command::new("rclone")
                    .arg("copy")
                    .arg(file.as_os_str())
                    .arg(format!("{}:{}", remote, remote_file))
                    .exec()
                    .downcast()?;
                Ok(())
            }
            #[allow(unreachable_patterns)]
            s => panic!("Unimplemented sync method {:?}", s),
        }
    }
    pub fn push_hash(&self, content: &HashMap<PathBuf, String>, remote_root: String) -> Result<()> {
        match &self {
            Self::WebDAV {
                endpoint,
                username: _,
                password: _,
                client,
                init: _,
            } => {
                client
                    .as_ref()
                    .unwrap()
                    .put(
                        ron::to_string(&content)?,
                        &format!("{}{}/hash.ron", endpoint, remote_root),
                    )?
                    .error_for_status()?;
                Ok(())
            }
            Self::RClone { remote } => {
                let mut tmp = env::temp_dir();
                tmp.push("hash.tmp.ron");
                fs::write(&tmp, ron::to_string(content)?)?;
                std::process::Command::new("rclone")
                    .arg("copy")
                    .arg(tmp.to_string_lossy().to_string())
                    .arg(format!("{}:{}/hash.ron", remote, remote_root))
                    .exec()
                    .downcast()?;
                fs::remove_file(tmp)?;
                Ok(())
            }
            #[allow(unreachable_patterns)]
            s => panic!("Unimplemented sync method {:?}", s),
        }
    }

    pub fn pull_file(&self, remote_file: String, to: &Path) -> Result<()> {
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
                    &(endpoint.clone() + &remote_file),
                );
                if !to
                    .parent()
                    .map(|x| x.to_path_buf())
                    .unwrap_or_default()
                    .exists()
                {
                    fs::create_dir_all(to.parent().map(|x| x.to_path_buf()).unwrap_or_default())?;
                }
                fs::write(
                    to,
                    client
                        .as_ref()
                        .unwrap()
                        .get(&(endpoint.clone() + &remote_file))?
                        .error_for_status()?
                        .bytes()?,
                )?;
                Ok(())
            }
            Self::RClone { remote } => {
                std::process::Command::new("rclone")
                    .arg("copy")
                    .arg(format!("{}:{}", remote, remote_file))
                    .arg(to.as_os_str())
                    .exec()
                    .downcast()?;
                Ok(())
            }
            #[allow(unreachable_patterns)]
            s => panic!("Unimplemented sync method {:?}", s),
        }
    }

    pub fn delete_file(&self, remote_file: String) -> Result<()> {
        match &self {
            Self::WebDAV {
                endpoint,
                username: _,
                password: _,
                client,
                init: _,
            } => {
                log::debug!("Deleting {}", remote_file);
                client
                    .as_ref()
                    .unwrap()
                    .delete(&(endpoint.clone() + &remote_file))?
                    .error_for_status()?;
                Ok(())
            }
            Self::RClone { remote } => {
                std::process::Command::new("rclone")
                    .arg("delete")
                    .arg(format!("{}:{}", remote, remote_file))
                    .exec()
                    .downcast()?;
                Ok(())
            }
            #[allow(unreachable_patterns)]
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
        self.pull_file(format!("{}/hash.ron", remote_root), &hash_tmp)?;
        let mut hashs = ron::from_str(&fs::read_to_string(&hash_tmp)?)?;
        for ((relative, conflict), (old_hash, new_hash)) in
            compare_hash(folder.to_path_buf(), &hashs)?
        {
            let mut item = folder.to_path_buf();
            item.extend(&relative);
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
        self.pull_file(format!("{}/hash.ron", remote_root), &hash_tmp)?;
        let hashs = ron::from_str(&fs::read_to_string(&hash_tmp)?)?;
        if !skip_conflict {
            for ((relative, conflict), _) in compare_hash(folder.to_path_buf(), &hashs)? {
                let mut item = folder.to_path_buf();
                item.extend(&relative);
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
        for (relative, hash) in hashs {
            let mut path = folder.to_path_buf();
            path.extend(&relative);
            if !path.exists() {
                self.pull_file(format!("{}/{}", remote_root, hash), &path)?;
            }
        }
        fs::remove_file(&hash_tmp)?;
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
                    *init = true;
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
                            &(endpoint.to_string() + "minebak/backup/hash.ron"),
                        )?
                        .error_for_status()?;
                }
            }
            s => todo!("{:?} sync not implemented", s),
        }
        Ok(())
    }
}
