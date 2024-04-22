use crate::prelude::*;
use cargo::CargoResult;
use std::path::{Path, PathBuf};

pub fn get_codspeed_target_dir(ws: &Workspace) -> PathBuf {
    ws.target_dir()
        .as_path_unlocked()
        .to_path_buf()
        .join("codspeed")
}

/// Get the packages to run benchmarks for
/// If a package name is provided, only that package is a target
/// If no package name is provided,
///     and the current directory is a package then only that package is a target
///     Otherwise all packages in the workspace are targets
pub fn get_target_packages<'a>(
    package_name: &Option<String>,
    ws: &'a Workspace<'_>,
) -> Result<Vec<&'a cargo::core::Package>> {
    let packages_to_run = if let Some(package) = package_name.as_ref() {
        let p = ws
            .members()
            .find(|m| m.manifest().name().to_string().as_str() == package)
            .ok_or(anyhow!("Package {} not found", package))?;
        vec![p]
    } else if let CargoResult::Ok(p) = ws.current() {
        vec![p]
    } else {
        ws.members().collect::<Vec<_>>()
    };
    Ok(packages_to_run)
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
