//! # puzzle-persistence
//!
//! Session persistence layer for the puzzle game.
//!
//! ## Features
//!
//! | Target   | Storage          | Functions                                     |
//! |----------|------------------|-----------------------------------------------|
//! | native   | Disk (JSON file) | [`save_session`], [`load_session`]            |
//! | wasm32   | `localStorage`   | `save_session_wasm`, `load_session_wasm`      |
//!
//! ## Quick start
//!
//! ```rust
//! use puzzle_persistence::{Session, PuzzleState, PuzzleType, save_session, load_session};
//! use std::path::Path;
//!
//! let state = PuzzleState {
//!     puzzle_type: PuzzleType::Word,
//!     puzzle_id: "crossword-001".to_string(),
//!     data: serde_json::json!({"clues": []}),
//!     is_solved: false,
//! };
//! let session = Session::new(state);
//! let path = Path::new("/tmp/my_session.json");
//!
//! save_session(&session, path).unwrap();
//! let loaded = load_session(path).unwrap();
//! assert_eq!(session, loaded);
//! ```

pub mod error;
pub mod models;
pub mod version;
mod backend;
pub mod ops;

#[cfg(not(target_arch = "wasm32"))]
pub mod autosave;

#[cfg(not(target_arch = "wasm32"))]
pub mod cli;

// Re-export the public surface.
pub use error::SaveError;
pub use models::{PuzzleState, PuzzleType, Session, SessionId};
pub use version::SAVE_FORMAT_VERSION;

#[cfg(not(target_arch = "wasm32"))]
pub use ops::{load_session, save_session};

#[cfg(target_arch = "wasm32")]
pub use ops::{load_session_wasm, save_session_wasm};

#[cfg(test)]
mod tests;
