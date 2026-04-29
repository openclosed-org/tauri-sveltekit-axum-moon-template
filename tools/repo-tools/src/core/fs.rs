use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use tempfile::TempDir;
use walkdir::WalkDir;

pub(crate) fn read(path: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read {}", path.as_ref().display()))
}

pub(crate) fn write(path: impl AsRef<Path>, content: &str) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path.as_ref(), content)
        .with_context(|| format!("failed to write {}", path.as_ref().display()))
}

#[allow(dead_code)]
pub(crate) fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

pub(crate) fn collect_files_with_extension(
    root: impl AsRef<Path>,
    extension: &str,
) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension() == Some(OsStr::new(extension)))
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

pub(crate) fn collect_files_named(root: impl AsRef<Path>, file_name: &str) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.file_name() == OsStr::new(file_name))
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

pub(crate) fn list_directories(root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut directories = Vec::new();
    for entry in fs::read_dir(root.as_ref())
        .with_context(|| format!("failed to read {}", root.as_ref().display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            directories.push(entry.path());
        }
    }
    directories.sort();
    Ok(directories)
}

#[allow(dead_code)]
pub(crate) fn wait_for_port(host: &str, port: u16, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if is_port_open(host, port) {
            return true;
        }
        thread::sleep(Duration::from_secs(1));
    }
    false
}

#[allow(dead_code)]
pub(crate) fn is_port_open(host: &str, port: u16) -> bool {
    let address = format!("{host}:{port}");
    address
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.next())
        .and_then(|addr| TcpStream::connect_timeout(&addr, Duration::from_millis(500)).ok())
        .is_some()
}

pub(crate) fn copy_dir_contents(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    for entry in WalkDir::new(source) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(source)?;
        let target = destination.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("failed to create {}", target.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target).with_context(|| {
                format!("failed to copy {} to {}", path.display(), target.display())
            })?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn remove_dir_contents(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(&entry_path)
                .with_context(|| format!("failed to remove {}", entry_path.display()))?;
        } else {
            fs::remove_file(&entry_path)
                .with_context(|| format!("failed to remove {}", entry_path.display()))?;
        }
    }
    Ok(())
}

pub(crate) fn tempdir() -> Result<TempDir> {
    TempDir::new().context("failed to create temporary directory")
}

#[allow(dead_code)]
pub(crate) fn read_binary(path: &Path) -> Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("failed to read {}", path.display()))
}

pub(crate) fn download_to(url: &str, path: &Path) -> Result<()> {
    let response = reqwest::blocking::get(url)
        .with_context(|| format!("failed to download {url}"))?
        .error_for_status()
        .with_context(|| format!("download returned error status for {url}"))?;
    let bytes = response.bytes().context("failed to read download bytes")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, &bytes).with_context(|| format!("failed to write {}", path.display()))
}

pub(crate) fn extract_tar_gz(
    archive: &Path,
    destination: &Path,
    target_suffix: Option<&str>,
) -> Result<()> {
    let reader =
        fs::File::open(archive).with_context(|| format!("failed to open {}", archive.display()))?;
    let decoder = flate2::read::GzDecoder::new(reader);
    let mut archive = tar::Archive::new(decoder);
    for entry in archive.entries().context("failed to list tar entries")? {
        let mut entry = entry?;
        let entry_path = entry.path()?.to_path_buf();
        let should_extract = target_suffix
            .map(|suffix| entry_path.to_string_lossy().ends_with(suffix))
            .unwrap_or(true);
        if !should_extract {
            continue;
        }
        let target = if let Some(suffix) = target_suffix {
            destination.join(
                entry_path
                    .to_string_lossy()
                    .strip_suffix(suffix)
                    .map(|_| Path::new(suffix).file_name().unwrap_or(OsStr::new("out")))
                    .unwrap_or_else(|| entry_path.file_name().unwrap_or(OsStr::new("out"))),
            )
        } else {
            destination.join(entry_path.file_name().unwrap_or(OsStr::new("out")))
        };
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        entry.unpack(&target)?;
    }
    Ok(())
}

pub(crate) fn extract_zip_file(
    archive: &Path,
    destination: &Path,
    needle: &str,
) -> Result<PathBuf> {
    let file =
        fs::File::open(archive).with_context(|| format!("failed to open {}", archive.display()))?;
    let mut archive = zip::ZipArchive::new(file).context("failed to read zip archive")?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        if !name.ends_with(needle) {
            continue;
        }
        let target = destination.join(Path::new(needle).file_name().unwrap_or(OsStr::new(needle)));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&target)?;
        io::copy(&mut entry, &mut out)?;
        return Ok(target);
    }
    bail!("zip archive does not contain {needle}")
}

pub(crate) fn normalize_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[allow(dead_code)]
pub(crate) fn walk_relative_dirs(root: &Path, max_depth: usize) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for entry in WalkDir::new(root).max_depth(max_depth).min_depth(1) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            result.push(normalize_slashes(entry.path().strip_prefix(root)?));
        }
    }
    result.sort();
    Ok(result)
}

#[allow(dead_code)]
pub(crate) fn read_stdin_string() -> Result<String> {
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("failed to read stdin")?;
    Ok(buffer)
}

#[allow(dead_code)]
pub(crate) fn btreeset<I: IntoIterator<Item = String>>(items: I) -> BTreeSet<String> {
    items.into_iter().collect()
}
