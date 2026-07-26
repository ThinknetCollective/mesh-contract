use crate::errors::GameError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// An individual entry recorded on the leaderboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    /// Name of the player
    pub player_name: String,
    /// Type or identifier of the puzzle completed
    pub puzzle_type: String,
    /// Score achieved by the player
    pub score: u64,
    /// Unix timestamp (in seconds) when score was recorded
    pub timestamp: u64,
}

impl LeaderboardEntry {
    /// Creates a new leaderboard entry with the specified parameters.
    pub fn new(player_name: impl Into<String>, puzzle_type: impl Into<String>, score: u64, timestamp: u64) -> Self {
        Self {
            player_name: player_name.into(),
            puzzle_type: puzzle_type.into(),
            score,
            timestamp,
        }
    }
}

/// Criteria used to sort leaderboard entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SortBy {
    /// Sort by highest score first
    #[default]
    Score,
    /// Sort by most recent timestamp first
    Recency,
}

impl std::str::FromStr for SortBy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "score" => Ok(SortBy::Score),
            "recency" => Ok(SortBy::Recency),
            _ => Err(format!("Unknown sort order '{}'. Options: score, recency", s)),
        }
    }
}

impl fmt::Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortBy::Score => write!(f, "score"),
            SortBy::Recency => write!(f, "recency"),
        }
    }
}

/// Errors returned during leaderboard operations.
pub type LeaderboardError = GameError;
