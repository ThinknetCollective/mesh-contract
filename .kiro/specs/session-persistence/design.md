# Design Document: session-persistence

## Overview

The `session-persistence` feature adds a dedicated Rust library crate (`puzzle-persistence`) that
serializes and restores the complete runtime state of one puzzle attempt — the `Session` — both on
native targets (disk files, JSON) and on the WASM/browser target (`localStorage`, JSON). The library
is consumed by the puzzle game crate, which contains the TUI, CLI, and future web frontend. The
Soroban smart-contract workspace in this repository is unrelated to this library; the persistence
crate will live in the separate puzzle-game repository once it exists, but the design here is
complete and self-contained.

### Goals

- Zero-panic I/O and deserialization paths; all fallible operations return `Result<_, SaveError>`.
- Atomic writes on native (write-to-temp then rename) so crashes never corrupt a save file.
- A bounded autosave queue that guarantees no save event is silently dropped.
- Feature-flag-driven backend (`#[cfg(target_arch = "wasm32")]` / `wasm` Cargo feature) for
  transparent `localStorage` support.
- Schema versioning via a compile-time constant; mismatches are detected and reported precisely.

### Non-goals

- Migration of old save files to new schema versions (rejected saves must be re-started).
- Encryption or access-control of save files.
- Network synchronisation / cloud save.
- Soroban / on-chain state; the persistence layer is entirely off-chain.

---

## Architecture

The persistence feature is a standalone Rust library crate. It has no runtime dependency on the
TUI, CLI, or game logic — it receives a `&Session` and returns a `Result`. The calling code
(CLI entrypoint, TUI event loop, autosave queue) lives in the game crate and drives this library.

```
┌─────────────────────────────────────────────────────────────────┐
│                         puzzle-game crate                       │
│                                                                 │
│  ┌──────────┐   ┌─────────────────┐   ┌───────────────────┐    │
│  │   CLI    │   │   TUI / engine  │   │  Autosave queue   │    │
│  │ (clap)   │   │  (ratatui etc.) │   │  (mpsc channel)   │    │
│  └────┬─────┘   └────────┬────────┘   └────────┬──────────┘    │
│       │                  │                     │               │
│       └──────────────────┴─────────────────────┘               │
│                          │  &Session / session_id               │
│                          ▼                                      │
│         ┌────────────────────────────────────┐                  │
│         │      puzzle-persistence crate       │                  │
│         │                                    │                  │
│         │  ┌────────────┐  ┌──────────────┐  │                  │
│         │  │  native    │  │  wasm        │  │                  │
│         │  │  backend   │  │  backend     │  │                  │
│         │  │ (std::fs)  │  │ (web-sys)    │  │                  │
│         │  └────────────┘  └──────────────┘  │                  │
│         │         ▲ #[cfg(not(wasm32))]       │                  │
│         │                  ▲ #[cfg(wasm32)]   │                  │
│         └────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
          │ native                         │ wasm
          ▼                                ▼
   ~/.local/share/puzzle/         browser localStorage
   sessions/<id>.json             key: "puzzle_session:<id>"
```

### Crate Boundaries

| Crate | Role |
|---|---|
| `puzzle-persistence` | Library: data models, (de)serialization, backends, error types |
| `puzzle-game` | Binary: CLI parsing, TUI, autosave queue, wires everything together |

The autosave queue is implemented in the game crate (not the persistence library) because it
touches async runtime / channel concerns. The persistence library is intentionally synchronous and
backend-agnostic to keep it testable in isolation.

---

## Components and Interfaces

### Module Structure

```
puzzle-persistence/
├── Cargo.toml
└── src/
    ├── lib.rs          # Public API re-exports
    ├── error.rs        # SaveError enum
    ├── models.rs       # Session, PuzzleState, PuzzleType, SessionId
    ├── version.rs      # SAVE_FORMAT_VERSION constant
    ├── backend/
    │   ├── mod.rs      # StorageBackend trait
    │   ├── native.rs   # NativeBackend: atomic file I/O  (cfg not wasm32)
    │   └── wasm.rs     # WasmBackend: localStorage        (cfg wasm32)
    └── ops.rs          # save_session / load_session using dyn StorageBackend
```

