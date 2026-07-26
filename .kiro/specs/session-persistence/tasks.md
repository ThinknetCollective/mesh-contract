# Implementation Plan: session-persistence

## Overview

Implement the `puzzle-persistence` standalone Rust library crate and integrate it into the
`puzzle-game` crate. Work proceeds in eleven logical phases: crate scaffold → data models →
error types → native backend → WASM backend → public ops API → autosave queue → CLI `--resume`
flag → TUI warning overlay → unit tests → property-based tests. Each phase builds directly on
the previous ones so no orphaned code is left unintegrated.

---

## Tasks

- [ ] 1. Scaffold the `puzzle-persistence` crate
  - [ ] 1.1 Create the crate directory and `Cargo.toml`
    - Create `puzzle-persistence/Cargo.toml` with `[lib]`, `edition = "2021"`, and the following
      dependencies: `serde = { version = "1", features = ["derive"] }`,
      `serde_json = "1"`, `thiserror = "1"`, `uuid = { version = "1", features = ["v4"] }`;
      add dev-dependencies `proptest = "1"`, `proptest-derive = "0.4"`, `tempfile = "3"`;
      add optional dependency `web-sys = { version = "0.3", features = ["Storage", "Window"],
      optional = true }` and `js-sys = { version = "0.3", optional = true }` gated on the
      `wasm` feature flag.
    - Add the `[lints.clippy]` section: `unwrap_used = "deny"`, `expect_used = "deny"`.
    - Add the crate to the workspace `Cargo.toml` members list.
    - _Requirements: 5.3_

  - [ ] 1.2 Create the module skeleton (`lib.rs` + empty module files)
    - Create `puzzle-persistence/src/lib.rs` with `pub mod error; pub mod models; pub mod version;
      mod backend; pub mod ops;` and re-export the public surface:
      `pub use error::SaveError; pub use models::{Session, SessionId, PuzzleState, PuzzleType};
      pub use version::SAVE_FORMAT_VERSION; pub use ops::*;`
    - Create empty placeholder files: `src/error.rs`, `src/models.rs`, `src/version.rs`,
      `src/backend/mod.rs`, `src/backend/native.rs`, `src/backend/wasm.rs`, `src/ops.rs`.
    - Confirm `cargo check -p puzzle-persistence` compiles with zero errors.
    - _Requirements: 5.3_

- [ ] 2. Implement data models
  - [ ] 2.1 Define `SAVE_FORMAT_VERSION` constant in `version.rs`
    - Write `pub const SAVE_FORMAT_VERSION: u32 = 1;` with a doc-comment explaining the
      increment policy (removing/renaming a field, type change, changed semantics).
    - _Requirements: 6.1, 6.2_

  - [ ] 2.2 Implement `PuzzleType` and `PuzzleState` in `models.rs`
    - Define `PuzzleType` enum with variants `Word`, `Numeric`, `Logic`; derive
      `Debug, Clone, PartialEq, Serialize, Deserialize`.
    - Define `PuzzleState` struct with fields `puzzle_type: PuzzleType`, `puzzle_id: String`,
      `data: serde_json::Value`, `is_solved: bool`; derive the same traits plus `Default`.
    - _Requirements: 1.3, 2.9_

  - [ ] 2.3 Implement `SessionId` newtype in `models.rs`
    - Define `SessionId(String)` with `#[serde(try_from = "String", into = "String")]`; derive
      `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`.
    - Implement `SessionId::new_unique() -> Self` using `uuid::Uuid::new_v4().to_string()`.
    - Implement `SessionId::try_from_str(s: &str) -> Result<Self, SaveError>` that validates
      length 1–128 and all characters are printable non-whitespace (`!c.is_whitespace() &&
      c.is_ascii_graphic()`), returning `SaveError::Corrupted` on violation.
    - Implement `TryFrom<String>` (delegates to `try_from_str`) and `From<SessionId>` for
      `String` to satisfy the serde bridge.
    - Implement `SessionId::as_str(&self) -> &str`.
    - _Requirements: 6.5, 6.6, 6.7, 2.7_

  - [ ] 2.4 Implement `Session` struct in `models.rs`
    - Define `Session` with fields `version: u32`, `session_id: SessionId`,
      `puzzle_state: PuzzleState`, `elapsed_time: u64`, `hint_count: u32`, `score: i64`; derive
      `Debug, Clone, PartialEq, Serialize, Deserialize`.
    - _Requirements: 1.1, 1.2, 1.3_

