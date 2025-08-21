#![cfg(unix)]
use deviceid::DevDeviceId;
use std::path::PathBuf;

#[test]
fn test_get_or_generate_first_time() {
    // set HOME to a temporary directory
    let tmp_home = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("fake_home");
    println!("Using tmp home: {}", tmp_home.display());
    std::fs::create_dir_all(&tmp_home).unwrap();
    unsafe { std::env::set_var("HOME", tmp_home) }

    let no_id = DevDeviceId::get().unwrap();
    assert!(no_id.is_none());

    let id = DevDeviceId::get_or_generate().unwrap();
    let id2 = DevDeviceId::get().unwrap().unwrap();
    assert_eq!(id, id2);
}
