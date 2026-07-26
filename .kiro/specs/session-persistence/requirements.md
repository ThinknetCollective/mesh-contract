# Requirements Document

## Introduction

This feature adds a persistence layer to the puzzle game so that an in-progress session can be saved and later restored. Currently, quitting the game mid-session — whether through the TUI or a future WASM/web frontend — permanently discards all progress. The persistence layer will serialize the full session state (puzzle state, elapsed time, hint count, and score-in-progress) to disk on native targets and to `localStorage` on WASM targets. Sessions can be resumed via a `--resume` CLI flag. Save files that are corrupted or version-mismatched are rejected gracefully with a descriptive error rather than a panic.

---

## Glossary

- **Session**: The complete runtime state of one puzzle attempt, including the active puzzle, elapsed time, hint count, and score-in-progress.
- **Session_ID**: A stable, unique identifier assigned to a Session at creation time and preserved across saves and loads.
- **Persistence_Module**: The Rust module (`persistence`) responsible for serializing and deserializing Session data.
- **Save_File**: A file on disk (native) or a `localStorage` entry (WASM) that contains a serialized Session.
- **Puzzle_Type**: A variant of the puzzle domain (e.g., word, numeric, logic). At least three distinct types must be supported.
- **Autosave**: An automatic, transparent save triggered by specific in-game events without requiring explicit user action.
- **CLI**: The command-line interface through which the game is launched and controlled.
- **TUI**: The terminal user interface rendered during native gameplay.
- **WASM**: The WebAssembly compilation target used for browser-based gameplay.
- **serde**: The Rust serialization/deserialization framework (`serde` + `serde_json` or equivalent).
- **SaveError**: The error type returned by the Persistence_Module when a save or load operation fails.

---

## Requirements

### Requirement 1: Session Serialization Support

**User Story:** As a developer, I want the Session struct to be serializable and deserializable using serde, so that session state can be converted to and from a portable format for storage and retrieval.

#### Acceptance Criteria

1. THE Session SHALL derive `serde::Serialize` and `serde::Deserialize`.
2. THE Session SHALL include a `version` field of type `u32` that identifies the schema version of the serialized format.
3. WHEN the Session is serialized, THE Persistence_Module SHALL produce a valid JSON object whose top-level keys include `puzzle_state`, `elapsed_time`, `hint_count`, `score`, `session_id`, and `version`; each key SHALL map to a value of the correct JSON type for that field.
4. FOR ALL valid Session values, serializing then deserializing SHALL produce a Session whose fields are equal to those of the original value field-by-field (round-trip property).
5. IF deserialization of a Save_File fails due to missing required fields or unexpected types, THEN THE Persistence_Module SHALL return a `SaveError::Corrupted` variant rather than panicking.

---

### Requirement 2: Save and Load Operations

**User Story:** As a player, I want my session to be saved to and loaded from storage, so that I can quit the game and resume my progress later without losing any state.

#### Acceptance Criteria

1. THE Persistence_Module SHALL expose a `save_session(session: &Session, path: &Path) -> Result<(), SaveError>` function for native targets.
2. THE Persistence_Module SHALL expose a `load_session(path: &Path) -> Result<Session, SaveError>` function for native targets.
3. WHEN `save_session` is called with a valid Session and a writable path, THE Persistence_Module SHALL atomically write the complete serialized Save_File to that path and return `Ok(())`; either the complete file is written or any pre-existing file at that path is left unchanged.
4. IF `save_session` fails due to a write error (disk full, permissions denied, or other I/O error), THEN THE Persistence_Module SHALL return a `SaveError::Io` variant wrapping the underlying error and the pre-existing file at that path SHALL remain unchanged.
5. WHEN `load_session` is called with a path to a Save_File that is structurally valid and schema-compatible, THE Persistence_Module SHALL deserialize and return the stored Session.
6. IF `load_session` is called with a path to a file that is absent, corrupted, or schema-incompatible, THEN THE Persistence_Module SHALL return the appropriate `SaveError` variant and no partial Session state SHALL be returned.
7. WHERE the WASM feature flag is enabled, THE Persistence_Module SHALL save and load Session data using the browser `localStorage` API instead of the filesystem, using the Session_ID as the storage key; the localStorage key SHALL be at most 128 characters long.
8. IF the WASM target cannot access `localStorage` (unavailable or quota exceeded), THEN THE Persistence_Module SHALL return a `SaveError::Io` variant with a description of the storage failure.
9. WHEN a save followed by a load is performed for a valid Session of any of at least 3 distinct Puzzle_Types, THE loaded Session SHALL have puzzle state, elapsed time (in whole seconds), hint count, and score-in-progress equal to those of the original Session.

---

### Requirement 3: Autosave on Game Events

**User Story:** As a player, I want my session to be automatically saved when I submit an answer or reveal a hint, so that my progress is preserved without requiring manual save actions.

#### Acceptance Criteria