- [ ] 3. Implement `SaveError` in `error.rs`
  - [ ] 3.1 Define the `SaveError` enum with `thiserror` derives
    - Write `Corrupted { message: String }` with display `"save file is corrupted: {message}"`.
    - Write `VersionMismatch { found: u32, expected: u32 }` with display
      `"version mismatch: file has v{found}, expected v{expected}"`.
    - Write `Io(#[from] std::io::Error)` with display `"I/O error: {0}"`.
    - Derive `Debug` and `thiserror::Error` on the enum.
    - _Requirements: 1.5, 2.4, 2.6, 2.8, 5.1, 5.2, 5.4, 5.5_

- [ ] 4. Implement the native storage backend
  - [ ] 4.1 Define `StorageBackend` internal trait in `backend/mod.rs`
    - Write `pub(crate) trait StorageBackend { fn write(&self, key: &str, data: &str) ->
      Result<(), SaveError>; fn read(&self, key: &str) -> Result<String, SaveError>; }`
    - _Requirements: 2.1, 2.2_

  - [ ] 4.2 Implement `NativeBackend` with atomic write in `backend/native.rs`
    - Gate the entire file with `#[cfg(not(target_arch = "wasm32"))]`.
    - Implement `StorageBackend::write` using the atomic temp-file-then-rename algorithm:
      (1) compute temp path in the same directory using a UUID v4 name; (2) create and write
      the temp file; (3) flush and sync; (4) rename to target; clean up temp on any error;
      never touch the target path until the rename step succeeds.
    - Implement `StorageBackend::read` using `std::fs::read_to_string`, mapping `io::Error`
      to `SaveError::Io`.
    - Ensure no `.unwrap()` or `.expect()` calls appear; all errors propagate via `?` or
      explicit `map_err`.
    - _Requirements: 2.3, 2.4, 5.3, 5.5_

- [ ] 5. Implement the WASM `localStorage` backend
  - [ ] 5.1 Implement `WasmBackend` in `backend/wasm.rs`
    - Gate the entire file with `#[cfg(target_arch = "wasm32")]`.
    - Implement `StorageBackend::write`: obtain `window().local_storage()`, call
      `storage.set_item(key, data)`, convert any `JsValue` error to `SaveError::Io` via the
      `js_err_to_io` helper.
    - Implement `StorageBackend::read`: obtain `window().local_storage()`, call
      `storage.get_item(key)`, return `SaveError::Io("session not found in localStorage")`
      when the result is `None`.
    - Implement the `js_err_to_io(js_val: JsValue) -> SaveError` helper that stringifies the
      `JsValue` and wraps it in `std::io::Error::new(std::io::ErrorKind::Other, msg)`.
    - Use `SessionId::as_str()` directly as the storage key (no prefix) — this satisfies the
      ≤128-char constraint from Req 2.7 because `SessionId` is already bounded to 128 chars.
    - Ensure no `.unwrap()` / `.expect()` calls.
    - _Requirements: 2.7, 2.8, 5.3_

