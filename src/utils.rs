use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use anyhow::Result;
use data_encoding::HEXLOWER;
use ring::digest::{Context, Digest, SHA256};

pub fn hash<R: Read>(source: &mut R) -> Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buf = [0; 1024];
    loop {
        let count = source.read(&mut buf)?;
        if count == 0 {
            break;
        }
        context.update(&buf[..count]);
    }

    Ok(context.finish())
}

pub fn generate_sum_for_folder(root: PathBuf) -> Result<HashMap<PathBuf, String>> {
    let mut res = HashMap::new();
    for item in walkdir::WalkDir::new(&root) {
        let entry = item?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let digest = hash(&mut File::open(path)?)?;
        res.insert(
            path.strip_prefix(&root)?.to_path_buf(),
            HEXLOWER.encode(digest.as_ref()),
        );
    }
    Ok(res)
}

pub fn compare_hash(root: PathBuf, hashs: &HashMap<PathBuf, String>) -> Result<HashMap<(PathBuf, bool), (Option<String>, String)>> {
    let mut res = HashMap::new();
    for item in walkdir::WalkDir::new(&root) {
        let entry = item?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let relative = path.strip_prefix(&root)?;
        let digest = HEXLOWER.encode(hash(&mut File::open(path)?)?.as_ref());
        if hashs.get(relative) != Some(&digest) {
            res.insert((path.to_path_buf(), hashs.contains_key(relative)), (hashs.get(relative).cloned(), digest));
        }
    }
    Ok(res)
}