1. WHEN a player submits an answer, THE Session SHALL be saved to its associated Save_File within 2 seconds of submission, regardless of whether the result is subsequently displayed.
2. WHEN a player reveals a hint, THE Session SHALL be saved to its associated Save_File before the hint content is displayed.
3. IF an Autosave operation fails, THEN THE TUI SHALL display a warning message that does not block player input, includes the reason for failure, and automatically dismisses after 5 seconds; gameplay SHALL continue uninterrupted.
4. WHILE an Autosave is in progress, THE Session's score, hint count, and current puzzle state visible to the player SHALL remain identical to the values present at the moment the Autosave was triggered.
5. IF two Autosave-triggering events occur before the first Autosave completes, THEN THE Persistence_Module SHALL queue the second save and execute it after the first completes, ensuring no save event is silently dropped.

---

### Requirement 4: Session Resume via CLI Flag

**User Story:** As a player, I want to resume a previously saved session using a CLI flag, so that I can return to exactly where I left off after quitting.

#### Acceptance Criteria

1. THE CLI SHALL accept a `--resume <session_id>` flag where `<session_id>` is a string of 1 to 36 non-whitespace printable characters that identifies the Session to restore.
2. WHEN `--resume <session_id>` is provided and a matching Save_File exists, THE CLI SHALL load the Session and launch the TUI with the restored puzzle state, elapsed time, hint count, and score-in-progress.
3. WHEN `--resume <session_id>` is provided and no matching Save_File is found, THE CLI SHALL write a human-readable error message that includes the unresolved Session_ID to stderr and exit with a non-zero status code.
4. WHEN `--resume` is provided without a following `<session_id>` value, THE CLI SHALL write a message to stderr that contains the text `--resume <session_id>` (showing correct usage) and exit with a non-zero status code.
5. WHEN `--resume <session_id>` is provided and the matching Save_File is corrupted or version-mismatched, THE CLI SHALL write a human-readable error message that identifies the file and the nature of the error to stderr and exit with a non-zero status code.

---

### Requirement 5: Graceful Handling of Corrupted or Version-Mismatched Save Files

**User Story:** As a player, I want the game to fail clearly and safely when a save file is corrupted or incompatible, so that the game never panics or produces silent data loss.

#### Acceptance Criteria

1. IF a Save_File cannot be deserialized due to malformed content (invalid JSON, missing required fields, or unexpected value types), THEN THE Persistence_Module SHALL return a `SaveError::Corrupted` variant whose message field contains a non-empty description of the parse failure.
2. IF a Save_File contains a `version` field with a value that differs from `SAVE_FORMAT_VERSION`, THEN THE Persistence_Module SHALL return `SaveError::VersionMismatch { found: u32, expected: u32 }` where `found` is the value read from the file and `expected` is the current `SAVE_FORMAT_VERSION`; no Session value SHALL be returned.
3. THE Persistence_Module SHALL contain no calls to `unwrap()` or `expect()` on values produced by I/O operations or deserialization operations; all such fallible paths SHALL propagate errors via the `Result` type.
4. WHEN a `SaveError` is returned to the CLI entry point, THE CLI SHALL write a human-readable message derived from the error to stderr AND exit with a non-zero status code; omitting either action is not permitted.
5. IF a Save_File path is not readable due to filesystem permissions or absence, THEN THE Persistence_Module SHALL return a `SaveError::Io` variant that wraps the underlying `std::io::Error`.

---

### Requirement 6: Save File Format and Versioning

**User Story:** As a developer, I want save files to carry a schema version, so that future changes to the Session format can be detected and migration or rejection can be handled explicitly.

#### Acceptance Criteria

1. THE Persistence_Module SHALL define a compile-time constant `SAVE_FORMAT_VERSION: u32` that SHALL be incremented whenever the Session schema changes in a backward-incompatible way, where backward-incompatible changes include: removing a field, renaming a field, changing a field's type, or changing the meaning of an existing field's value.
2. WHEN a Save_File is written, THE Persistence_Module SHALL embed the current `SAVE_FORMAT_VERSION` value in the serialized output as the `version` field.
3. WHEN a Save_File is read and its embedded `version` value is less than `SAVE_FORMAT_VERSION`, THE Persistence_Module SHALL return `SaveError::VersionMismatch { found, expected }` including the embedded version and the current constant; no Session SHALL be returned.
4. WHEN a Save_File is read and its embedded `version` value is greater than `SAVE_FORMAT_VERSION`, THE Persistence_Module SHALL return `SaveError::VersionMismatch { found, expected }` including the embedded version and the current constant; no Session SHALL be returned.
5. WHEN a new Session is created, THE Session SHALL be assigned a `session_id` of type `String` that is between 1 and 128 characters in length, contains only printable non-whitespace characters, and is unique among all Sessions created during the same process lifetime.
6. WHEN a Session is loaded from a Save_File, THE loaded Session SHALL have a `session_id` field equal to the value stored in the Save_File, with no modification.
7. IF a Save_File is loaded and its `session_id` field is absent or empty, THEN THE Persistence_Module SHALL return `SaveError::Corrupted` with a message indicating the missing session identifier.
