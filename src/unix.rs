#![cfg(target_family = "unix")]

use crate::{DevDeviceId, Result, Storage};

const DEV_DEVICEID_PATH: &str = "Microsoft/DeveloperTools";
const FILENAME: &str = "deviceid";

#[cfg(target_os = "macos")]
fn root_path() -> Result<std::path::PathBuf> {
    const BASE_STORAGE_PATH: &str = "Library/Application Support";
    let home = std::env::var_os("HOME");
    match home {
        Some(home) => {
            let mut path = std::path::PathBuf::from(home);
            path.push(BASE_STORAGE_PATH);
            Ok(path)
        }
        None => Err(super::Error::StorageError(
            "HOME environment variable not set".to_string(),
        )),
    }
}

#[cfg(target_os = "linux")]
fn root_path() -> Result<std::path::PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| {
                let mut path = std::path::PathBuf::from(home);
                path.push(".cache");
                path
            })
        })
        .ok_or_else(|| {
            super::Error::StorageError(
                "XDG_CACHE_HOME and HOME environment variables not set".to_string(),
            )
        })
}

fn folder_path() -> Result<std::path::PathBuf> {
    let mut path = root_path()?;
    path.push(DEV_DEVICEID_PATH);
    Ok(path)
}

fn path() -> Result<std::path::PathBuf> {
    let mut path = folder_path()?;
    path.push(FILENAME);
    Ok(path)
}

pub struct UnixStorage;

impl Storage for UnixStorage {
    fn retrieve(&self) -> Result<Option<DevDeviceId>> {
        let path = path()?;
        if path.exists() {
            // TODO: don't read too much!
            let data =
                std::fs::read(path).map_err(|e| super::Error::StorageError(e.to_string()))?;
            let id = uuid::Uuid::try_parse_ascii(data.as_slice())
                .map_err(|e| super::Error::BadUuidFormat(e.to_string()))?;
            Ok(Some(DevDeviceId(id)))
        } else {
            Ok(None)
        }
    }

    fn store(&mut self, id: &DevDeviceId) -> Result<()> {
        std::fs::create_dir_all(folder_path()?)
            .map_err(|e| super::Error::StorageError(e.to_string()))?;
        if !path()?.exists() {
            let id_str = format!("{id}");
            std::fs::write(path()?, id_str.as_bytes())
                .map_err(|e| super::Error::StorageError(e.to_string()))?;
            Ok(())
        } else {
            Err(super::Error::AlreadySet)
        }
    }
}

pub fn retrieve() -> Result<Option<DevDeviceId>> {
    UnixStorage.retrieve()
}

pub fn store(id: &DevDeviceId) -> Result<()> {
    let mut storage = UnixStorage;
    storage.store(id)
}
