#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsValue;

use crate::error::SaveError;

use super::StorageBackend;

pub(crate) struct WasmBackend;

impl StorageBackend for WasmBackend {
    /// Save `data` to `localStorage` using `key` as the storage key.
    ///
    /// The `key` is the `SessionId` string directly (max 128 chars), satisfying
    /// the ≤128-char constraint structurally.
    fn write(&self, key: &str, data: &str) -> Result<(), SaveError> {
        let storage = get_local_storage()?;
        storage
            .set_item(key, data)
            .map_err(js_err_to_io)
    }

    /// Read `key` from `localStorage`.  Returns `SaveError::Io` if the key is
    /// not present or `localStorage` is unavailable.
    fn read(&self, key: &str) -> Result<String, SaveError> {
        let storage = get_local_storage()?;
        match storage.get_item(key).map_err(js_err_to_io)? {
            Some(value) => Ok(value),
            None => Err(SaveError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("session '{}' not found in localStorage", key),
            ))),
        }
    }
}

fn get_local_storage() -> Result<web_sys::Storage, SaveError> {
    let window = web_sys::window().ok_or_else(|| {
        SaveError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "no browser window object available",
        ))
    })?;
    let storage = window
        .local_storage()
        .map_err(js_err_to_io)?
        .ok_or_else(|| {
            SaveError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "localStorage is unavailable in this browser context",
            ))
        })?;
    Ok(storage)
}

fn js_err_to_io(js_val: JsValue) -> SaveError {
    let msg = js_sys::JSON::stringify(&js_val)
        .ok()
        .and_then(|s| s.as_string())
        .unwrap_or_else(|| "unknown localStorage error".to_string());
    SaveError::Io(std::io::Error::new(std::io::ErrorKind::Other, msg))
}
