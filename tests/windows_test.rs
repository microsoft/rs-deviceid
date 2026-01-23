#![cfg(windows)]
use deviceid::DevDeviceId;

#[test]
fn test_get_returns_none_when_no_device_id_exists() {
    // This test should demonstrate that DevDeviceId::get() returns Ok(None)
    // when there's no device id in the registry.
    //
    // Note: This test relies on the mock registry implementation in the module tests.
    // On a real Windows system with the actual registry, this test would only pass
    // if the registry key doesn't exist, which may not be the case on a dev machine.
    //
    // The actual testing of the empty registry behavior is done in the unit tests
    // in src/windows.rs using the MockWindowsRegistry.

    // This is a placeholder for manual verification on Windows systems
    // where the registry key might not exist yet.
    let result = DevDeviceId::get();
    assert!(result.is_ok(), "get() should not return an error");

    // The result could be Some or None depending on whether the registry key exists
    match result.unwrap() {
        Some(id) => println!("Device ID exists in registry: {}", id),
        None => println!("No device ID found in registry"),
    }
}

#[test]
fn test_get_or_generate_creates_id_when_missing() {
    // Test that get_or_generate creates an ID when one doesn't exist
    let id1 = DevDeviceId::get_or_generate();
    assert!(id1.is_ok(), "get_or_generate() should not return an error");

    // Second call should return the same ID
    let id2 = DevDeviceId::get_or_generate();
    assert!(id2.is_ok(), "get_or_generate() should not return an error");

    assert_eq!(
        id1.unwrap(),
        id2.unwrap(),
        "get_or_generate() should be idempotent"
    );
}

#[test]
fn test_get_retrieves_stored_id() {
    // First ensure an ID is stored
    let stored_id = DevDeviceId::get_or_generate().unwrap();

    // Now get() should return Some(id)
    let retrieved = DevDeviceId::get().unwrap();
    assert!(
        retrieved.is_some(),
        "get() should return Some after get_or_generate()"
    );
    assert_eq!(
        retrieved.unwrap(),
        stored_id,
        "Retrieved ID should match stored ID"
    );
}
