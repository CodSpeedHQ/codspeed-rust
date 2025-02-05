use crate::{measurement_mode::MeasurementMode, prelude::*};
use cargo_metadata::Metadata;
use std::path::{Path, PathBuf};

pub fn get_codspeed_target_dir(metadata: &Metadata, measurement_mode: MeasurementMode) -> PathBuf {
    metadata
        .target_directory
        .join("codspeed")
        .join(measurement_mode.to_string())
        .into()
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
