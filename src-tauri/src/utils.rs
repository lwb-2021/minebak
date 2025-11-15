use std::{fs, path::PathBuf, sync::LazyLock};

pub static BACKUP_HOME: LazyLock<PathBuf> = LazyLock::new(get_backup_home);

fn get_backup_home() -> PathBuf {
    if let Some(mut path) = dirs::document_dir() {
        path.push("MineBak");
        if path.exists() || fs::create_dir_all(&path).is_ok() {
            return path;
        }
    }

    if let Some(mut path) = dirs::home_dir() {
        path.push("MineBak");
        if path.exists() || fs::create_dir_all(&path).is_ok() {
            return path;
        }
    }

    let mut path = PathBuf::new();
    path.push("MineBak");
    if path.exists() || fs::create_dir_all(&path).is_ok() {
        return path;
    }

    todo!("Cannot create backup home, I don't know what to do next")
}
