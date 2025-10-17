use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path,
};

use anyhow::{Context, Result};
use camino::Utf8PathBuf;

pub(crate) mod operms;
pub(crate) use operms::OPerms;

#[cfg(windows)]
const SEP: &str = ";";
#[cfg(not(windows))]
const SEP: &str = ":";

#[cfg(windows)]
const PATH_SEP: &str = "\\";
#[cfg(not(windows))]
const PATH_SEP: &str = "/";

#[cfg(windows)]
const PATH_SEP_OP: &str = "/";
#[cfg(not(windows))]
const PATH_SEP_OP: &str = "\\";

pub(crate) fn open<P: Into<Utf8PathBuf>>(path: P, perms: OPerms) -> Result<File> {
    let path: Utf8PathBuf = path.into();
    if !path.try_exists()? && !perms.contains(OPerms::CREATE) {
        return Err(anyhow::anyhow!("{} does not exist", path));
    }

    let mut opts = OpenOptions::new();
    if perms.contains(OPerms::READ) {
        opts.read(true);
    }
    if perms.contains(OPerms::WRITE) {
        opts.write(true);
    }
    if perms.contains(OPerms::APPEND) {
        opts.append(true);
    }
    if perms.contains(OPerms::CREATE) {
        opts.create(true);
    }
    if perms.contains(OPerms::TRUNC) {
        opts.truncate(true);
    }

    Ok(opts.open(&path)?)
}

pub(crate) fn get_tools_paths() -> Result<String> {
    let root = pwd()?.join("rust").join("tools");
    if !root.try_exists()? {
        anyhow::bail!("{} does not exist", root);
    }

    let curr: String = std::env::var_os("PATH")
        .unwrap_or_default()
        .into_string()
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    Ok([
        normalize_path(root.join("cmake")).as_str(),
        normalize_path(root.join("gcc")).as_str(),
        normalize_path(root.join("ninja")).as_str(),
    ]
    .into_iter()
    .chain(curr.split(SEP).filter(|s| !s.is_empty()))
    .collect::<Vec<_>>()
    .join(SEP))
}

pub(crate) fn read(f: &mut File) -> Result<String> {
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf)
}

pub(crate) fn rm<P: Into<Utf8PathBuf>>(path: P) -> Result<()> {
    let path: Utf8PathBuf = path.into();
    if !exists(&path)? {
        eprintln!("{} does not exist", path);
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub(crate) fn mkdir<P: Into<Utf8PathBuf>>(dir: P) -> Result<()> {
    let dir: Utf8PathBuf = dir.into();
    if exists(&dir)? {
        if !dir.is_dir() {
            anyhow::bail!("{} already exists as not a directory", dir);
        }
        return Ok(());
    }

    fs::create_dir_all(dir.as_std_path())?;
    Ok(())
}

pub(crate) fn cd<P: Into<Utf8PathBuf>>(dir: P) -> Result<()> {
    std::env::set_current_dir(dir.into()).context("could not chdir")
}

pub(crate) fn pwd() -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .map_err(|e| anyhow::anyhow!("bad path: {:?}", e))
}

pub(crate) fn add_env_path<C: Into<String>, D: Into<Utf8PathBuf>>(curr: C, new_dir: D) -> String {
    let curr: String = curr.into();
    let new_dir = normalize_path(new_dir.into());

    let mut parts: Vec<&str> = curr.split(SEP).filter(|s| !s.is_empty()).collect();
    parts.push((new_dir).as_str());
    parts.join(SEP)
}

pub(crate) fn normalize_path<P: Into<String>>(path: P) -> String {
    let path: String = path.into();
    path.replace(PATH_SEP_OP, PATH_SEP)
}

pub(crate) fn write(f: &mut File, c: &[u8]) -> Result<()> {
    Ok(f.write_all(c)?)
}

/// Truncates the file before writing
pub(crate) fn write_over(f: &mut File, c: &[u8]) -> Result<()> {
    f.set_len(0)?;
    f.seek(SeekFrom::Start(0))?;
    Ok(f.write_all(c)?)
}

pub(crate) fn exists<P: Into<Utf8PathBuf>>(path: P) -> Result<bool> {
    let path = path.into();
    Ok(path
        .try_exists()
        .context("failed to check if path exists")?)
}