- [ ] 6. Implement the public ops API in `ops.rs`
  - [ ] 6.1 Implement `save_session` and `load_session` for native targets
    - Gate both functions with `#[cfg(not(target_arch = "wasm32"))]`.
    - `save_session(session: &Session, path: &Path) -> Result<(), SaveError>`: serialize to
      JSON via `serde_json::to_string`, delegate write to `NativeBackend`, using the path's
      string representation as the key.
    - `load_session(path: &Path) -> Result<Session, SaveError>`: delegate read to
      `NativeBackend`, then run the two-phase load pipeline:
      (1) parse as `serde_json::Value`; (2) extract and check `version` field against
      `SAVE_FORMAT_VERSION` — return `SaveError::VersionMismatch` on mismatch; (3) deserialize
      into `Session`; (4) validate `session_id` is non-empty; return `SaveError::Corrupted`
      with a non-empty message at each failure point.
    - Also expose a crate-internal `load_from_str(s: &str) -> Result<Session, SaveError>` and
      `load_from_value(v: serde_json::Value) -> Result<Session, SaveError>` helpers (used by
      property tests).
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 5.1, 5.2, 5.3, 5.5, 6.2, 6.3, 6.4, 6.7_

  - [ ] 6.2 Implement `save_session_wasm` and `load_session_wasm` for WASM targets
    - Gate both functions with `#[cfg(target_arch = "wasm32")]`.
    - `save_session_wasm(session: &Session) -> Result<(), SaveError>`: serialize to JSON,
      delegate to `WasmBackend` using `session.session_id.as_str()` as the key.
    - `load_session_wasm(session_id: &SessionId) -> Result<Session, SaveError>`: delegate read
      to `WasmBackend`, then run the same two-phase load pipeline as `load_session`.
    - _Requirements: 2.7, 2.8, 5.1, 5.2, 5.3, 6.3, 6.4_

- [ ] 7. Checkpoint — verify native compilation and basic behavior
  - Run `cargo build -p puzzle-persistence` and `cargo clippy -p puzzle-persistence -- -D warnings`
    and confirm zero errors and zero `unwrap_used`/`expect_used` violations.
  - _Ensure all tests pass, ask the user if questions arise._

- [ ] 8. Implement the autosave queue in the game crate
  - [ ] 8.1 Define `AutosaveEvent`, `AutosaveWarning`, and `AutosaveQueue` types
    - In the game crate (e.g., `src/autosave.rs`), define:
      `enum AutosaveEvent { Save(Session) }`
      `struct AutosaveWarning { pub message: String, pub triggered_at: std::time::Instant }`
      `struct AutosaveQueue { tx: std::sync::mpsc::SyncSender<AutosaveEvent> }`
    - _Requirements: 3.5_

  - [ ] 8.2 Implement `spawn_autosave_worker` and queue lifecycle
    - Implement `AutosaveQueue::new(save_dir: PathBuf, warn_tx: std::sync::mpsc::Sender<AutosaveWarning>) -> Self`
      that creates a bounded `sync_channel(64)` and spawns a worker thread.
    - The worker thread loops on `rx` (serial drain), calls `save_session(&session, &path)` for
      each event, and on error sends an `AutosaveWarning` through `warn_tx`.
    - _Requirements: 3.1, 3.3, 3.5_

  - [ ] 8.3 Implement `on_answer_submit` and `on_hint_reveal` hooks
    - `on_answer_submit(queue: &AutosaveQueue, session: &Session)`: clone `session` to capture
      a snapshot at trigger time, then call `queue.tx.send(AutosaveEvent::Save(snapshot))`.
    - `on_hint_reveal(session: &Session, save_dir: &Path) -> Result<HintContent, SaveError>`:
      call `save_session(session, &path)?` synchronously (before returning hint content), then
      return the hint. This guarantees save-before-display for hint reveals (Req 3.2).
    - _Requirements: 3.1, 3.2, 3.4, 3.5_

- [ ] 9. Implement the `--resume <session_id>` CLI flag
  - [ ] 9.1 Add `--resume` argument to the clap CLI definition
    - In the game crate's CLI struct (or `Args` derive), add:
      `#[arg(long, value_name = "session_id")] resume: Option<String>`
    - _Requirements: 4.1, 4.4_

  - [ ] 9.2 Implement the resume dispatch logic
    - After argument parsing, if `args.resume` is `Some(id)`:
      (a) Validate `id` is non-empty and 1–36 printable non-whitespace characters; on
          violation print usage guidance `"usage: --resume <session_id>"` to stderr and
          `std::process::exit(1)`.
      (b) Construct the save file path, call `load_session(&path)`.
      (c) On `Err(e)` call `report_error_and_exit(e, &id)` which writes a human-readable
          message containing the `session_id` string and the error to stderr, then exits 1.
      (d) On `Ok(session)` launch the TUI with the restored state.
    - Implement `report_error_and_exit(err: &SaveError, id: &str) -> !` that writes
      `"error: {err}"` (or richer context for `VersionMismatch`) to stderr and calls
      `std::process::exit(1)`.
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 5.4_

