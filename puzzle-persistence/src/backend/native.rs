#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::Path;

use crate::error::SaveError;

use super::StorageBackend;

pub(crate) struct NativeBackend;

impl StorageBackend for NativeBackend {
    /// Atomically write `data` to the file at `key` (treated as a filesystem path).
    ///
    /// Algorithm:
    /// 1. Serialise `data` (already a string — caller's responsibility).
    /// 2. Write to a UUID-named temp file in the **same directory** as `key`
    ///    (same filesystem, so `rename` cannot fail with EXDEV).
    /// 3. Flush + sync the temp file.
    /// 4. Atomically rename the temp file over `key`.
    ///
    /// If any step 2–4 fails the target file is never touched.
    fn write(&self, key: &str, data: &str) -> Result<(), SaveError> {
        let target = Path::new(key);

        // Determine parent directory; default to "." if the path has no parent.
        let dir = target.parent().unwrap_or_else(|| Path::new("."));

        // Create parent directories if they don't exist yet.
        std::fs::create_dir_all(dir).map_err(SaveError::Io)?;

        // Unique temp filename in the same directory.
        let temp_name = format!("{}.tmp", uuid::Uuid::new_v4());
        let temp_path = dir.join(temp_name);

        // Write to temp file.
        let write_result = (|| -> Result<(), SaveError> {
            let mut file = std::fs::File::create(&temp_path).map_err(SaveError::Io)?;
            file.write_all(data.as_bytes()).map_err(SaveError::Io)?;
            file.flush().map_err(SaveError::Io)?;
            file.sync_all().map_err(SaveError::Io)?;
            Ok(())
        })();

        if let Err(e) = write_result {
            // Best-effort cleanup; ignore secondary error.
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }

        // Atomic rename — on POSIX this is guaranteed atomic; on Windows it
        // replaces the destination if it already exists.
        if let Err(e) = std::fs::rename(&temp_path, target) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(SaveError::Io(e));
        }

        Ok(())
    }

    /// Read the file at `key` (treated as a filesystem path) into a `String`.
    fn read(&self, key: &str) -> Result<String, SaveError> {
        std::fs::read_to_string(key).map_err(SaveError::Io)
    }
}