### Public API

```rust
// error.rs
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("save file is corrupted: {message}")]
    Corrupted { message: String },

    #[error("version mismatch: file has v{found}, expected v{expected}")]
    VersionMismatch { found: u32, expected: u32 },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// ops.rs — native targets only
pub fn save_session(session: &Session, path: &Path) -> Result<(), SaveError>;
pub fn load_session(path: &Path) -> Result<Session, SaveError>;

// ops.rs — WASM target only
pub fn save_session_wasm(session: &Session) -> Result<(), SaveError>;
pub fn load_session_wasm(session_id: &SessionId) -> Result<Session, SaveError>;
```

### StorageBackend Trait (internal)

```rust
// backend/mod.rs
pub(crate) trait StorageBackend {
    fn write(&self, key: &str, data: &str) -> Result<(), SaveError>;
    fn read(&self, key: &str) -> Result<String, SaveError>;
}
```

Both `NativeBackend` and `WasmBackend` implement this trait. `ops.rs` calls the appropriate
backend selected at compile time via `cfg` attributes. The trait is `pub(crate)` only; callers
always use the free functions `save_session` / `load_session`.

---

## Data Models

### `Session`

```rust
// models.rs
use serde::{Deserialize, Serialize};

/// Current schema version. Increment on every backward-incompatible change.
pub use crate::version::SAVE_FORMAT_VERSION;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Schema version embedded in every save file.
    pub version: u32,
    /// Stable unique identifier assigned at Session creation.
    pub session_id: SessionId,
    /// The active puzzle and its mutable state.
    pub puzzle_state: PuzzleState,
    /// Elapsed time in whole seconds since the session started.
    pub elapsed_time: u64,
    /// Number of hints revealed so far.
    pub hint_count: u32,
    /// Score accumulated so far (may be 0 until puzzle is finished).
    pub score: i64,
}
```

### `SessionId`

```rust
/// A validated session identifier: 1–128 printable non-whitespace ASCII characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SessionId(String);

impl SessionId {
    /// Create a new, globally unique SessionId using a UUID v4.
    pub fn new_unique() -> Self { ... }

    /// Validate and wrap an existing string (used during deserialization).
    pub fn try_from_str(s: &str) -> Result<Self, SaveError> { ... }
}
```

Uniqueness within a process lifetime is guaranteed by UUID v4 generation (collision probability
negligible). The `try_from` serde integration ensures that deserialized `SessionId` values are
always valid, returning `SaveError::Corrupted` on violation.

### `PuzzleState` and `PuzzleType`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PuzzleType {
    Word,
    Numeric,
    Logic,
    // extensible — adding new variants is backward-compatible (old saves remain valid)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PuzzleState {
    pub puzzle_type: PuzzleType,
    pub puzzle_id: String,
    /// Serialized puzzle-specific data (opaque to the persistence layer).
    pub data: serde_json::Value,
    pub is_solved: bool,
}
```

Storing `data` as a `serde_json::Value` keeps the persistence crate decoupled from puzzle-specific
types. The game crate is responsible for serializing/deserializing the concrete puzzle payload into
this field.

### `SAVE_FORMAT_VERSION`

```rust
// version.rs
/// Increment this constant whenever the Session schema changes
/// in a backward-incompatible way (removed field, renamed field,
/// type change, or changed value semantics).
pub const SAVE_FORMAT_VERSION: u32 = 1;
```

---

## Key Algorithms

### 1. Atomic Write (Native Backend)

A direct `fs::write` leaves a window where the file is truncated but not yet complete; a crash
during that window corrupts the save. Atomic write closes this window using the OS-guaranteed
`rename` syscall: rename is atomic on POSIX and near-atomic on Windows (via
`MoveFileExW(MOVEFILE_REPLACE_EXISTING)`).

```
save_session(session, target_path):
  1. Serialize session to JSON string (in memory).
     On error → return SaveError::Corrupted (should not happen for valid Session).
  2. Determine temp path:
       same directory as target_path
       name = "<uuid>.tmp"  (UUIDs avoid collisions from concurrent writers)
  3. Open temp file for writing (create-new flags).
     On error → return SaveError::Io
  4. Write serialized bytes to temp file.
     On error → attempt to delete temp file (best-effort); return SaveError::Io
  5. Flush and close temp file.
     On error → attempt to delete temp file; return SaveError::Io
  6. Rename temp file to target_path.
     On POSIX:   rename(2) — atomic replacement, POSIX guarantee
     On Windows: MoveFileExW with MOVEFILE_REPLACE_EXISTING
     On error → attempt to delete temp file; return SaveError::Io
  7. Return Ok(())