- [ ] 10. Implement the TUI autosave warning overlay
  - [ ] 10.1 Add the warning state and render logic to the TUI
    - In the TUI state struct add:
      `autosave_warning: Option<(String, std::time::Instant)>`
    - In the event loop, check the `warn_rx` channel for new `AutosaveWarning` messages each
      tick; on receipt set `autosave_warning = Some((warning.message, Instant::now()))`.
    - In the render function: when `autosave_warning` is `Some((msg, at))` and
      `at.elapsed() < Duration::from_secs(5)`, render a non-modal overlay widget (e.g., a
      bottom-right `Paragraph` block) displaying `"⚠ Autosave failed: {msg}"` that does not
      consume keyboard input.
    - Clear `autosave_warning` once elapsed ≥ 5 s (auto-dismiss).
    - _Requirements: 3.3_

- [ ] 11. Unit tests (11 named tests)
  - [ ] 11.1 Write unit tests for save/load across all three puzzle types
    - `test_save_and_load_word_puzzle`: create a `Session` with `PuzzleType::Word`, save to a
      `tempfile` dir, load, assert all fields equal. _Requirements: 2.9_
    - `test_save_and_load_numeric_puzzle`: same for `PuzzleType::Numeric`. _Requirements: 2.9_
    - `test_save_and_load_logic_puzzle`: same for `PuzzleType::Logic`. _Requirements: 2.9_

  - [ ] 11.2 Write unit tests for error conditions
    - `test_load_missing_file_returns_io`: call `load_session` on a non-existent path; assert
      `Err(SaveError::Io(_))`. _Requirements: 5.5_
    - `test_load_missing_session_id_returns_corrupted`: construct JSON with `session_id` key
      absent; call `load_from_str`; assert `Err(SaveError::Corrupted { .. })` with non-empty
      message. _Requirements: 6.7_
    - `test_load_empty_session_id_returns_corrupted`: construct JSON with `session_id = ""`; 
      call `load_from_str`; assert `Err(SaveError::Corrupted { .. })`. _Requirements: 6.7_
    - `test_atomic_write_preserves_old_file_on_failure`: write a valid save file to a path,
      then attempt to save to a read-only path (or simulate via `NativeBackend` with a bad
      directory); assert that the original file content is unchanged and the function returns
      `Err(SaveError::Io(_))`. _Requirements: 2.4_

  - [ ] 11.3 Write unit tests for CLI and autosave error paths
    - `test_cli_resume_missing_session_stderr_and_exit`: invoke the resume dispatch logic with
      a session ID that has no matching file; capture stderr output; assert it contains the
      session ID string and that the function would call `exit(1)`. _Requirements: 4.3_
    - `test_cli_resume_no_arg_shows_usage`: simulate `--resume` provided without a value (or
      empty string); assert stderr contains `"--resume <session_id>"`. _Requirements: 4.4_
    - `test_autosave_failure_produces_warning_event`: set up an `AutosaveQueue` with a save dir
      that is unwritable; trigger `on_answer_submit`; receive on the warning channel; assert
      an `AutosaveWarning` is received within 2 s with a non-empty message. _Requirements: 3.3_
    - `test_hint_save_before_hint_content`: assert that `on_hint_reveal` calls `save_session`
      and returns the hint only after save succeeds (verify ordering via side effect on
      temporary file). _Requirements: 3.2_

