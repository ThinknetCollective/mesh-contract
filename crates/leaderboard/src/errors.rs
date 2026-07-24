use thiserror::Error;
use serde_json;
use std::io;

/// Comprehensive, crate-wide custom error enum for the Mesh ecosystem.
#[derive(Debug, Error)]
pub enum GameError {
    #[error("I/O failure: {0}")]
    Io(#[from] io::Error),

    #[error("Malformed configuration input: {0}")]
    Config(String),

    #[error("State persistence issue: {0}")]
    Persistence(String),

    #[error("Serialization/deserialization drop: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal engine panic: {0}")]
    Internal(String),

    #[error("Invalid argument: {0}")]
    InvalidInput(String),

    #[error("Puzzle error: {0}")]
    Puzzle(String),

    #[error("Unknown error occurred")]
    Unknown,
}

/// A specialized Result type for Game operations.
pub type GameResult<T> = Result<T, GameError>;
