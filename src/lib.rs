//! devDeviceId
//!
//! A library to retrieve or generate a unique device ID.
//!
//! Example usage:
//!
//! ```rust
//! use deviceid::DevDeviceId;
//! let device_id = DevDeviceId::get_or_generate().unwrap();
//! eprintln!("Device ID: {}", device_id);
//! let device_id2 = DevDeviceId::get().unwrap();
//! assert_eq!(device_id, device_id2.unwrap());
//! ```
//!
//! Optional features:
//! - `serde`: (default) Enables serialization and deserialization of `DevDeviceId` using Serde
//!
//! **Note**: This crate assumes that the device ID is unlikely to be stored by multiple applications at once,
//! so it does not go to great lengths to ensure that it does not overwrite an existing ID.
use thiserror::Error;
use uuid::Uuid;

/// A unique identifier for a device, generated or retrieved from storage.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct DevDeviceId(Uuid);

mod unix;
mod windows;

mod storage {
    #[cfg(target_family = "unix")]
    pub use super::unix::*;
    #[cfg(target_family = "windows")]
    pub use super::windows::*;
}

/// Errors that can occur while retrieving or generating a device ID.
#[derive(Debug, Error)]
pub enum Error {
    /// Error with the underlying storage mechanism (file I/O on Unix, or registry on Windows)
    #[error("Failed to store or retrieve device ID due to storage error: {0}")]
    StorageError(String),
    /// Error when parsing the device ID as a UUID
    #[error("Failed to parse device ID, as UUID due to {0}")]
    BadUuidFormat(String),
    /// Error when the device ID is already set and cannot be generated again
    #[error("Device ID is already set")]
    AlreadySet,
}

pub type Result<T> = std::result::Result<T, Error>;

trait Storage {
    fn retrieve(&mut self) -> Result<Option<DevDeviceId>>;
    fn store(&mut self, id: &DevDeviceId) -> Result<()>;
}

struct PlatformStorage;

impl Storage for PlatformStorage {
    fn retrieve(&mut self) -> Result<Option<DevDeviceId>> {
        storage::retrieve()
    }

    fn store(&mut self, id: &DevDeviceId) -> Result<()> {
        storage::store(id)
    }
}

trait IdGenerator {
    fn generate(&mut self) -> DevDeviceId;
}

struct UuidV4Generator;

impl IdGenerator for UuidV4Generator {
    fn generate(&mut self) -> DevDeviceId {
        DevDeviceId(Uuid::new_v4())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum InitialDecision {
    Return(DevDeviceId),
    Generate,
}

fn decide_initial(existing: Option<DevDeviceId>) -> InitialDecision {
    match existing {
        Some(id) => InitialDecision::Return(id),
        None => InitialDecision::Generate,
    }
}

fn decide_after_store(generated: DevDeviceId, reloaded: Option<DevDeviceId>) -> DevDeviceId {
    reloaded.unwrap_or(generated)
}

fn get_with_storage<S: Storage>(storage: &mut S) -> Result<Option<DevDeviceId>> {
    storage.retrieve()
}

fn get_or_generate_with<S: Storage, G: IdGenerator>(
    storage: &mut S,
    generator: &mut G,
) -> Result<DevDeviceId> {
    match decide_initial(storage.retrieve()?) {
        InitialDecision::Return(id) => Ok(id),
        InitialDecision::Generate => {
            let generated = generator.generate();
            storage.store(&generated)?;
            let reloaded = storage.retrieve()?;
            Ok(decide_after_store(generated, reloaded))
        }
    }
}

impl DevDeviceId {
    /// Retrieves the device ID from storage or generates a new one if it doesn't exist.
    /// If an ID does not exist, a new one is generated and stored.
    /// If the function does not return `Ok(device_id)`, the generated ID was not stored.
    pub fn get_or_generate() -> Result<Self> {
        get_or_generate_with(&mut PlatformStorage, &mut UuidV4Generator)
    }

    /// Retrieves the device ID from storage, returning `None` if it does not exist
    /// or an error if there was a problem retrieving it.
    pub fn get() -> Result<Option<Self>> {
        get_with_storage(&mut PlatformStorage)
    }
}

impl std::fmt::Display for DevDeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[cfg(test)]
mod tests;
