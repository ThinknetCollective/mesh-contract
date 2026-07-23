use thiserror::Error;

/// All errors that the persistence module can return.
///
/// No operation in this crate calls `unwrap()` or `expect()` on I/O or
/// deserialization results — every fallible path propagates through this type.
#[derive(Debug, Error)]
pub enum SaveError {
    /// The save file could not be parsed: invalid JSON, missing required fields,
    /// wrong value types, or an absent/empty `session_id`.
    #[error("save file is corrupted: {message}")]
    Corrupted { message: String },

    /// The save file was written by a different schema version.
    /// `found` is the version embedded in the file; `expected` is the current
    /// `SAVE_FORMAT_VERSION` compiled into this binary.
    #[error("version mismatch: save file has v{found}, this binary expects v{expected}")]
    VersionMismatch { found: u32, expected: u32 },

    /// An I/O error occurred while reading or writing the save file, or while
    /// accessing `localStorage` on WASM targets.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
