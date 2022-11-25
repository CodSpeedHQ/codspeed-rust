use crate::prelude::*;
use std::path::{Path, PathBuf};

pub fn get_codspeed_dir(ws: &Workspace) -> PathBuf {
    ws.target_dir()
        .as_path_unlocked()
        .to_path_buf()
        .join("codspeed")
}

pub fn read_dir_recursive<P>(dir: P) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let mut out = vec![];
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.extend(read_dir_recursive(&path)?);
        } else {
            out.push(path);
        }
    }
    Ok(out)
}

pub fn clear_dir<P>(dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            std::fs::remove_file(path)?;
        } else {
            std::fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}
