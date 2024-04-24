use crate::prelude::*;
use std::path::{Path, PathBuf};

pub fn get_codspeed_target_dir(ws: &Workspace) -> PathBuf {
    ws.target_dir()
        .as_path_unlocked()
        .to_path_buf()
        .join("codspeed")
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

pub mod style {
    use anstyle::{AnsiColor, Color, Style};

    pub const TITLE: Style = Style::new().bold();
    pub const ERROR: Style = Style::new()
        .fg_color(Some(Color::Ansi(AnsiColor::Red)))
        .bold();
    pub const SUCCESS: Style = Style::new()
        .fg_color(Some(Color::Ansi(AnsiColor::Green)))
        .bold();
    pub const ACTIVE: Style = Style::new()
        .fg_color(Some(Color::Ansi(AnsiColor::Yellow)))
        .bold();
}
