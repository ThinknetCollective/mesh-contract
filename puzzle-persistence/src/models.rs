use serde::{Deserialize, Serialize};

use crate::error::SaveError;
use crate::version::SAVE_FORMAT_VERSION;

// ---------------------------------------------------------------------------
// PuzzleType
// ---------------------------------------------------------------------------

/// The category of puzzle contained in a session.
///
/// Adding new variants here is backward-compatible: existing save files that
/// contain an unknown variant will fail with `SaveError::Corrupted`, which is
/// the correct behaviour until those saves are migrated or restarted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PuzzleType {
    Word,
    Numeric,
    Logic,
}

// ---------------------------------------------------------------------------
// PuzzleState
// ---------------------------------------------------------------------------

/// The mutable state of the active puzzle.
///
/// `data` stores puzzle-type-specific payload as an opaque JSON value so the
/// persistence crate remains decoupled from concrete puzzle implementations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PuzzleState {
    pub puzzle_type: PuzzleType,
    /// A stable identifier for the puzzle definition (e.g. a database ID or slug).
    pub puzzle_id: String,
    /// Puzzle-specific runtime data serialised as a JSON value.
    pub data: serde_json::Value,
    pub is_solved: bool,
}

impl Default for PuzzleType {
    fn default() -> Self {
        PuzzleType::Word
    }
}

// ---------------------------------------------------------------------------
// SessionId
// ---------------------------------------------------------------------------

/// A validated session identifier.
///
/// Constraints:
/// - Length: 1–128 characters (inclusive)
/// - Characters: printable, non-whitespace ASCII (`0x21`–`0x7E`)
///
/// These constraints are enforced both at construction time and during
/// deserialisation via the `TryFrom<String>` serde bridge.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SessionId(String);

impl SessionId {
    /// Generate a new, globally unique `SessionId` backed by a UUID v4.
    pub fn new_unique() -> Self {
        // uuid::Uuid::new_v4() never produces an empty or invalid string so
        // the `try_from_str` call below is infallible in practice.  We use
        // a fallback to the literal "fallback-id" purely to satisfy the
        // clippy::expect_used lint — the branch is unreachable.
        let s = uuid::Uuid::new_v4().to_string();
        match Self::try_from_str(&s) {
            Ok(id) => id,
            Err(_) => SessionId(s), // unreachable; belt-and-suspenders
        }
    }

    /// Validate and wrap an existing string.
    ///
    /// Returns `SaveError::Corrupted` if the string is empty, longer than 128
    /// characters, or contains whitespace or non-ASCII-graphic characters.
    pub fn try_from_str(s: &str) -> Result<Self, SaveError> {
        if s.is_empty() || s.len() > 128 {
            return Err(SaveError::Corrupted {
                message: format!(
                    "session_id must be 1–128 characters, got {}",
                    s.len()
                ),
            });
        }
        if s.chars().any(|c| c.is_whitespace() || !c.is_ascii_graphic()) {
            return Err(SaveError::Corrupted {
                message: "session_id contains invalid characters (must be printable non-whitespace ASCII)".to_string(),
            });
        }
        Ok(SessionId(s.to_string()))
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SessionId {
    type Error = SaveError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from_str(&s)
    }
}

impl From<SessionId> for String {
    fn from(id: SessionId) -> Self {
        id.0
    }
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// The complete runtime state of one puzzle attempt.
///
/// This struct is the primary unit of serialisation.  Every field is required
/// in the JSON representation; missing fields during deserialisation cause a
/// `SaveError::Corrupted`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Schema version embedded in every save file.  Always set to
    /// [`SAVE_FORMAT_VERSION`] when creating a new session.
    pub version: u32,
    /// Stable unique identifier assigned at session creation.
    pub session_id: SessionId,
    /// The active puzzle and its mutable state.
    pub puzzle_state: PuzzleState,
    /// Elapsed time in whole seconds since the session started.
    pub elapsed_time: u64,
    /// Number of hints revealed so far in this session.
    pub hint_count: u32,
    /// Score accumulated so far (may be 0 until puzzle completion).
    pub score: i64,
}

impl Session {
    /// Create a new session for the given puzzle state, setting the schema
    /// version and generating a unique `session_id`.
    pub fn new(puzzle_state: PuzzleState) -> Self {
        Session {
            version: SAVE_FORMAT_VERSION,
            session_id: SessionId::new_unique(),
            puzzle_state,
            elapsed_time: 0,
            hint_count: 0,
            score: 0,
        }
    }
}