- [ ] 12. Property-based tests (11 properties, 256 iterations each)
  - [ ] 12.1 Write shared arbitrary generators in a `tests/proptest_helpers.rs` module
    - Implement `arb_session_id()` using the regex strategy `"[!-~]{1,128}"` mapped through
      `SessionId::try_from_str`.
    - Implement `arb_puzzle_type()` using `prop_oneof![Just(Word), Just(Numeric), Just(Logic)]`.
    - Implement `arb_session()` composing the above with `any::<u64>()`, `any::<u32>()`,
      `any::<i64>()`.
    - _Requirements: 1.4, 2.9_

  - [ ]* 12.2 Write property test for Property 1: round-trip serialization
    - Tag: `// Feature: session-persistence, Property 1: round-trip serialization preserves all fields`
    - Config: `ProptestConfig::with_cases(256)`
    - Serialize `session` to JSON string; deserialize back; `prop_assert_eq!(session, restored)`.
    - **Property 1: Round-Trip Serialization Preserves All Fields**
    - **Validates: Requirements 1.4, 2.9, 6.6**

  - [ ]* 12.3 Write property test for Property 2: serialized JSON shape and version
    - Tag: `// Feature: session-persistence, Property 2: serialized JSON has correct shape and embedded version`
    - Config: `ProptestConfig::with_cases(256)`
    - Parse to `serde_json::Value`; assert all six required keys present with correct JSON
      types; assert `version == SAVE_FORMAT_VERSION`.
    - **Property 2: Serialized JSON Has Correct Shape and Embedded Version**
    - **Validates: Requirements 1.3, 6.2**

  - [ ]* 12.4 Write property test for Property 3: corrupted input returns `SaveError::Corrupted`
    - Tag: `// Feature: session-persistence, Property 3: corrupted input always returns SaveError::Corrupted`
    - Config: `ProptestConfig::with_cases(256)`
    - Generate arbitrary byte strings via `".*"`; skip strings that happen to parse as valid
      sessions; assert `Err(SaveError::Corrupted { message })` with non-empty `message` and no
      panic.
    - **Property 3: Corrupted Input Always Returns `SaveError::Corrupted` with Non-Empty Message**
    - **Validates: Requirements 1.5, 5.1, 6.7**

  - [ ]* 12.5 Write property test for Property 4: version mismatch returns correct fields
    - Tag: `// Feature: session-persistence, Property 4: version mismatch returns correct found/expected`
    - Config: `ProptestConfig::with_cases(256)`
    - Filter `any::<u32>()` to exclude `SAVE_FORMAT_VERSION`; inject `v` as `version` into
      otherwise-valid session JSON; call `load_from_value`; assert
      `Err(SaveError::VersionMismatch { found: v, expected: SAVE_FORMAT_VERSION })`.
    - **Property 4: Version Mismatch Returns Correct `found` and `expected` Values**
    - **Validates: Requirements 5.2, 6.3, 6.4**

  - [ ]* 12.6 Write property test for Property 5: autosave snapshot immutability
    - Tag: `// Feature: session-persistence, Property 5: autosave snapshot is immutable after trigger`
    - Config: `ProptestConfig::with_cases(256)`
    - Clone session as `snapshot`, save `snapshot` to a tempdir path, mutate the original
      session's `hint_count` by `delta`, load the saved file, assert `loaded == snapshot`.
    - **Property 5: Autosave Snapshot Is Immutable After Trigger**
    - **Validates: Requirements 3.4**

  - [ ]* 12.7 Write property test for Property 6: autosave queue processes all events
    - Tag: `// Feature: session-persistence, Property 6: autosave queue processes all events`
    - Config: `ProptestConfig::with_cases(256)`
    - For N in 2–10, submit N `AutosaveEvent::Save` to the queue in rapid succession; after a
      brief wait, assert that N save files exist in the tempdir (none dropped).
    - **Property 6: Autosave Queue Processes All Events**
    - **Validates: Requirements 3.5**

  - [ ]* 12.8 Write property test for Property 7: `SessionId` validity and uniqueness
    - Tag: `// Feature: session-persistence, Property 7: session_id validity and uniqueness`
    - Config: `ProptestConfig::with_cases(256)`
    - For N in 1..=20, generate N `SessionId::new_unique()` values; assert each is 1–128
      chars, all chars are `ascii_graphic`; assert all IDs are pairwise unique.
    - **Property 7: `SessionId` Validity and Uniqueness**
    - **Validates: Requirements 6.5**

  - [ ]* 12.9 Write property test for Property 8: CLI error message contains session ID
    - Tag: `// Feature: session-persistence, Property 8: CLI error message contains session ID for missing file`
    - Config: `ProptestConfig::with_cases(256)`
    - Generate valid session ID strings (1–36 printable non-whitespace); invoke
      `report_error_and_exit` with a `SaveError::Io` for a non-existent path; capture the
      formatted error string; assert it contains the session ID literal.
    - **Property 8: CLI Error Message Contains Session ID for Missing File**
    - **Validates: Requirements 4.3**

  - [ ]* 12.10 Write property test for Property 9: any `SaveError` produces non-empty stderr and exit 1
    - Tag: `// Feature: session-persistence, Property 9: any SaveError produces non-empty stderr and non-zero exit`
    - Config: `ProptestConfig::with_cases(256)`
    - Generate all three `SaveError` variants via `prop_oneof!`; pass each to the error
      formatter; assert the resulting string is non-empty and exit code would be 1.
    - **Property 9: Any `SaveError` Produces Non-Empty Stderr and Non-Zero Exit**
    - **Validates: Requirements 5.4**

  - [ ]* 12.11 Write property test for Property 10: `localStorage` key length ≤ 128 chars
    - Tag: `// Feature: session-persistence, Property 10: localStorage key length does not exceed 128 characters`
    - Config: `ProptestConfig::with_cases(256)`
    - Use `arb_session_id()`; call `session_id.as_str()`; assert `len() <= 128`.
    - **Property 10: `localStorage` Key Length Does Not Exceed 128 Characters**
    - **Validates: Requirements 2.7**

  - [ ]* 12.12 Write property test for Property 11: `--resume` accepts valid / rejects invalid IDs
    - Tag: `// Feature: session-persistence, Property 11: --resume accepts valid and rejects invalid session IDs`
    - Config: `ProptestConfig::with_cases(256)`
    - For strings of 1–36 printable non-whitespace chars: assert validation passes (no
      usage-error path taken). For empty strings, whitespace-containing strings, or strings
      longer than 36 chars: assert validation fails with usage guidance written.
    - **Property 11: `--resume` Accepts Valid and Rejects Invalid Session IDs**
    - **Validates: Requirements 4.1**

