use crate::backend::StorageBackend;
use crate::error::SaveError;
use crate::models::Session;
#[cfg(target_arch = "wasm32")]
use crate::models::SessionId;
use crate::version::SAVE_FORMAT_VERSION;

// ---------------------------------------------------------------------------
// Shared deserialization pipeline (used by both backends)
// ---------------------------------------------------------------------------

/// Deserialize a session from a raw JSON string.
///
/// Two-phase approach:
/// 1. Parse to `serde_json::Value` to extract and validate the `version` field
///    before full deserialization — this keeps `VersionMismatch` errors precise
///    and distinct from structural corruption.
/// 2. Deserialize into `Session`.
/// 3. Validate `session_id`.
pub(crate) fn load_from_str(s: &str) -> Result<Session, SaveError> {
    let value: serde_json::Value =
        serde_json::from_str(s).map_err(|e| SaveError::Corrupted {
            message: format!("invalid JSON: {e}"),
        })?;
    load_from_value(value)
}

/// Deserialize a session from a `serde_json::Value`.
pub(crate) fn load_from_value(value: serde_json::Value) -> Result<Session, SaveError> {
    // --- Phase 1: extract and check version ---
    let version = value
        .get("version")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .ok_or_else(|| SaveError::Corrupted {
            message: "missing or non-integer 'version' field".to_string(),
        })?;

    if version != SAVE_FORMAT_VERSION {
        return Err(SaveError::VersionMismatch {
            found: version,
            expected: SAVE_FORMAT_VERSION,
        });
    }

    // --- Phase 2: full deserialization ---
    let session: Session =
        serde_json::from_value(value).map_err(|e| SaveError::Corrupted {
            message: format!("failed to deserialize session: {e}"),
        })?;

    // --- Phase 3: validate session_id ---
    // SessionId's TryFrom<String> already validates on deserialisation, but we
    // add an explicit empty-string check here for belt-and-suspenders clarity.
    if session.session_id.as_str().is_empty() {
        return Err(SaveError::Corrupted {
            message: "session_id field is absent or empty".to_string(),
        });
    }

    Ok(session)
}

// ---------------------------------------------------------------------------
// Native target: save_session / load_session
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
use crate::backend::native::NativeBackend;

/// Save `session` to the file at `path`.
///
/// Writes are **atomic**: the file is written to a temporary location in the
/// same directory first, then renamed over the target.  If the operation fails
/// at any point, any pre-existing file at `path` is left unchanged.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_session(session: &Session, path: &std::path::Path) -> Result<(), SaveError> {
    let json = serde_json::to_string(session).map_err(|e| SaveError::Corrupted {
        message: format!("failed to serialize session: {e}"),
    })?;
    let key = path.to_str().ok_or_else(|| SaveError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "path contains non-UTF-8 characters",
    )))?;
    NativeBackend.write(key, &json)
}

/// Load a session from the file at `path`.
///
/// Fails gracefully — never panics — returning the appropriate `SaveError`
/// variant for every failure mode:
/// - File not found / unreadable → `SaveError::Io`
/// - Invalid JSON or missing fields → `SaveError::Corrupted`
/// - Wrong schema version → `SaveError::VersionMismatch`
#[cfg(not(target_arch = "wasm32"))]
pub fn load_session(path: &std::path::Path) -> Result<Session, SaveError> {
    let key = path.to_str().ok_or_else(|| SaveError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "path contains non-UTF-8 characters",
    )))?;
    let raw = NativeBackend.read(key)?;
    load_from_str(&raw)
}

// ---------------------------------------------------------------------------
// WASM target: save_session_wasm / load_session_wasm
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
use crate::backend::wasm::WasmBackend;

/// Save `session` to `localStorage` using the `session_id` as the key.
#[cfg(target_arch = "wasm32")]
pub fn save_session_wasm(session: &Session) -> Result<(), SaveError> {
    let json = serde_json::to_string(session).map_err(|e| SaveError::Corrupted {
        message: format!("failed to serialize session: {e}"),
    })?;
    WasmBackend.write(session.session_id.as_str(), &json)
}

/// Load a session from `localStorage` by `session_id`.
#[cfg(target_arch = "wasm32")]
pub fn load_session_wasm(session_id: &SessionId) -> Result<Session, SaveError> {
    let raw = WasmBackend.read(session_id.as_str())?;
    load_from_str(&raw)
}
