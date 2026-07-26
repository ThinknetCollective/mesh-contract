use crate::error::SaveError;

/// Internal abstraction over storage backends.
///
/// `NativeBackend` (disk) is selected at compile time on non-WASM targets.
/// `WasmBackend` (localStorage) is selected on `wasm32` targets.
/// Callers never interact with this trait directly — they use the free
/// functions `save_session` / `load_session` exported from `ops`.
pub(crate) trait StorageBackend {
    fn write(&self, key: &str, data: &str) -> Result<(), SaveError>;
    fn read(&self, key: &str) -> Result<String, SaveError>;
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod native;

#[cfg(target_arch = "wasm32")]
pub(crate) mod wasm;