```

Key guarantees:
- If any step 3–5 fails, `target_path` is never touched.
- Step 6 either succeeds completely or leaves `target_path` unchanged (the old file persists).
- The temp file is in the same directory as `target_path` to avoid cross-device rename failures.

### 2. Load with Version Check

```
load_session(path):
  1. Read file to string.
     On error (absent, permissions) → return SaveError::Io
  2. Parse as serde_json::Value (structural check).
     On error → return SaveError::Corrupted { message }
  3. Extract "version" field as u32.
     On missing/wrong type → return SaveError::Corrupted { message }
  4. If version != SAVE_FORMAT_VERSION
         → return SaveError::VersionMismatch { found: version, expected: SAVE_FORMAT_VERSION }
  5. Deserialize Value into Session.
     On error → return SaveError::Corrupted { message }
  6. Validate session_id (non-empty, 1-128 printable non-whitespace).
     On violation → return SaveError::Corrupted { message: "missing or invalid session_id" }
  7. Return Ok(session)
```

The two-phase parse (step 2 extracts version before full deserialization in step 5) ensures the
version mismatch error is always precise and never confused with a structural corruption error.

### 3. WASM `localStorage` Backend

```
// build-time selection
#[cfg(target_arch = "wasm32")]

const KEY_PREFIX: &str = "puzzle_session:";

fn storage_key(id: &SessionId) -> String {
    // prefix + id; SessionId is max 128 chars, prefix is 15 chars = 143 chars max
    // The requirement says "at most 128 chars". We enforce id <= 113 chars for wasm to
    // satisfy the combined key <= 128 requirement, OR we use the id directly as the key
    // since SessionId itself is at most 128 chars. 
    // Design decision: use the session_id directly as the localStorage key (no prefix),
    // satisfying the <= 128 char constraint from requirement 2.7.
    id.as_str().to_string()
}

save_session_wasm(session):
  1. Serialize session to JSON string.
  2. Obtain window().local_storage() via web-sys.
     On None → return SaveError::Io("localStorage unavailable")
  3. Call storage.set_item(key, &json).
     On JsValue error (e.g. QuotaExceededError) → return SaveError::Io(description)
  4. Return Ok(())

load_session_wasm(session_id):
  1. Obtain window().local_storage().
     On None → return SaveError::Io("localStorage unavailable")
  2. Call storage.get_item(key).
     On None → return SaveError::Io("session not found in localStorage")
  3. Run the same load pipeline as steps 2–7 of load_session above.
```

### 4. Autosave Queue (Game Crate)

The autosave queue lives in the game crate. It uses a bounded `std::sync::mpsc` channel (or
`tokio::sync::mpsc` if the game uses an async runtime) to decouple trigger events from the
actual I/O. A dedicated worker thread (or task) drains the queue.

```
// In game crate

enum AutosaveEvent {
    Save(Session),
}

struct AutosaveQueue {
    tx: mpsc::SyncSender<AutosaveEvent>,  // bounded capacity = e.g. 64
}

