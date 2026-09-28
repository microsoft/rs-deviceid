#![cfg(target_family = "windows")]

use crate::{DevDeviceId, Error, Result};
use windows_registry::{CURRENT_USER, Key, OpenOptions};

const REGISTRY_PATH: &str = r"SOFTWARE\Microsoft\DeveloperTools";
const REGISTRY_KEY: &str = "deviceid";

struct RegistryStorage<'a> {
    path: &'a str,
    value_name: &'a str,
}

impl<'a> RegistryStorage<'a> {
    const fn new(path: &'a str, value_name: &'a str) -> Self {
        Self { path, value_name }
    }

    fn open_read_key(&self) -> Result<Option<Key>> {
        reg_options(false)
            .open(self.path)
            .map(Some)
            .or_else(error_not_found_to_none)
    }

    fn open_create_key(&self) -> Result<Key> {
        reg_options(true).open(self.path).map_err(storage_error)
    }

    fn retrieve(&self) -> Result<Option<DevDeviceId>> {
        let Some(key) = self.open_read_key()? else {
            return Ok(None);
        };
        match key.get_string(self.value_name) {
            Ok(s) => {
                let uuid =
                    uuid::Uuid::try_parse(&s).map_err(|e| Error::BadUuidFormat(e.to_string()))?;
                Ok(Some(DevDeviceId(uuid)))
            }
            Err(err) => error_not_found_to_none(err),
        }
    }

    fn store(&self, id: &DevDeviceId) -> Result<()> {
        let key = self.open_create_key()?;
        let s = id.to_string();
        key.set_string(self.value_name, &s).map_err(storage_error)
    }
}

const PLATFORM_STORAGE: RegistryStorage<'static> =
    RegistryStorage::new(REGISTRY_PATH, REGISTRY_KEY);

fn reg_options(create: bool) -> OpenOptions<'static> {
    let mut options = CURRENT_USER.options();
    options.read().wow64_64();
    if create {
        options.write();
        options.create();
    }
    options
}

/// Maps Windows "not found" errors to `Ok(None)`, and all other errors to [`Error::StorageError`].
fn error_not_found_to_none<T>(err: windows_result::Error) -> Result<Option<T>> {
    if std::io::Error::from(err.clone()).kind() == std::io::ErrorKind::NotFound {
        Ok(None)
    } else {
        Err(storage_error(err))
    }
}

fn storage_error(err: windows_result::Error) -> Error {
    Error::StorageError(err.to_string())
}

pub fn retrieve() -> Result<Option<DevDeviceId>> {
    PLATFORM_STORAGE.retrieve()
}

pub fn store(id: &DevDeviceId) -> Result<()> {
    PLATFORM_STORAGE.store(id)
}

#[cfg(test)]
mod tests;
