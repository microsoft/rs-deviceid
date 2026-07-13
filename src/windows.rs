#![cfg(target_family = "windows")]

use crate::{DevDeviceId, Error, Result};
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::KEY_WOW64_64KEY;
use windows_registry::{CURRENT_USER, Key, OpenOptions};
use windows_result::HRESULT;

const REGISTRY_PATH: &str = r"SOFTWARE\Microsoft\DeveloperTools";
const REGISTRY_KEY: &str = "deviceid";

fn reg_options(create: bool) -> OpenOptions<'static> {
    let mut options = CURRENT_USER.options();
    options.read().access(KEY_WOW64_64KEY.0);
    if create {
        options.write();
        options.create();
    }
    options
}

/// Maps [`ERROR_FILE_NOT_FOUND`] to Ok(None), and all other errors to [`Error::StorageError`].
fn error_not_found_to_none<T>(err: windows_result::Error) -> Result<Option<T>> {
    match err.code() {
        hr if hr == HRESULT::from(ERROR_FILE_NOT_FOUND) => Ok(None),
        _ => Err(storage_error(err)),
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
