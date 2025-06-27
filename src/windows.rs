#![cfg(target_family = "windows")]

use crate::{DevDeviceId, Error, Result};
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, KEY_WOW64_64KEY};

const REGISTRY_PATH: &str = "SOFTWARE\\Microsoft\\DeveloperTools";
const REGISTRY_KEY: &str = "deviceid";

fn open_read_key() -> Result<Option<RegKey>> {
    let result = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(REGISTRY_PATH, KEY_WOW64_64KEY | KEY_READ);
    match result {
        Ok(key) => Ok(Some(key)),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => Ok(None),
            _ => Err(Error::StorageError(err.to_string())),
        },
    }
}

fn open_create_key() -> Result<RegKey> {
    RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey_with_flags(REGISTRY_PATH, KEY_WOW64_64KEY | KEY_ALL_ACCESS)
        .map(|(key, _)| key)
        .map_err(|e| Error::StorageError(e.to_string()))
}

pub fn retrieve() -> Result<Option<DevDeviceId>> {
    let Some(key) = open_read_key()? else {
        return Ok(None);
    };
    match key.get_value::<String, &str>(REGISTRY_KEY) {
        Ok(value) => {
            let uuid =
                uuid::Uuid::try_parse(&value).map_err(|e| Error::BadUuidFormat(e.to_string()))?;
            Ok(Some(DevDeviceId(uuid)))
        }
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => Ok(None),
            _ => Err(Error::StorageError(err.to_string())),
        },
    }
}

pub fn store(id: &DevDeviceId) -> Result<()> {
    let key = open_create_key()?;
    let s = id.to_string();
    key.set_value(REGISTRY_KEY, &s)
        .map_err(|e| Error::StorageError(e.to_string()))?;
    Ok(())
}