- [ ] 13. Final checkpoint — full test suite and lint pass
  - Run `cargo test -p puzzle-persistence`, `cargo clippy -p puzzle-persistence -- -D warnings`,
    and confirm zero failures and zero lint violations.
  - Confirm `wasm-pack test --headless --chrome` passes for the WASM backend tests.
  - Ensure all tests pass, ask the user if questions arise.

---

## Notes

- Tasks marked with `*` are optional and can be skipped for an MVP build.
- All 11 property tests (12.2–12.12) must run at 256 iterations each via `ProptestConfig::with_cases(256)`.
- The `--resume` validation accepts strings up to 36 characters (UUID v4 length) per Req 4.1, while `SessionId` internally allows up to 128 characters for future flexibility.
- The autosave worker in task 8.2 uses a `sync_channel(64)` (bounded) to prevent unbounded memory growth; `send` (blocking) is preferred over `try_send` to ensure no event is dropped (Req 3.5).
- `on_hint_reveal` (task 8.3) performs a synchronous save (not queued) to satisfy the save-before-display ordering guarantee of Req 3.2.
- The WASM backend (task 5.1) uses `SessionId::as_str()` directly as the localStorage key with no prefix, ensuring the ≤128-char constraint is structurally satisfied by the `SessionId` validator rather than additional runtime logic.
- Temp files (task 4.2) are placed in the same directory as the target path to avoid EXDEV cross-device rename failures.

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "1.2"] },
    { "id": 1, "tasks": ["2.1", "2.2"] },
    { "id": 2, "tasks": ["2.3"] },
    { "id": 3, "tasks": ["2.4", "3.1"] },
    { "id": 4, "tasks": ["4.1"] },
    { "id": 5, "tasks": ["4.2", "5.1"] },
    { "id": 6, "tasks": ["6.1", "6.2"] },
    { "id": 7, "tasks": ["8.1"] },
    { "id": 8, "tasks": ["8.2", "9.1"] },
    { "id": 9, "tasks": ["8.3", "9.2", "10.1"] },
    { "id": 10, "tasks": ["11.1", "11.2", "11.3", "12.1"] },
    { "id": 11, "tasks": ["12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "12.8", "12.9", "12.10", "12.11", "12.12"] }
  ]
}
```
