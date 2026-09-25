#![cfg(target_family = "windows")]

use crate::{DevDeviceId, Error, Result};
use windows_registry::{CURRENT_USER, Key, OpenOptions};

const REGISTRY_PATH: &str = r"SOFTWARE\Microsoft\DeveloperTools";
const REGISTRY_KEY: &str = "deviceid";

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

fn open_read_key() -> Result<Option<Key>> {
    reg_options(false)
        .open(REGISTRY_PATH)
        .map(Some)
        .or_else(error_not_found_to_none)
}

fn open_create_key() -> Result<Key> {
    reg_options(true).open(REGISTRY_PATH).map_err(storage_error)
}

pub fn retrieve() -> Result<Option<DevDeviceId>> {
    let Some(key) = open_read_key()? else {
        return Ok(None);
    };
    match key.get_string(REGISTRY_KEY) {
        Ok(s) => {
            let uuid =
                uuid::Uuid::try_parse(&s).map_err(|e| Error::BadUuidFormat(e.to_string()))?;
            Ok(Some(DevDeviceId(uuid)))
        }
        Err(err) => error_not_found_to_none(err),
    }
}

pub fn store(id: &DevDeviceId) -> Result<()> {
    let key = open_create_key()?;
    let s = id.to_string();
    key.set_string(REGISTRY_KEY, &s).map_err(storage_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_not_found_maps_to_none() {
        let missing_path = format!(
            r"SOFTWARE\Microsoft\DeveloperTools\deviceid-test-{}",
            uuid::Uuid::new_v4()
        );
        let err = CURRENT_USER
            .open(missing_path)
            .expect_err("random registry path should not exist");
        let result: Result<Option<()>> = error_not_found_to_none(err);

        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn other_windows_errors_map_to_storage_error() {
        let err = windows_result::Error::empty();
        let result: Result<Option<()>> = error_not_found_to_none(err);

        assert!(matches!(result, Err(Error::StorageError(_))));
    }
}
