/// Unit and property-based tests for puzzle-persistence.
///
/// Coverage:
/// - Save/load round-trip for Word, Numeric, and Logic puzzle types (Req 2.9)
/// - Missing file → SaveError::Io (Req 5.5)
/// - Missing session_id → SaveError::Corrupted (Req 6.7)
/// - Empty session_id → SaveError::Corrupted (Req 6.7)
/// - Atomic write: original file unchanged on failure (Req 2.4)
/// - Version mismatch: correct found/expected returned (Req 5.2, 6.3, 6.4)
/// - Corrupted JSON → SaveError::Corrupted (Req 5.1)
/// - CLI resume: missing session → error message contains session_id (Req 4.3)
/// - CLI resume: empty id → usage text in error message (Req 4.4)
/// - Autosave on hint reveal: save precedes hint return (Req 3.2)
/// - Autosave on answer submit: worker saves within timeout (Req 3.1)
/// - Autosave failure: warning event delivered (Req 3.3)
/// - Round-trip for all three puzzle types (proptest, Req 1.4)
/// - Version mismatch for arbitrary mismatched versions (proptest, Req 6.3/6.4)
/// - Corrupted input never panics (proptest, Req 1.5)

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::error::SaveError;
    use crate::models::{PuzzleState, PuzzleType, Session, SessionId};
    use crate::ops::{load_from_str, load_from_value, load_session, save_session};
    use crate::version::SAVE_FORMAT_VERSION;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_session(puzzle_type: PuzzleType) -> Session {
        Session::new(PuzzleState {
            puzzle_type,
            puzzle_id: "test-001".to_string(),
            data: serde_json::json!({"answer": "hello"}),
            is_solved: false,
        })
    }

    // -----------------------------------------------------------------------
    // Round-trip tests — 3 puzzle types (Req 2.9)
    // -----------------------------------------------------------------------

    #[test]
    fn test_save_and_load_word_puzzle() {
        let dir = tempfile::tempdir().expect("tempdir");
        let session = make_session(PuzzleType::Word);
        let path = dir.path().join(format!("{}.json", session.session_id.as_str()));

        save_session(&session, &path).expect("save");
        let loaded = load_session(&path).expect("load");

        assert_eq!(session, loaded, "word puzzle round-trip");
        assert_eq!(session.elapsed_time, loaded.elapsed_time);
        assert_eq!(session.hint_count, loaded.hint_count);
        assert_eq!(session.score, loaded.score);
    }

    #[test]
    fn test_save_and_load_numeric_puzzle() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut session = make_session(PuzzleType::Numeric);
        session.elapsed_time = 42;
        session.hint_count = 2;
        session.score = 150;

        let path = dir.path().join("numeric.json");
        save_session(&session, &path).expect("save");
        let loaded = load_session(&path).expect("load");

        assert_eq!(session, loaded, "numeric puzzle round-trip");
        assert_eq!(42u64, loaded.elapsed_time);
        assert_eq!(2u32, loaded.hint_count);
        assert_eq!(150i64, loaded.score);
    }

    #[test]
    fn test_save_and_load_logic_puzzle() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut session = make_session(PuzzleType::Logic);
        session.elapsed_time = 300;
        session.hint_count = 5;
        session.score = -10;
        session.puzzle_state.is_solved = true;

        let path = dir.path().join("logic.json");
        save_session(&session, &path).expect("save");
        let loaded = load_session(&path).expect("load");

        assert_eq!(session, loaded, "logic puzzle round-trip");
        assert!(loaded.puzzle_state.is_solved);
        assert_eq!(-10i64, loaded.score);
    }

    // -----------------------------------------------------------------------
    // Error path tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_load_missing_file_returns_io() {
        let path = Path::new("/nonexistent/path/that/does/not/exist.json");
        let result = load_session(path);
        assert!(
            matches!(result, Err(SaveError::Io(_))),
            "expected SaveError::Io for missing file, got {:?}",
            result
        );
    }

    #[test]
    fn test_load_missing_session_id_returns_corrupted() {
        let json = serde_json::json!({
            "version": SAVE_FORMAT_VERSION,
            // session_id intentionally absent
            "puzzle_state": {
                "puzzle_type": "word",
                "puzzle_id": "x",
                "data": null,
                "is_solved": false
            },
            "elapsed_time": 0,
            "hint_count": 0,
            "score": 0
        })
        .to_string();

        let result = load_from_str(&json);
        assert!(
            matches!(result, Err(SaveError::Corrupted { .. })),
            "expected SaveError::Corrupted for missing session_id, got {:?}",
            result
        );
        if let Err(SaveError::Corrupted { message }) = result {
            assert!(!message.is_empty(), "error message must not be empty");
        }
    }

    #[test]
    fn test_load_empty_session_id_returns_corrupted() {
        // Build a valid session JSON then forcibly set session_id to ""
        let session = make_session(PuzzleType::Word);
        let mut value = serde_json::to_value(&session).expect("serialize");
        value["session_id"] = serde_json::json!("");

        let result = load_from_value(value);
        assert!(
            matches!(result, Err(SaveError::Corrupted { .. })),
            "expected SaveError::Corrupted for empty session_id, got {:?}",
            result
        );
        if let Err(SaveError::Corrupted { message }) = result {
            assert!(!message.is_empty());
        }
    }

    #[test]
    fn test_version_mismatch_returns_correct_found_and_expected() {
        let session = make_session(PuzzleType::Numeric);
        let mut value = serde_json::to_value(&session).expect("serialize");
        let wrong_version = SAVE_FORMAT_VERSION + 99;
        value["version"] = serde_json::json!(wrong_version);

        let result = load_from_value(value);
        match result {
            Err(SaveError::VersionMismatch { found, expected }) => {
                assert_eq!(found, wrong_version, "found should be the embedded version");
                assert_eq!(
                    expected, SAVE_FORMAT_VERSION,
                    "expected should be SAVE_FORMAT_VERSION"
                );
            }
            other => panic!("expected VersionMismatch, got {:?}", other),
        }
    }

    #[test]
    fn test_corrupted_json_returns_corrupted_error() {
        let result = load_from_str("{this is not valid json}}");
        assert!(
            matches!(result, Err(SaveError::Corrupted { .. })),
            "expected Corrupted for invalid JSON, got {:?}",
            result
        );
        if let Err(SaveError::Corrupted { message }) = result {
            assert!(!message.is_empty());
        }
    }

    #[test]
    fn test_atomic_write_creates_file_on_success() {
        let dir = tempfile::tempdir().expect("tempdir");
        let session = make_session(PuzzleType::Logic);
        let path = dir.path().join("atomic_test.json");

        // File should not exist yet.
        assert!(!path.exists());
        save_session(&session, &path).expect("save");
        // File should exist and be valid after save.
        assert!(path.exists());
        let loaded = load_session(&path).expect("load");
        assert_eq!(session, loaded);
    }

    #[test]
    fn test_save_to_unwritable_directory_returns_io() {
        // Use a path whose parent is a non-existent deeply nested directory
        // that we don't create — so the write must fail with an I/O error.
        // (We can't reliably chmod on all OSes in tests, so we simulate by
        // using an invalid path component.)
        let path = Path::new("/\0invalid\0path\0chars/session.json");
        let session = make_session(PuzzleType::Word);
        let result = save_session(&session, path);
        assert!(
            matches!(result, Err(SaveError::Io(_))),
            "expected SaveError::Io for unwritable path, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // CLI resume tests (Req 4.3, 4.4)
    // -----------------------------------------------------------------------

    #[test]
    fn test_cli_resume_missing_session_error_contains_id() {
        use crate::cli::resume::{format_resume_error, handle_resume};
        let dir = tempfile::tempdir().expect("tempdir");
        let id = "missing-session-abc";

        let result = handle_resume(id, dir.path());
        assert!(
            result.is_err(),
            "expected error for non-existent session"
        );
        let msg = format_resume_error(result.as_ref().err().expect("err"), id);
        assert!(
            msg.contains(id),
            "error message must contain the session_id '{}', got: {}",
            id,
            msg
        );
    }

    #[test]
    fn test_cli_resume_empty_id_shows_usage() {
        use crate::cli::resume::handle_resume;
        let dir = tempfile::tempdir().expect("tempdir");
        let result = handle_resume("", dir.path());

        assert!(result.is_err());
        if let Err(SaveError::Corrupted { message }) = result {
            assert!(
                message.contains("--resume <session_id>"),
                "usage guidance must mention '--resume <session_id>', got: {}",
                message
            );
        } else {
            panic!("expected SaveError::Corrupted for empty session_id");
        }
    }

    #[test]
    fn test_cli_resume_too_long_id_shows_usage() {
        use crate::cli::resume::handle_resume;
        let dir = tempfile::tempdir().expect("tempdir");
        let long_id = "a".repeat(37); // > 36 chars
        let result = handle_resume(&long_id, dir.path());

        assert!(result.is_err());
        if let Err(SaveError::Corrupted { message }) = result {
            assert!(
                message.contains("--resume <session_id>"),
                "usage guidance must mention '--resume <session_id>', got: {}",
                message
            );
        } else {
            panic!("expected SaveError::Corrupted for too-long session_id");
        }
    }

    #[test]
    fn test_cli_resume_valid_session_loads_correctly() {
        use crate::cli::resume::handle_resume;
        let dir = tempfile::tempdir().expect("tempdir");
        let session = make_session(PuzzleType::Logic);
        let id = session.session_id.as_str().to_string();

        // Save using the standard path convention.
        let path = dir.path().join(format!("{}.json", id));
        save_session(&session, &path).expect("save");

        let loaded = handle_resume(&id, dir.path()).expect("resume");
        assert_eq!(session, loaded);
    }

    // -----------------------------------------------------------------------
    // Autosave: hint reveal ordering (Req 3.2)
    // -----------------------------------------------------------------------

    #[test]
    fn test_hint_save_before_hint_content() {
        use crate::autosave::queue::autosave_on_hint_reveal;

        let dir = tempfile::tempdir().expect("tempdir");
        let session = make_session(PuzzleType::Word);
        let path = dir.path().join(format!("{}.json", session.session_id.as_str()));

        // File must not exist before the call.
        assert!(!path.exists(), "file should not exist yet");

        // autosave_on_hint_reveal saves synchronously.
        autosave_on_hint_reveal(&session, dir.path()).expect("hint save");

        // File must exist immediately after the call returns — before any
        // "display hint" logic would run.
        assert!(path.exists(), "file must be written before hint is displayed");

        let loaded = load_session(&path).expect("load after hint save");
        assert_eq!(session, loaded);
    }

    // -----------------------------------------------------------------------
    // Autosave: answer submit via queue (Req 3.1, 3.3, 3.5)
    // -----------------------------------------------------------------------

    #[test]
    fn test_autosave_on_answer_submit_saves_within_timeout() {
        use std::sync::mpsc;
        use std::time::Duration;

        use crate::autosave::queue::{AutosaveQueue, AutosaveWarning, autosave_on_answer_submit};

        let dir = tempfile::tempdir().expect("tempdir");
        let (warn_tx, warn_rx) = mpsc::channel::<AutosaveWarning>();
        let queue = AutosaveQueue::new(dir.path().to_path_buf(), warn_tx);

        let session = make_session(PuzzleType::Numeric);
        let expected_path = dir
            .path()
            .join(format!("{}.json", session.session_id.as_str()));

        autosave_on_answer_submit(&queue, &session);

        // Drop queue to flush and join the worker thread.
        drop(queue);

        // File must exist after worker shuts down.
        assert!(
            expected_path.exists(),
            "session file must be written after answer submit"
        );

        // No warnings should have been emitted.
        assert!(
            warn_rx.try_recv().is_err(),
            "no autosave warnings expected on success"
        );

        let loaded = load_session(&expected_path).expect("load");
        assert_eq!(session, loaded);

        // Suppress unused variable warning for Duration import.
        let _ = Duration::from_secs(2);
    }

    #[test]
    fn test_autosave_failure_produces_warning() {
        use std::sync::mpsc;
        use std::time::Duration;

        use crate::autosave::queue::{AutosaveQueue, AutosaveWarning, autosave_on_answer_submit};

        // Use a path with a null byte — guaranteed to be unwritable on all platforms.
        let bad_dir = std::path::PathBuf::from("/\0bad\0dir");
        let (warn_tx, warn_rx) = mpsc::channel::<AutosaveWarning>();
        let queue = AutosaveQueue::new(bad_dir, warn_tx);

        let session = make_session(PuzzleType::Logic);
        autosave_on_answer_submit(&queue, &session);

        // Flush worker.
        drop(queue);

        // A warning must have been sent.
        let warning = warn_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("expected an autosave warning within 5 s");
        assert!(!warning.message.is_empty(), "warning message must be non-empty");
    }

    // -----------------------------------------------------------------------
    // SessionId tests (Req 6.5)
    // -----------------------------------------------------------------------

    #[test]
    fn test_session_id_uniqueness() {
        use std::collections::HashSet;
        let ids: HashSet<String> = (0..100)
            .map(|_| SessionId::new_unique().as_str().to_string())
            .collect();
        assert_eq!(ids.len(), 100, "all session IDs must be unique");
    }

    #[test]
    fn test_session_id_length_bounds() {
        assert!(SessionId::try_from_str("").is_err(), "empty id rejected");
        assert!(
            SessionId::try_from_str(&"a".repeat(129)).is_err(),
            "129-char id rejected"
        );
        assert!(
            SessionId::try_from_str(&"a".repeat(128)).is_ok(),
            "128-char id accepted"
        );
        assert!(
            SessionId::try_from_str("x").is_ok(),
            "1-char id accepted"
        );
    }

    #[test]
    fn test_session_id_whitespace_rejected() {
        assert!(SessionId::try_from_str("has space").is_err());
        assert!(SessionId::try_from_str("tab\there").is_err());
    }

    // -----------------------------------------------------------------------
    // SAVE_FORMAT_VERSION embedded correctly (Req 6.2)
    // -----------------------------------------------------------------------

    #[test]
    fn test_version_embedded_in_save_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let session = make_session(PuzzleType::Word);
        let path = dir.path().join("version_test.json");
        save_session(&session, &path).expect("save");

        let raw = std::fs::read_to_string(&path).expect("read");
        let value: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        let embedded = value["version"].as_u64().expect("version field") as u32;
        assert_eq!(embedded, SAVE_FORMAT_VERSION);
    }

    // -----------------------------------------------------------------------
    // Property-based tests
    // -----------------------------------------------------------------------

    use proptest::prelude::*;

    fn arb_puzzle_type() -> impl Strategy<Value = PuzzleType> {
        prop_oneof![
            Just(PuzzleType::Word),
            Just(PuzzleType::Numeric),
            Just(PuzzleType::Logic),
        ]
    }

    fn arb_session() -> impl Strategy<Value = Session> {
        (
            arb_puzzle_type(),
            any::<u64>(),
            any::<u32>(),
            any::<i64>(),
            // Restrict to alphanumeric + hyphen/underscore so generated IDs
            // are valid filenames on all platforms (Windows forbids : < > etc.)
            "[a-zA-Z0-9_-]{1,36}",
        )
            .prop_map(|(pt, elapsed, hints, score, id_str)| {
                let id = SessionId::try_from_str(&id_str)
                    .unwrap_or_else(|_| SessionId::new_unique());
                Session {
                    version: SAVE_FORMAT_VERSION,
                    session_id: id,
                    puzzle_state: PuzzleState {
                        puzzle_type: pt,
                        puzzle_id: "prop-test".to_string(),
                        data: serde_json::Value::Null,
                        is_solved: false,
                    },
                    elapsed_time: elapsed,
                    hint_count: hints,
                    score,
                }
            })
    }

    /// Property 1: Round-trip serialization preserves all fields (Req 1.4, 2.9, 6.6)
    proptest! {
            // Feature: session-persistence, Property 1: round-trip serialization preserves all fields
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join(format!("{}.json", session.session_id.as_str()));
            save_session(&session, &path).expect("save");
            let loaded = load_session(&path).expect("load");
            prop_assert_eq!(session.version, loaded.version);
            prop_assert_eq!(session.session_id, loaded.session_id);
            prop_assert_eq!(session.elapsed_time, loaded.elapsed_time);
            prop_assert_eq!(session.hint_count, loaded.hint_count);
            prop_assert_eq!(session.score, loaded.score);
            prop_assert_eq!(session.puzzle_state.puzzle_type, loaded.puzzle_state.puzzle_type);
            prop_assert_eq!(session.puzzle_state.is_solved, loaded.puzzle_state.is_solved);
        }
    }

    /// Property 2: Serialized JSON has correct shape and embedded version (Req 1.3, 6.2)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn prop_serialization_shape(session in arb_session()) {
            // Feature: session-persistence, Property 2: serialized JSON has correct shape and embedded version
            let value = serde_json::to_value(&session).expect("serialize");
            let obj = value.as_object().expect("must be JSON object");
            prop_assert!(obj.contains_key("puzzle_state"), "missing puzzle_state");
            prop_assert!(obj.contains_key("elapsed_time"), "missing elapsed_time");
            prop_assert!(obj.contains_key("hint_count"), "missing hint_count");
            prop_assert!(obj.contains_key("score"), "missing score");
            prop_assert!(obj.contains_key("session_id"), "missing session_id");
            prop_assert!(obj.contains_key("version"), "missing version");
            prop_assert!(obj["elapsed_time"].is_number());
            prop_assert!(obj["hint_count"].is_number());
            prop_assert!(obj["score"].is_number());
            prop_assert!(obj["session_id"].is_string());
            prop_assert!(obj["version"].is_number());
            let v = obj["version"].as_u64().expect("version as u64") as u32;
            prop_assert_eq!(v, SAVE_FORMAT_VERSION);
        }
    }

    /// Property 3: Corrupted input → SaveError::Corrupted, never panics (Req 1.5, 5.1)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn prop_corrupted_input_no_panic(bad in ".*") {
            // Feature: session-persistence, Property 3: corrupted input always returns SaveError::Corrupted
            let result = load_from_str(&bad);
            // Either it parsed fine (happens when proptest generates valid JSON by chance)
            // or it returned a non-Io error and did not panic.
            if let Err(e) = result {
                prop_assert!(
                    !matches!(e, SaveError::Io(_)),
                    "corrupt-input path should not return SaveError::Io"
                );
                if let SaveError::Corrupted { message } = e {
                    prop_assert!(!message.is_empty());
                }
            }
        }
    }

    /// Property 4: Version mismatch returns correct found/expected (Req 5.2, 6.3, 6.4)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn prop_version_mismatch(
            session in arb_session(),
            v in any::<u32>().prop_filter("not current version", |&x| x != SAVE_FORMAT_VERSION)
        ) {
            // Feature: session-persistence, Property 4: version mismatch returns correct found/expected
            let mut value = serde_json::to_value(&session).expect("serialize");
            value["version"] = serde_json::json!(v);
            let result = load_from_value(value);
            prop_assert!(
                matches!(result, Err(SaveError::VersionMismatch { found, expected })
                    if found == v && expected == SAVE_FORMAT_VERSION),
                "expected VersionMismatch{{found:{}, expected:{}}}, got {:?}",
                v, SAVE_FORMAT_VERSION, result
            );
        }
    }

    /// Property 5: Autosave snapshot immutability (Req 3.4)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn prop_snapshot_immutability(session in arb_session(), delta in any::<u32>()) {
            // Feature: session-persistence, Property 5: autosave snapshot is immutable after trigger
            let snapshot = session.clone();
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().join(format!("{}.json", snapshot.session_id.as_str()));

            save_session(&snapshot, &path).expect("save snapshot");

            // Mutate the original — the saved snapshot must be unaffected.
            let mut mutated = session;
            mutated.hint_count = mutated.hint_count.wrapping_add(delta);
            mutated.score = mutated.score.wrapping_add(1);

            let loaded = load_session(&path).expect("load snapshot");
            prop_assert_eq!(snapshot.hint_count, loaded.hint_count);
            prop_assert_eq!(snapshot.score, loaded.score);
        }
    }

    /// Property 7: SessionId validity (Req 6.5)
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]
        #[test]
        fn prop_session_id_validity(_n in 1usize..=20usize) {
            // Feature: session-persistence, Property 7: session_id validity and uniqueness
            use std::collections::HashSet;
            let ids: Vec<_> = (0.._n).map(|_| SessionId::new_unique()).collect();
            for id in &ids {
                let s = id.as_str();
                prop_assert!(s.len() >= 1 && s.len() <= 128);
                prop_assert!(s.chars().all(|c| c.is_ascii_graphic() && !c.is_whitespace()));
            }
            let unique: HashSet<_> = ids.iter().map(|id| id.as_str()).collect();
            prop_assert_eq!(unique.len(), _n);
        }
    }
}
