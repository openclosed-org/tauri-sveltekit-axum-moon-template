use std::path::PathBuf;

use anyhow::{Result, bail};

pub(crate) fn user_home_dir() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home));
    }
    if let Some(home) = std::env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(home));
    }
    let drive = std::env::var_os("HOMEDRIVE");
    let path = std::env::var_os("HOMEPATH");
    match (drive, path) {
        (Some(drive), Some(path)) => {
            let mut full = PathBuf::from(drive);
            full.push(path);
            Ok(full)
        }
        _ => bail!("HOME/USERPROFILE/HOMEDRIVE+HOMEPATH are not set"),
    }
}
