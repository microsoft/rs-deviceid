# GitHub Copilot Instructions for rs-deviceid

## Project Overview
`deviceid` is a Rust library that provides a unique device ID for a given system, based on the devDeviceId specification. It generates or retrieves a persistent device identifier stored in platform-specific locations.

## Tech Stack
- **Language**: Rust (Edition 2024)
- **Minimum Rust Version**: 1.88
- **Key Dependencies**:
  - `uuid` (v1.18) - for generating v4 UUIDs
  - `serde` (v1.0) - optional, for serialization support
  - `thiserror` (v2.0) - for error handling
  - `winreg` (v0.55.0) - Windows-specific registry access

## Platform-Specific Implementation
This crate has separate implementations for different platforms:
- **Unix/Linux/macOS**: Uses file-based storage (`src/unix.rs`)
  - Linux: Stores in `$XDG_CACHE_HOME/Microsoft/DeveloperTools/deviceid` or `$HOME/.cache/Microsoft/DeveloperTools/deviceid`
  - macOS: Stores in `$HOME/Library/Application Support/Microsoft/DeveloperTools/deviceid`
- **Windows**: Uses Windows Registry (`src/windows.rs`)
  - Registry path: `HKEY_CURRENT_USER\SOFTWARE\Microsoft\DeveloperTools`
  - Registry key: `deviceid`

## Build, Test, and Lint Commands
**Always run these commands in order before committing:**
1. **Format check**: `cargo fmt -- --check`
2. **Linting**: `cargo clippy -- -D warnings`
3. **Build**: `cargo build --verbose`
4. **Test**: `cargo test --verbose`

**Note**: All of these checks are enforced in CI via `.github/workflows/rust.yml`.

## Coding Conventions and Style
- Follow standard Rust formatting (use `cargo fmt`)
- All code must pass `cargo clippy` with no warnings (enforced with `-D warnings`)
- Use the Rust 2024 edition features
- Error handling should use the `thiserror` crate and the defined `Error` enum
- Maintain platform-specific code in separate modules (`unix.rs` and `windows.rs`)
- Use conditional compilation with `cfg` attributes for platform-specific code
- Keep the public API minimal and focused on the core functionality

## Error Handling Standards
- Use the custom `Error` enum defined in `lib.rs` with variants:
  - `StorageError`: For file I/O or registry errors
  - `BadUuidFormat`: For UUID parsing errors
  - `AlreadySet`: When attempting to store a device ID that already exists in storage
- Always convert platform errors to our custom error types with context using `.map_err()`

## Testing Requirements
- All tests must pass with `cargo test --verbose`
- Platform-specific tests are located in the `tests/` directory
- Tests should verify idempotency of `get_or_generate()`

## Features
- **`serde`** (default feature): Enables serialization/deserialization of `DevDeviceId`
- When adding features, ensure they are optional and don't break existing functionality

## Restrictions and Boundaries
- **DO NOT** change the storage locations (registry paths on Windows, file paths on Unix)
- **DO NOT** modify the UUID format (must remain lowercase hyphenated format)
- **DO NOT** introduce breaking changes to the public API without careful consideration
- **DO NOT** add new dependencies unless absolutely necessary
- **DO NOT** add platform-specific code outside of the designated modules (`unix.rs`, `windows.rs`)
- **DO NOT** remove or weaken the CI checks (fmt, clippy, build, test)

## Security Considerations
- Device IDs are stored in user-accessible locations (not encrypted)
- The crate assumes device ID is unlikely to be stored by multiple applications simultaneously
- **Platform behavior differences**:
  - **Unix**: Implements file locking check - returns `AlreadySet` error when attempting to store if file already exists
  - **Windows**: No pre-write check - will overwrite existing registry value without error
- Race conditions during concurrent writes are not handled on either platform

## Documentation Standards
- All public APIs must have doc comments
- Examples should be provided in doc comments where helpful
- Keep the `README.md` up to date with API changes
- Use `//!` for module-level documentation
- Use `///` for function/struct/enum documentation

## Example Code Style
```rust
// Good: Clear error handling with context (Unix file-based storage)
pub fn retrieve() -> Result<Option<DevDeviceId>> {
    let path = path()?;
    if path.exists() {
        let data = std::fs::read(path)
            .map_err(|e| super::Error::StorageError(e.to_string()))?;
        // Unix: Reads file as bytes, parses ASCII representation of UUID
        // File contains UUID string like "550e8400-e29b-41d4-a716-446655440000"
        let id = uuid::Uuid::try_parse_ascii(data.as_slice())
            .map_err(|e| super::Error::BadUuidFormat(e.to_string()))?;
        Ok(Some(DevDeviceId(id)))
    } else {
        Ok(None)
    }
}

// Good: Platform-specific parsing (Windows registry string data)
// uuid::Uuid::try_parse(&value) for string values from Windows registry

// Good: Platform-specific conditional compilation
#[cfg(target_family = "unix")]
pub use super::unix::*;
#[cfg(target_family = "windows")]
pub use super::windows::*;
```

## Additional Notes
- This is a Microsoft open source project following the Microsoft Open Source Code of Conduct
- Contributors must agree to the CLA (Contributor License Agreement)
- The crate name uses `deviceid` but the struct name is `DevDeviceId` (matching the specification)
- UUID generation uses v4 (random) UUIDs via the `uuid` crate
