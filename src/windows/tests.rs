use super::*;

const TEST_REGISTRY_PARENT: &str = r"SOFTWARE";

struct TestRegistryGuard {
    child_path: String,
}

impl Drop for TestRegistryGuard {
    fn drop(&mut self) {
        let mut options = CURRENT_USER.options();
        options.read().write().wow64_64();
        if let Ok(parent) = options.open(TEST_REGISTRY_PARENT) {
            _ = parent.remove_tree(&self.child_path);
        }
    }
}

fn test_storage() -> (String, TestRegistryGuard) {
    let child_path = format!("rs-deviceid-test-{}", uuid::Uuid::new_v4());
    let path = format!(r"{TEST_REGISTRY_PARENT}\{child_path}");
    (path, TestRegistryGuard { child_path })
}

#[test]
fn registry_storage_returns_none_when_key_is_missing() {
    let (path, _guard) = test_storage();
    let storage = RegistryStorage::new(&path, REGISTRY_KEY);

    assert_eq!(storage.retrieve().unwrap(), None);
}

#[test]
fn registry_storage_returns_none_when_value_is_missing() {
    let (path, _guard) = test_storage();
    let storage = RegistryStorage::new(&path, REGISTRY_KEY);
    drop(storage.open_create_key().unwrap());

    assert_eq!(storage.retrieve().unwrap(), None);
}

#[test]
fn registry_storage_round_trips_device_id() {
    let (path, _guard) = test_storage();
    let storage = RegistryStorage::new(&path, REGISTRY_KEY);
    let id = DevDeviceId(uuid::Uuid::new_v4());

    storage.store(&id).unwrap();

    assert_eq!(storage.retrieve().unwrap(), Some(id));
}

#[test]
fn registry_storage_rejects_invalid_uuid() {
    let (path, _guard) = test_storage();
    let storage = RegistryStorage::new(&path, REGISTRY_KEY);
    storage
        .open_create_key()
        .unwrap()
        .set_string(REGISTRY_KEY, "not-a-uuid")
        .unwrap();

    assert!(matches!(storage.retrieve(), Err(Error::BadUuidFormat(_))));
}

#[test]
fn other_windows_errors_map_to_storage_error() {
    let err = windows_result::Error::empty();
    let result: Result<Option<()>> = error_not_found_to_none(err);

    assert!(matches!(result, Err(Error::StorageError(_))));
}
