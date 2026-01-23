#![cfg(target_family = "windows")]

use crate::{DevDeviceId, Error, Result};
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, KEY_WOW64_64KEY};

const REGISTRY_PATH: &str = "SOFTWARE\\Microsoft\\DeveloperTools";
const REGISTRY_KEY: &str = "deviceid";

/// Trait to abstract Windows registry operations for testability
trait WindowsRegistry {
    /// Opens a registry key for reading, returns None if the key doesn't exist
    fn open_read_key(&self, path: &str) -> Result<bool>;

    /// Opens or creates a registry key for writing
    fn open_create_key(&self, path: &str) -> Result<()>;

    /// Gets a string value from the registry
    fn get_value(&self, path: &str, key: &str) -> Result<Option<String>>;

    /// Sets a string value in the registry
    fn set_value(&self, path: &str, key: &str, value: &str) -> Result<()>;
}

/// Real implementation that uses the actual Windows registry via winreg crate
struct RealWindowsRegistry;

impl WindowsRegistry for RealWindowsRegistry {
    fn open_read_key(&self, path: &str) -> Result<bool> {
        let result = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(path, KEY_WOW64_64KEY | KEY_READ);
        match result {
            Ok(_) => Ok(true),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Ok(false),
                _ => Err(Error::StorageError(err.to_string())),
            },
        }
    }

    fn open_create_key(&self, path: &str) -> Result<()> {
        RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(path, KEY_WOW64_64KEY | KEY_ALL_ACCESS)
            .map(|(_, _)| ())
            .map_err(|e| Error::StorageError(e.to_string()))
    }

    fn get_value(&self, path: &str, key: &str) -> Result<Option<String>> {
        let reg_key = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(path, KEY_WOW64_64KEY | KEY_READ);

        match reg_key {
            Ok(k) => match k.get_value::<String, &str>(key) {
                Ok(value) => Ok(Some(value)),
                Err(err) => match err.kind() {
                    std::io::ErrorKind::NotFound => Ok(None),
                    _ => Err(Error::StorageError(err.to_string())),
                },
            },
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Ok(None),
                _ => Err(Error::StorageError(err.to_string())),
            },
        }
    }

    fn set_value(&self, path: &str, key: &str, value: &str) -> Result<()> {
        let reg_key = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(path, KEY_WOW64_64KEY | KEY_ALL_ACCESS)
            .map(|(k, _)| k)
            .map_err(|e| Error::StorageError(e.to_string()))?;

        reg_key
            .set_value(key, &value)
            .map_err(|e| Error::StorageError(e.to_string()))?;
        Ok(())
    }
}

/// Mock implementation for testing that uses an in-memory HashMap
#[cfg(test)]
struct MockWindowsRegistry {
    data: std::sync::Arc<
        std::sync::Mutex<
            std::collections::HashMap<String, std::collections::HashMap<String, String>>,
        >,
    >,
}

#[cfg(test)]
impl MockWindowsRegistry {
    fn new() -> Self {
        Self {
            data: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    fn with_value(path: &str, key: &str, value: &str) -> Self {
        let registry = Self::new();
        let mut data = registry.data.lock().unwrap();
        data.entry(path.to_string())
            .or_insert_with(std::collections::HashMap::new)
            .insert(key.to_string(), value.to_string());
        drop(data);
        registry
    }
}

#[cfg(test)]
impl WindowsRegistry for MockWindowsRegistry {
    fn open_read_key(&self, path: &str) -> Result<bool> {
        let data = self.data.lock().unwrap();
        Ok(data.contains_key(path))
    }

    fn open_create_key(&self, path: &str) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.entry(path.to_string())
            .or_insert_with(std::collections::HashMap::new);
        Ok(())
    }

    fn get_value(&self, path: &str, key: &str) -> Result<Option<String>> {
        let data = self.data.lock().unwrap();
        Ok(data.get(path).and_then(|keys| keys.get(key).cloned()))
    }

    fn set_value(&self, path: &str, key: &str, value: &str) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.entry(path.to_string())
            .or_insert_with(std::collections::HashMap::new)
            .insert(key.to_string(), value.to_string());
        Ok(())
    }
}

fn retrieve_with_registry<R: WindowsRegistry>(registry: &R) -> Result<Option<DevDeviceId>> {
    if !registry.open_read_key(REGISTRY_PATH)? {
        return Ok(None);
    }

    match registry.get_value(REGISTRY_PATH, REGISTRY_KEY)? {
        Some(value) => {
            let uuid =
                uuid::Uuid::try_parse(&value).map_err(|e| Error::BadUuidFormat(e.to_string()))?;
            Ok(Some(DevDeviceId(uuid)))
        }
        None => Ok(None),
    }
}

fn store_with_registry<R: WindowsRegistry>(registry: &R, id: &DevDeviceId) -> Result<()> {
    registry.open_create_key(REGISTRY_PATH)?;
    let s = id.to_string();
    registry.set_value(REGISTRY_PATH, REGISTRY_KEY, &s)?;
    Ok(())
}

pub fn retrieve() -> Result<Option<DevDeviceId>> {
    retrieve_with_registry(&RealWindowsRegistry)
}

pub fn store(id: &DevDeviceId) -> Result<()> {
    store_with_registry(&RealWindowsRegistry, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_returns_none_with_empty_registry() {
        let registry = MockWindowsRegistry::new();
        let result = retrieve_with_registry(&registry).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_returns_value_with_preinitialized_registry() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let registry = MockWindowsRegistry::with_value(REGISTRY_PATH, REGISTRY_KEY, uuid_str);

        let result = retrieve_with_registry(&registry).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), uuid_str);
    }

    #[test]
    fn test_store_and_retrieve() {
        let registry = MockWindowsRegistry::new();
        let id = DevDeviceId(uuid::Uuid::new_v4());

        store_with_registry(&registry, &id).unwrap();
        let retrieved = retrieve_with_registry(&registry).unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), id);
    }

    #[test]
    fn test_get_or_generate_with_empty_registry() {
        // This test simulates the behavior of get_or_generate with an empty registry
        let registry = MockWindowsRegistry::new();

        // First call should return None (empty registry)
        let initial = retrieve_with_registry(&registry).unwrap();
        assert!(initial.is_none());

        // Generate and store a new ID
        let new_id = DevDeviceId(uuid::Uuid::new_v4());
        store_with_registry(&registry, &new_id).unwrap();

        // Second call should return the stored ID
        let retrieved = retrieve_with_registry(&registry).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), new_id);
    }
}