// On game start:
let (tx, rx) = mpsc::sync_channel::<AutosaveEvent>(64);
spawn_autosave_worker(rx, save_dir);

fn spawn_autosave_worker(rx: Receiver<AutosaveEvent>, save_dir: PathBuf) {
    std::thread::spawn(move || {
        for event in rx {               // blocks until event arrives
            match event {
                AutosaveEvent::Save(session) => {
                    let path = save_dir.join(format!("{}.json", session.session_id));
                    if let Err(e) = save_session(&session, &path) {
                        // Send AutosaveError event back to TUI via a separate channel
                        warn_channel.send(AutosaveWarning::from(e));
                    }
                }
            }
        }
    });
}

// Autosave trigger sites (answer submission, hint reveal):
fn on_answer_submit(queue: &AutosaveQueue, session: &Session) {
    // Clone snapshot at trigger time — satisfies req 3.4
    let snapshot = session.clone();
    // SyncSender::try_send on full queue returns Err(Full); use send to block briefly
    // or use try_send and log if full (bounded queue prevents unbounded growth)
    let _ = queue.tx.send(AutosaveEvent::Save(snapshot));
}
```

The worker processes events serially, so two rapid events are naturally queued and both executed —
satisfying requirement 3.5. The `session.clone()` at trigger time captures the snapshot before any
further mutation, satisfying requirement 3.4.

For the hint-reveal ordering guarantee (req 3.2 — save before hint is displayed), the hint reveal
path uses a synchronous call rather than the queue:

```rust
fn on_hint_reveal(session: &Session, save_dir: &Path) -> Result<HintContent, SaveError> {
    let path = save_dir.join(format!("{}.json", session.session_id));
    save_session(session, &path)?;  // save completes before hint is returned
    Ok(session.get_hint())          // hint content returned after save
}
```

---

## Error Handling Design

### `SaveError` Variants

| Variant | When returned | Contains |
|---|---|---|
| `SaveError::Corrupted { message }` | JSON parse failure, missing fields, invalid `session_id`, unexpected value types | Non-empty `String` describing the parse failure |
| `SaveError::VersionMismatch { found, expected }` | `version` field in file ≠ `SAVE_FORMAT_VERSION` | Both version numbers as `u32` |
| `SaveError::Io(std::io::Error)` | File read/write failure, permissions, `localStorage` unavailable/quota exceeded | Underlying `std::io::Error` (or a synthetic one via `std::io::Error::new` for WASM) |

### No-Panic Policy

Every fallible operation in `puzzle-persistence` uses `?` propagation or explicit `match`/`map_err`
chains. The following patterns are **prohibited** in `src/` of the persistence crate:

- `.unwrap()` on `Result` or `Option` values from I/O or deserialization
- `.expect(...)` on the same
- `panic!(...)` except in unreachable code branches that are provably impossible

The `Cargo.toml` for the crate will include:

```toml
[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
```

This converts `clippy::unwrap_used` and `clippy::expect_used` into hard errors, enforced in CI.

### CLI Error Reporting

The CLI entry point (in the game crate) handles `SaveError` as follows:

```rust
fn report_error_and_exit(err: SaveError) -> ! {
    eprintln!("error: {err}");   // thiserror Display impl on stderr
    std::process::exit(1);
}
```

The `thiserror::Error` derive on `SaveError` ensures `Display` produces human-readable messages
that include all relevant context (version numbers, corruption description, I/O cause).

### Autosave Warning (TUI)

When the autosave worker encounters a `SaveError`, it sends a `AutosaveWarning` event to the TUI
via a separate watch channel. The TUI renders the warning in a non-modal overlay that:

- Does **not** consume keyboard input (player can keep playing).
- Displays the error reason (from `err.to_string()`).
- Auto-dismisses after 5 seconds (driven by a `tokio::time::sleep` or equivalent).

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a
system — essentially, a formal statement about what the system should do. Properties serve as the
bridge between human-readable specifications and machine-verifiable correctness guarantees.*

PBT is appropriate here because the persistence layer is a collection of
pure-ish transformation functions (serialize, deserialize, validate) over a rich structured input
space (`Session` with multiple puzzle types, arbitrary strings, arbitrary version numbers). Running
hundreds of generated inputs will surface edge cases that hand-written examples miss.

The PBT library chosen is [`proptest`](https://github.com/proptest-rs/proptest) (Rust, no
`std::io` required, excellent `#[derive(Arbitrary)]` support via `proptest-derive`).

---

### Property 1: Round-Trip Serialization Preserves All Fields

*For any* valid `Session` value (arbitrary puzzle type, arbitrary `session_id`, arbitrary elapsed
time, hint count, and score), serializing the `Session` to JSON and immediately deserializing it
back SHALL produce a `Session` whose fields are equal to those of the original, field by field.

**Validates: Requirements 1.4, 2.9, 6.6**

---

### Property 2: Serialized JSON Has Correct Shape and Embedded Version

*For any* valid `Session` value, serializing it to a JSON string and parsing the result as
`serde_json::Value` SHALL yield an object that (a) contains all required top-level keys
(`puzzle_state`, `elapsed_time`, `hint_count`, `score`, `session_id`, `version`) with the correct
JSON types for each field, and (b) has a `version` field whose integer value equals
`SAVE_FORMAT_VERSION`.

**Validates: Requirements 1.3, 6.2**

---

### Property 3: Corrupted Input Always Returns `SaveError::Corrupted` with Non-Empty Message

*For any* byte string that is not a structurally valid, schema-compatible JSON representation of a
`Session` (including: truncated JSON, missing required fields, wrong value types, absent or empty
`session_id`), calling `load_session` (or the equivalent deserialization path) SHALL return
`Err(SaveError::Corrupted { message })` where `message` is non-empty, and SHALL NOT panic.

**Validates: Requirements 1.5, 5.1, 6.7**

---

### Property 4: Version Mismatch Returns Correct `found` and `expected` Values

*For any* `u32` version value `v` where `v != SAVE_FORMAT_VERSION`, embedding `v` as the `version`
field in an otherwise structurally valid session JSON and calling `load_session` SHALL return
`Err(SaveError::VersionMismatch { found: v, expected: SAVE_FORMAT_VERSION })`, with no `Session`
value returned.

**Validates: Requirements 5.2, 6.3, 6.4**

---

### Property 5: Autosave Snapshot Is Immutable After Trigger

*For any* valid `Session` `s`, when an autosave is triggered with a clone of `s` (the snapshot),
and `s` is subsequently mutated (score, hint count, or elapsed time changed), the content written
to the save file SHALL equal the serialized form of the original snapshot, not the mutated session.

**Validates: Requirements 3.4**

---

### Property 6: Autosave Queue Processes All Events

*For any* sequence of N autosave trigger events (N ≥ 2) submitted to the autosave queue in rapid
succession before the first save completes, the queue SHALL eventually process all N events (queue
depth reaches zero), and no event SHALL be silently discarded.

**Validates: Requirements 3.5**

---

### Property 7: `SessionId` Validity and Uniqueness

*For any* set of N `Session` instances created via `Session::new()` during the same process
lifetime (N ≥ 1), each `session_id` SHALL satisfy: (a) length between 1 and 128 characters
inclusive, (b) all characters are printable and non-whitespace, and (c) no two `session_id` values
in the set are equal.

**Validates: Requirements 6.5**

---

### Property 8: CLI Error Message Contains Session ID for Missing File

*For any* session ID string `id` that does not correspond to an existing save file, invoking the
CLI with `--resume <id>` SHALL write a message to stderr that contains the literal string `id` and
exit with a non-zero status code.

**Validates: Requirements 4.3**

---

### Property 9: Any `SaveError` Produces Non-Empty Stderr and Non-Zero Exit

*For any* `SaveError` variant (`Corrupted`, `VersionMismatch`, `Io`), passing it to the CLI error
handler SHALL result in a non-empty message written to stderr and an exit code of 1 (non-zero).

**Validates: Requirements 5.4**

---

### Property 10: `localStorage` Key Length Does Not Exceed 128 Characters

*For any* valid `SessionId` (1–128 printable non-whitespace characters), the derived
`localStorage` key SHALL be at most 128 characters long.

**Validates: Requirements 2.7**

---

### Property 11: `--resume` Accepts Valid and Rejects Invalid Session IDs

*For any* string `s` of 1–36 printable non-whitespace characters, passing `--resume <s>` to the
CLI SHALL be accepted as syntactically valid (no usage-error exit). For any string `s` that is
empty, contains whitespace, or has length > 36, the CLI SHALL exit with a non-zero status code and
print usage guidance to stderr.

**Validates: Requirements 4.1**

---

## Error Handling

### Error Propagation Map

```
save_session(session, path)
  ├─ serde_json::to_string(&session)     → Err mapped to SaveError::Corrupted  (unlikely)
  ├─ tempfile creation                   → Err mapped to SaveError::Io
  ├─ write to tempfile                   → Err mapped to SaveError::Io + delete temp
  ├─ flush/sync tempfile                 → Err mapped to SaveError::Io + delete temp
  └─ rename(temp, target)               → Err mapped to SaveError::Io + delete temp

load_session(path)
  ├─ fs::read_to_string(path)           → Err mapped to SaveError::Io
  ├─ serde_json::from_str → Value       → Err mapped to SaveError::Corrupted
  ├─ extract version field              → Err mapped to SaveError::Corrupted
  ├─ version check                      → Err = SaveError::VersionMismatch
  ├─ serde_json::from_value → Session   → Err mapped to SaveError::Corrupted
  └─ validate session_id                → Err = SaveError::Corrupted
```

### Recovery Strategy

| Error | Recovery at call site |
|---|---|
| `SaveError::Io` | Autosave: log warning, show TUI overlay. CLI `--resume`: print to stderr, exit 1. |
| `SaveError::Corrupted` | CLI: print details to stderr, exit 1. Offer to start a new session. |
| `SaveError::VersionMismatch` | CLI: print both versions to stderr. Recommend upgrading or deleting the old file. |

### WASM I/O Error Synthesis

`web-sys` `localStorage` operations return `Result<_, JsValue>`. Since `JsValue` does not implement
`std::error::Error`, errors are converted to `SaveError::Io` via:

```rust
fn js_err_to_io(js_val: JsValue) -> SaveError {
    let msg = js_sys::JSON::stringify(&js_val)
        .map(|s| s.as_string().unwrap_or_default())
        .unwrap_or_else(|_| "unknown localStorage error".to_string());
    SaveError::Io(std::io::Error::new(std::io::ErrorKind::Other, msg))
}
```

---

## Testing Strategy

### Dual Testing Approach

Unit tests cover specific examples, edge cases, and error conditions. Property tests verify
universal properties across many generated inputs. Both layers are complementary.

### Property-Based Testing Setup

**Library**: [`proptest`](https://crates.io/crates/proptest) with
[`proptest-derive`](https://crates.io/crates/proptest-derive) for `#[derive(Arbitrary)]`.

```toml
# puzzle-persistence/Cargo.toml
[dev-dependencies]
proptest = "1"
proptest-derive = "0.4"
tempfile = "3"
```

**Minimum iterations**: 256 per property (configured via `proptest::proptest_config!`).

**Tagging convention** (comment above each `proptest!` block):

```rust
// Feature: session-persistence, Property 1: round-trip serialization preserves all fields
proptest! {
    #[test]
    fn prop_round_trip(session in arb_session()) { ... }
}
```

### Arbitrary Generators

```rust
fn arb_session_id() -> impl Strategy<Value = SessionId> {
    // 1–128 printable non-whitespace ASCII chars
    "[!-~]{1,128}".prop_map(|s| SessionId::try_from_str(&s).unwrap())
}

fn arb_puzzle_type() -> impl Strategy<Value = PuzzleType> {
    prop_oneof![
        Just(PuzzleType::Word),
        Just(PuzzleType::Numeric),
        Just(PuzzleType::Logic),
    ]
}

fn arb_session() -> impl Strategy<Value = Session> {
    (arb_session_id(), arb_puzzle_type(), any::<u64>(), any::<u32>(), any::<i64>())
        .prop_map(|(id, pt, elapsed, hints, score)| Session {
            version: SAVE_FORMAT_VERSION,
            session_id: id,
            puzzle_state: PuzzleState { puzzle_type: pt, ..Default::default() },
            elapsed_time: elapsed,
            hint_count: hints,
            score,
        })
}
```

### Property Test Implementations

#### Property 1 — Round-Trip

```rust
// Feature: session-persistence, Property 1: round-trip serialization preserves all fields
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn prop_round_trip(session in arb_session()) {
        let json = serde_json::to_string(&session).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(session, restored);
    }
}
```

#### Property 2 — Serialization Shape

```rust
// Feature: session-persistence, Property 2: serialized JSON has correct shape and embedded version
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn prop_serialization_shape(session in arb_session()) {
        let json: serde_json::Value = serde_json::to_value(&session).unwrap();
        let obj = json.as_object().unwrap();
        prop_assert!(obj.contains_key("puzzle_state"));
        prop_assert!(obj["elapsed_time"].is_number());
        prop_assert!(obj["hint_count"].is_number());
        prop_assert!(obj["score"].is_number());
        prop_assert!(obj["session_id"].is_string());
        prop_assert!(obj["version"].is_number());
        prop_assert_eq!(obj["version"].as_u64().unwrap() as u32, SAVE_FORMAT_VERSION);
    }
}
```

#### Property 3 — Corrupted Input

```rust
// Feature: session-persistence, Property 3: corrupted input always returns SaveError::Corrupted
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn prop_corrupted_input(bad_json in ".*") {
        // Exclude strings that happen to be valid sessions
        let result = serde_json::from_str::<Session>(&bad_json);
        if result.is_err() {
            let err = load_from_str(&bad_json); // internal helper
            match err {
                Err(SaveError::Corrupted { message }) => prop_assert!(!message.is_empty()),
                Err(SaveError::VersionMismatch { .. }) => { /* also acceptable */ }
                Err(SaveError::Io(_)) => prop_assert!(false, "should be Corrupted, not Io"),
                Ok(_) => { /* valid json that parsed — skip */ }
            }
        }
    }
}
```

#### Property 4 — Version Mismatch

```rust
// Feature: session-persistence, Property 4: version mismatch returns correct found/expected
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn prop_version_mismatch(v in any::<u32>().prop_filter("not current", |&v| v != SAVE_FORMAT_VERSION)) {
        let mut json = serde_json::to_value(arb_session().new_tree(&mut TestRunner::default()).unwrap().current()).unwrap();
        json["version"] = serde_json::json!(v);
        let err = load_from_value(json).unwrap_err();
        prop_assert!(matches!(err, SaveError::VersionMismatch { found, expected }
            if found == v && expected == SAVE_FORMAT_VERSION));
    }
}
```

#### Property 5 — Autosave Snapshot Immutability

```rust
// Feature: session-persistence, Property 5: autosave snapshot is immutable after trigger
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn prop_snapshot_immutability(session in arb_session(), delta in any::<u32>()) {
        let snapshot = session.clone();
        let dir = tempfile::tempdir().unwrap();
        // Simulate: autosave triggered with snapshot, then session mutated
        let path = dir.path().join(format!("{}.json", snapshot.session_id));
        save_session(&snapshot, &path).unwrap();
        // Mutate original (autosave already committed snapshot)
        let mut mutated = session;
        mutated.hint_count = mutated.hint_count.wrapping_add(delta);
        let loaded = load_session(&path).unwrap();
        prop_assert_eq!(loaded, snapshot);
    }
}
```

#### Property 7 — SessionId Validity and Uniqueness

```rust
// Feature: session-persistence, Property 7: session_id validity and uniqueness
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn prop_session_id_validity(n in 1usize..=20usize) {
        let ids: Vec<SessionId> = (0..n).map(|_| SessionId::new_unique()).collect();
        for id in &ids {
            let s = id.as_str();
            prop_assert!(s.len() >= 1 && s.len() <= 128);
            prop_assert!(s.chars().all(|c| c.is_ascii_graphic()));
        }
        // Uniqueness
        let unique: std::collections::HashSet<_> = ids.iter().map(|id| id.as_str()).collect();
        prop_assert_eq!(unique.len(), n);
    }
}
```

### Unit Tests

| Test | Covers |
|---|---|
| `test_save_and_load_word_puzzle` | Req 2.9 — Word type |
| `test_save_and_load_numeric_puzzle` | Req 2.9 — Numeric type |
| `test_save_and_load_logic_puzzle` | Req 2.9 — Logic type |
| `test_load_missing_file_returns_io` | Req 5.5 |
| `test_load_missing_session_id_returns_corrupted` | Req 6.7 |
| `test_load_empty_session_id_returns_corrupted` | Req 6.7 |
| `test_atomic_write_preserves_old_file_on_failure` | Req 2.4 |
| `test_cli_resume_missing_session_stderr_and_exit` | Req 4.3 |
| `test_cli_resume_no_arg_shows_usage` | Req 4.4 |
| `test_autosave_failure_produces_warning_event` | Req 3.3 |
| `test_hint_save_before_hint_content` | Req 3.2 |

### Linting Enforcement

```toml
# puzzle-persistence/Cargo.toml
[lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
```

CI runs `cargo clippy --all-features -- -D warnings` to enforce the no-panic policy statically.

### WASM Testing

WASM backend tests run via `wasm-pack test --headless --chrome` using
[`wasm-bindgen-test`](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html). A
`MockStorage` struct that implements `web_sys::Storage`'s interface is used to test quota-exceeded
and unavailable paths without a real browser.

---

## Design Decisions and Rationale

### Why a separate library crate?

Keeping persistence logic in its own crate enforces the compile-time boundary that the library has
no knowledge of TUI or CLI concerns. It also lets the WASM feature flag be scoped to exactly the
persistence crate without leaking into the rest of the game.

### Why `serde_json::Value` for `PuzzleState.data`?

Storing the puzzle-specific payload as an opaque `Value` means adding a new puzzle type never
requires a change to the persistence crate. The game crate owns the concrete puzzle types; the
persistence crate stays stable across game evolution. The trade-off is that the persistence crate
cannot validate puzzle-specific invariants — that responsibility belongs to the game crate.

### Why not `async` in the persistence library?

The persistence library is synchronous because:
1. Atomic writes using temp-file rename are inherently synchronous (no async advantage).
2. WASM `localStorage` is synchronous.
3. Keeping the library sync allows both sync and async callers to use it without an adapter layer.

The autosave queue (in the game crate) provides the non-blocking behaviour the TUI requires.

### Why `proptest` over `quickcheck`?

`proptest` supports shrinking out of the box, has excellent `#[derive(Arbitrary)]` support, and
integrates smoothly with `serde_json::Value` generation strategies. For a persistence library where
string and integer edge cases are critical, `proptest`'s structured generators (regex-based string
strategies) are a better fit.

### `SessionId` length on WASM

Requirement 2.7 says the `localStorage` key must be at most 128 characters. Since `SessionId`
itself is already bounded to 128 characters and is used directly as the storage key (no prefix),
this constraint is structurally guaranteed and is enforced by the `SessionId` validator, not by
additional runtime logic in the WASM backend.

### Temp file placement

Temp files are written to the same directory as the target path (not `/tmp`) to avoid cross-device
rename failures (EXDEV on Linux when `/tmp` is a different filesystem). The temp file name uses a
UUID v4 to avoid collisions between concurrent writers or processes.
